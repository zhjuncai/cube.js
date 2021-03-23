//! Query planning goes through the following stages:
//!   1. Logical plan produced by DataFusion. Contains table scans of [CubeTableLogical], does not
//!      know which physical nodes it has to query.
//!   2. [choose_index] transformation will replace the index and particular partitions to query.
//!      It will also place [ClusterSendNode] into the correct place.
//!      At this point, the logical plan is finalized, it only scans [CubeTable]s and contains
//!      enough information to distribute the plan into workers.
//!   3. We serialize the resulting logical plan into [SerializedPlan] and send it to workers.
//!   4. [CubeQueryPlanner] is used on both the router and the workers to produce a physical plan.
//!      Note that workers and the router produce different plans:
//!          - Router produces a physical plan that handles the "top" part of the logical plan, above
//!            the cluster send.
//!          - Workers take only the "bottom" part part of the logical plan, below the cluster send.
//!            In addition, workers will replace all table scans of data they do not have with empty
//!            results.
//!
//!       At this point we also optimize the physical plan to ensure we do as much work as possible
//!       on the workers, see [CubeQueryPlanner] for details.
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use arrow::datatypes::Field;
use async_trait::async_trait;
use datafusion::error::DataFusionError;
use datafusion::execution::context::ExecutionContextState;
use datafusion::logical_plan::{DFSchemaRef, Expr, LogicalPlan, UserDefinedLogicalNode};
use datafusion::physical_plan::empty::EmptyExec;
use datafusion::physical_plan::planner::ExtensionPlanner;
use datafusion::physical_plan::{
    ExecutionPlan, OptimizerHints, Partitioning, SendableRecordBatchStream,
};
use flatbuffers::bitflags::_core::any::Any;
use flatbuffers::bitflags::_core::fmt::Formatter;
use itertools::Itertools;

use crate::cluster::Cluster;
use crate::metastore::table::{Table, TablePath};
use crate::metastore::{Chunk, IdRow, Index, MetaStore, Partition, Schema};
use crate::queryplanner::optimizations::rewrite_plan::{rewrite_plan, PlanRewriter};
use crate::queryplanner::partition_filter::PartitionFilter;
use crate::queryplanner::query_executor::{ClusterSendExec, CubeTable};
use crate::queryplanner::serialized_plan::{IndexSnapshot, PartitionSnapshot, SerializedPlan};
use crate::queryplanner::CubeTableLogical;
use crate::CubeError;

pub async fn choose_index(
    p: &LogicalPlan,
    metastore: &dyn PlanIndexStore,
) -> Result<(LogicalPlan, Vec<IndexSnapshot>), DataFusionError> {
    let mut r = ChooseIndex {
        metastore,
        collected_snapshots: Vec::new(),
    };
    let plan = rewrite_plan(p, &None, &mut r).await?;
    Ok((plan, r.collected_snapshots))
}

#[async_trait]
pub trait PlanIndexStore: Send + Sync {
    async fn get_table(
        &self,
        schema_name: String,
        table_name: String,
    ) -> Result<IdRow<Table>, CubeError>;
    async fn get_schema_by_id(&self, schema_id: u64) -> Result<IdRow<Schema>, CubeError>;
    async fn get_default_index(&self, table_id: u64) -> Result<IdRow<Index>, CubeError>;
    async fn get_table_indexes(&self, table_id: u64) -> Result<Vec<IdRow<Index>>, CubeError>;
    async fn get_active_partitions_and_chunks_by_index_id_for_select(
        &self,
        index_id: u64,
    ) -> Result<Vec<(IdRow<Partition>, Vec<IdRow<Chunk>>)>, CubeError>;
}

#[async_trait]
impl<'a> PlanIndexStore for &'a dyn MetaStore {
    async fn get_table(
        &self,
        schema_name: String,
        table_name: String,
    ) -> Result<IdRow<Table>, CubeError> {
        MetaStore::get_table(*self, schema_name, table_name).await
    }

    async fn get_schema_by_id(&self, schema_id: u64) -> Result<IdRow<Schema>, CubeError> {
        MetaStore::get_schema_by_id(*self, schema_id).await
    }

    async fn get_default_index(&self, table_id: u64) -> Result<IdRow<Index>, CubeError> {
        MetaStore::get_default_index(*self, table_id).await
    }

    async fn get_table_indexes(&self, table_id: u64) -> Result<Vec<IdRow<Index>>, CubeError> {
        MetaStore::get_table_indexes(*self, table_id).await
    }

    async fn get_active_partitions_and_chunks_by_index_id_for_select(
        &self,
        index_id: u64,
    ) -> Result<Vec<(IdRow<Partition>, Vec<IdRow<Chunk>>)>, CubeError> {
        MetaStore::get_active_partitions_and_chunks_by_index_id_for_select(*self, index_id).await
    }
}

struct ChooseIndex<'a> {
    metastore: &'a dyn PlanIndexStore,
    collected_snapshots: Vec<IndexSnapshot>,
}

struct SortColumns {
    sort_on: Vec<String>,
    required: bool,
}

#[async_trait]
impl PlanRewriter for ChooseIndex<'_> {
    type Context = Option<SortColumns>;

    async fn rewrite(
        &mut self,
        n: LogicalPlan,
        c: &Self::Context,
    ) -> Result<LogicalPlan, DataFusionError> {
        let p = self
            .choose_table_index(n, c.as_ref().map(|sc| (&sc.sort_on, sc.required)))
            .await?;
        pull_up_cluster_send(p)
    }

    fn enter_node(
        &mut self,
        n: &LogicalPlan,
        _: &Option<SortColumns>,
    ) -> Option<Option<SortColumns>> {
        fn column_name(expr: &Expr) -> Option<String> {
            match expr {
                Expr::Alias(e, _) => column_name(e),
                Expr::Column(col, _) => Some(col.to_string()), // TODO use alias
                _ => None,
            }
        }
        match n {
            LogicalPlan::Aggregate { group_expr, .. } => {
                let sort_on = group_expr.iter().map(column_name).collect::<Vec<_>>();
                if !sort_on.is_empty() && sort_on.iter().all(|c| c.is_some()) {
                    Some(Some(SortColumns {
                        sort_on: sort_on.into_iter().map(|c| c.unwrap()).collect(),
                        required: false,
                    }))
                } else {
                    Some(None)
                }
            }
            _ => None,
        }
    }

    fn enter_join_left(
        &mut self,
        join: &LogicalPlan,
        _: &Option<SortColumns>,
    ) -> Option<Option<SortColumns>> {
        let join_on;
        if let LogicalPlan::Join { on, .. } = join {
            join_on = on;
        } else {
            panic!("expected join node");
        }
        Some(Some(SortColumns {
            sort_on: join_on
                .iter()
                .map(|(l, _)| l.split(".").last().unwrap().to_string())
                .collect(),
            required: true,
        }))
    }

    fn enter_join_right(
        &mut self,
        join: &LogicalPlan,
        _c: &Self::Context,
    ) -> Option<Self::Context> {
        let join_on;
        if let LogicalPlan::Join { on, .. } = join {
            join_on = on;
        } else {
            panic!("expected join node");
        }
        Some(Some(SortColumns {
            sort_on: join_on
                .iter()
                .map(|(_, r)| r.split(".").last().unwrap().to_string())
                .collect(),
            required: true,
        }))
    }
}

fn try_extract_cluster_send(p: &LogicalPlan) -> Option<&ClusterSendNode> {
    if let LogicalPlan::Extension { node } = p {
        return node.as_any().downcast_ref::<ClusterSendNode>();
    }
    return None;
}

impl ChooseIndex<'_> {
    async fn choose_table_index(
        &mut self,
        mut p: LogicalPlan,
        sort_on: Option<(&Vec<String>, bool)>,
    ) -> Result<LogicalPlan, DataFusionError> {
        let meta_store = self.metastore;
        match &mut p {
            LogicalPlan::TableScan {
                table_name,
                projection,
                filters,
                source,
                ..
            } => {
                let name_split = table_name.split(".").collect::<Vec<_>>();
                let table = meta_store
                    .get_table(name_split[0].to_string(), name_split[1].to_string())
                    .await?;
                let schema = meta_store
                    .get_schema_by_id(table.get_row().get_schema_id())
                    .await?;
                let default_index = meta_store.get_default_index(table.get_id()).await?;
                let (index, sort_on) = if let Some(projection_column_indices) = projection {
                    let projection_columns =
                        CubeTable::project_to_table(&table, &projection_column_indices);
                    let indexes = meta_store.get_table_indexes(table.get_id()).await?;
                    if let Some((index, _)) = indexes
                        .into_iter()
                        .filter_map(|i| {
                            if let Some((join_on_columns, _)) = sort_on.as_ref() {
                                let join_columns_in_index = join_on_columns
                                    .iter()
                                    .map(|c| {
                                        i.get_row()
                                            .get_columns()
                                            .iter()
                                            .find(|ic| ic.get_name().as_str() == c.as_str())
                                            .clone()
                                    })
                                    .collect::<Vec<_>>();
                                if join_columns_in_index.iter().any(|c| c.is_none()) {
                                    return None;
                                }
                                let join_columns_indices = CubeTable::project_to_index_positions(
                                    &join_columns_in_index
                                        .into_iter()
                                        .map(|c| c.unwrap().clone())
                                        .collect(),
                                    &i,
                                );
                                if (0..join_columns_indices.len())
                                    .map(|i| Some(i))
                                    .collect::<HashSet<_>>()
                                    != join_columns_indices.into_iter().collect::<HashSet<_>>()
                                {
                                    return None;
                                }
                            }
                            let projected_index_positions =
                                CubeTable::project_to_index_positions(&projection_columns, &i);
                            let score = projected_index_positions
                                .into_iter()
                                .fold_options(0, |a, b| a + b);
                            score.map(|s| (i, s))
                        })
                        .min_by_key(|(_, s)| *s)
                    {
                        (index, sort_on)
                    } else {
                        if let Some((join_on_columns, true)) = sort_on.as_ref() {
                            return Err(DataFusionError::Plan(format!(
                                "Can't find index to join table {} on {}. Consider creating index: CREATE INDEX {}_{} ON {} ({})",
                                name_split.join("."),
                                join_on_columns.join(", "),
                                &name_split[1],
                                join_on_columns.join("_"),
                                name_split.join("."),
                                join_on_columns.join(", ")
                            )));
                        }
                        (default_index, None)
                    }
                } else {
                    if let Some((join_on_columns, _)) = sort_on {
                        return Err(DataFusionError::Plan(format!(
                            "Can't find index to join table {} on {} and projection push down optimization has been disabled. Invalid state.",
                            name_split.join("."),
                            join_on_columns.join(", ")
                        )));
                    }
                    (default_index, None)
                };

                let partitions = meta_store
                    .get_active_partitions_and_chunks_by_index_id_for_select(index.get_id())
                    .await?;

                let partition_filter =
                    PartitionFilter::extract(&partition_filter_schema(&index), filters);
                log::trace!("Extracted partition filter is {:?}", partition_filter);
                let candidate_partitions = partitions.len();
                let mut pruned_partitions = 0;

                let mut partition_snapshots = Vec::new();
                for (partition, chunks) in partitions.into_iter() {
                    let min_row = partition
                        .get_row()
                        .get_min_val()
                        .as_ref()
                        .map(|r| r.values().as_slice());
                    let max_row = partition
                        .get_row()
                        .get_max_val()
                        .as_ref()
                        .map(|r| r.values().as_slice());

                    if !partition_filter.can_match(min_row, max_row) {
                        pruned_partitions += 1;
                        continue;
                    }

                    partition_snapshots.push(PartitionSnapshot { chunks, partition });
                }
                log::trace!(
                    "Pruned {} of {} partitions",
                    pruned_partitions,
                    candidate_partitions
                );

                assert!(source.as_any().is::<CubeTableLogical>());
                let snapshot = IndexSnapshot {
                    index,
                    partitions: partition_snapshots,
                    table_path: TablePath {
                        table,
                        schema: Arc::new(schema),
                    },
                    sort_on: sort_on.map(|(cols, _)| cols.clone()),
                };
                self.collected_snapshots.push(snapshot.clone());
                *source = Arc::new(CubeTable::try_new(
                    snapshot.clone(),
                    // Filled by workers
                    HashMap::new(),
                    HashSet::new(),
                )?);

                return Ok(ClusterSendNode {
                    input: Arc::new(p),
                    snapshots: vec![vec![snapshot]],
                }
                .into_plan());
            }
            _ => return Ok(p),
        }
    }
}

fn partition_filter_schema(index: &IdRow<Index>) -> arrow::datatypes::Schema {
    let schema_fields: Vec<Field>;
    schema_fields = index
        .get_row()
        .columns()
        .iter()
        .map(|c| c.clone().into())
        .take(index.get_row().sort_key_size() as usize)
        .collect();
    arrow::datatypes::Schema::new(schema_fields)
}

#[derive(Debug, Clone)]
pub struct ClusterSendNode {
    pub input: Arc<LogicalPlan>,
    pub snapshots: Vec<Vec<IndexSnapshot>>,
}

impl ClusterSendNode {
    pub fn into_plan(self) -> LogicalPlan {
        LogicalPlan::Extension {
            node: Arc::new(self),
        }
    }
}

impl UserDefinedLogicalNode for ClusterSendNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn inputs(&self) -> Vec<&LogicalPlan> {
        vec![self.input.as_ref()]
    }

    fn schema(&self) -> &DFSchemaRef {
        self.input.schema()
    }

    fn expressions(&self) -> Vec<Expr> {
        vec![]
    }

    fn prevent_predicate_push_down_columns(&self) -> HashSet<String, RandomState> {
        HashSet::new()
    }

    fn fmt_for_explain(&self, f: &mut Formatter<'a>) -> std::fmt::Result {
        write!(f, "ClusterSend")
    }

    fn from_template(
        &self,
        exprs: &Vec<Expr>,
        inputs: &Vec<LogicalPlan>,
    ) -> Arc<dyn UserDefinedLogicalNode + Send + Sync> {
        assert!(exprs.is_empty());
        assert_eq!(inputs.len(), 1);

        Arc::new(ClusterSendNode {
            input: Arc::new(inputs[0].clone()),
            snapshots: self.snapshots.clone(),
        })
    }
}

fn pull_up_cluster_send(mut p: LogicalPlan) -> Result<LogicalPlan, DataFusionError> {
    let snapshots;
    match &mut p {
        // These nodes have no children, return unchanged.
        LogicalPlan::TableScan { .. }
        | LogicalPlan::EmptyRelation { .. }
        | LogicalPlan::CreateExternalTable { .. }
        | LogicalPlan::Explain { .. } => return Ok(p),
        // The ClusterSend itself, return unchanged.
        LogicalPlan::Extension { .. } => return Ok(p),
        // These nodes collect results from multiple partitions, return unchanged.
        LogicalPlan::Aggregate { .. }
        | LogicalPlan::Sort { .. }
        | LogicalPlan::Limit { .. }
        | LogicalPlan::Repartition { .. } => return Ok(p),
        // We can always pull cluster send for these nodes.
        LogicalPlan::Projection { input, .. } | LogicalPlan::Filter { input, .. } => {
            let send;
            if let Some(s) = try_extract_cluster_send(input) {
                send = s;
            } else {
                return Ok(p);
            }
            snapshots = send.snapshots.clone();
            // Code after 'match' will wrap `p` in ClusterSend.
            *input = send.input.clone();
        }
        LogicalPlan::Union { inputs, .. } => {
            let mut union_snapshots = Vec::new();
            for i in inputs {
                let send;
                if let Some(s) = try_extract_cluster_send(i) {
                    send = s;
                } else {
                    return Err(DataFusionError::Plan(
                        "UNION argument not supported".to_string(),
                    ));
                }
                union_snapshots.extend(send.snapshots.concat());
                // Code after 'match' will wrap `p` in ClusterSend.
                *i = send.input.clone();
            }
            snapshots = vec![union_snapshots];
        }
        LogicalPlan::Join { left, right, .. } => {
            let lsend;
            let rsend;
            if let (Some(l), Some(r)) = (
                try_extract_cluster_send(left),
                try_extract_cluster_send(right),
            ) {
                lsend = l;
                rsend = r;
            } else {
                return Err(DataFusionError::Plan(
                    "JOIN argument not supported".to_string(),
                ));
            }
            snapshots = lsend
                .snapshots
                .iter()
                .chain(rsend.snapshots.iter())
                .cloned()
                .collect();
            // Code after 'match' will wrap `p` in ClusterSend.
            *left = lsend.input.clone();
            *right = rsend.input.clone();
        }
    }

    Ok(ClusterSendNode {
        input: Arc::new(p),
        snapshots,
    }
    .into_plan())
}

pub struct ClusterSendPlanner {
    pub cluster: Option<Arc<dyn Cluster>>,
    pub serialized_plan: Arc<SerializedPlan>,
}

impl ExtensionPlanner for ClusterSendPlanner {
    fn plan_extension(
        &self,
        node: &dyn UserDefinedLogicalNode,
        inputs: Vec<Arc<dyn ExecutionPlan>>,
        _state: &ExecutionContextState,
    ) -> Result<Arc<dyn ExecutionPlan>, DataFusionError> {
        assert_eq!(inputs.len(), 1);
        let input = inputs.into_iter().next().unwrap();

        let node = node.as_any().downcast_ref::<ClusterSendNode>().unwrap();
        if node.snapshots.is_empty() {
            return Ok(Arc::new(EmptyExec::new(
                false,
                node.schema().to_schema_ref(),
            )));
        }
        // Note that MergeExecs are added automatically when needed.
        if let Some(c) = self.cluster.as_ref() {
            Ok(Arc::new(ClusterSendExec::new(
                node.schema().clone(),
                c.clone(),
                self.serialized_plan.clone(),
                node.snapshots.clone(),
                input,
            )))
        } else {
            Ok(Arc::new(WorkerExec {
                input,
                schema: node.schema().clone(),
            }))
        }
    }
}

/// Produced on the worker, marks the subplan that the worker must execute. Anything above is the
/// router part of the plan and must be ignored.
#[derive(Debug)]
pub struct WorkerExec {
    pub input: Arc<dyn ExecutionPlan>,
    // TODO: remove and use `self.input.schema()`
    //       This is a hacky workaround for wrong schema of joins after projection pushdown.
    pub schema: DFSchemaRef,
}

#[async_trait]
impl ExecutionPlan for WorkerExec {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn schema(&self) -> DFSchemaRef {
        self.schema.clone()
    }

    fn output_partitioning(&self) -> Partitioning {
        self.input.output_partitioning()
    }

    fn children(&self) -> Vec<Arc<dyn ExecutionPlan>> {
        vec![self.input.clone()]
    }

    fn with_new_children(
        &self,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>, DataFusionError> {
        assert_eq!(children.len(), 1);
        Ok(Arc::new(WorkerExec {
            input: children.into_iter().next().unwrap(),
            schema: self.schema.clone(),
        }))
    }

    fn output_hints(&self) -> OptimizerHints {
        self.input.output_hints()
    }

    async fn execute(
        &self,
        partition: usize,
    ) -> Result<SendableRecordBatchStream, DataFusionError> {
        self.input.execute(partition).await
    }
}

/// Use this to pick the part of the plan that the worker must execute.
pub fn get_worker_plan(p: &Arc<dyn ExecutionPlan>) -> Option<Arc<dyn ExecutionPlan>> {
    if let Some(p) = p.as_any().downcast_ref::<WorkerExec>() {
        return Some(p.input.clone());
    } else {
        let children = p.children();
        // We currently do not split inside joins or leaf nodes.
        if children.len() != 1 {
            return None;
        } else {
            return get_worker_plan(&children[0]);
        }
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use arrow::datatypes::Schema as ArrowSchema;
    use async_trait::async_trait;
    use datafusion::datasource::TableProvider;
    use datafusion::execution::context::ExecutionContext;
    use datafusion::logical_plan::LogicalPlan;
    use datafusion::physical_plan::udaf::AggregateUDF;
    use datafusion::physical_plan::udf::ScalarUDF;
    use datafusion::sql::parser::Statement as DFStatement;
    use datafusion::sql::planner::{ContextProvider, SqlToRel};
    use itertools::Itertools;
    use pretty_assertions::assert_eq;

    use crate::metastore::table::{Table, TablePath};
    use crate::metastore::{Chunk, Column, ColumnType, IdRow, Index, Partition, Schema};
    use crate::queryplanner::planning::{choose_index, PlanIndexStore};
    use crate::queryplanner::{pretty_printers, CubeTableLogical};
    use crate::sql::parser::{CubeStoreParser, Statement};
    use crate::CubeError;

    #[tokio::test]
    pub async fn test_choose_index() {
        let indices = default_indices();
        let plan = initial_plan("SELECT * FROM s.Customers WHERE customer_id = 1", &indices);
        assert_eq!(
            pretty_printers::pp_plan(&plan),
            "Filter\
           \n  Scan s.Customers, source: CubeTableLogical, fields: *"
        );

        let plan = choose_index(&plan, &indices).await.unwrap().0;
        assert_eq!(
            pretty_printers::pp_plan(&plan),
            "ClusterSend, indices: [[0]]\
           \n  Filter\
           \n    Scan s.Customers, source: CubeTable(index: default:0:[]), fields: *"
        );

        // Should prefer a non-default index for joins.
        let plan = initial_plan(
            "SELECT order_id, order_amount, customer_name \
             FROM s.Orders \
             JOIN s.Customers ON order_customer = customer_id",
            &indices,
        );
        let plan = choose_index(&plan, &indices).await.unwrap().0;
        assert_eq!(pretty_printers::pp_plan(&plan), "ClusterSend, indices: [[3], [0]]\
                                  \n  Projection, [order_id, order_amount, customer_name]\
                                  \n    Join on: [order_customer = customer_id]\
                                  \n      Scan s.Orders, source: CubeTable(index: by_customer:3:[]:sort_on[order_customer]), fields: [order_id, order_customer, order_amount]\
                                  \n      Scan s.Customers, source: CubeTable(index: default:0:[]:sort_on[customer_id]), fields: [customer_id, customer_name]");

        let plan = initial_plan(
            "SELECT order_id, customer_name, product_name \
             FROM s.Orders \
             JOIN s.Customers on order_customer = customer_id \
             JOIN s.Products ON order_product = product_id",
            &indices,
        );
        let plan = choose_index(&plan, &indices).await.unwrap().0;
        assert_eq!(pretty_printers::pp_plan(&plan), "ClusterSend, indices: [[3], [0], [5]]\
        \n  Projection, [order_id, customer_name, product_name]\
        \n    Join on: [order_product = product_id]\
        \n      Join on: [order_customer = customer_id]\
        \n        Scan s.Orders, source: CubeTable(index: by_customer:3:[]:sort_on[order_customer]), fields: [order_id, order_customer, order_product]\
        \n        Scan s.Customers, source: CubeTable(index: default:0:[]:sort_on[customer_id]), fields: [customer_id, customer_name]\
        \n      Scan s.Products, source: CubeTable(index: default:5:[]:sort_on[product_id]), fields: *");

        let plan = initial_plan(
            "SELECT c2.customer_name \
             FROM s.Orders \
             JOIN s.Customers c1 on order_customer = c1.customer_id \
             JOIN s.Customers c2 ON order_city = c2.customer_city \
             WHERE c1.customer_name = 'Customer 1'",
            &indices,
        );
        let plan = choose_index(&plan, &indices).await.unwrap().0;
        assert_eq!(pretty_printers::pp_plan(&plan), "ClusterSend, indices: [[3], [0], [1]]\
                                  \n  Projection, [customer_name]\
                                  \n    Join on: [order_city = c2.customer_city]\
                                  \n      Join on: [order_customer = c1.customer_id]\
                                  \n        Scan s.Orders, source: CubeTable(index: by_customer:3:[]:sort_on[order_customer]), fields: [order_customer, order_city]\
                                  \n        Filter\
                                  \n          Scan s.Customers, source: CubeTable(index: default:0:[]:sort_on[customer_id]), fields: [c1.customer_id, c1.customer_name]\
                                  \n      Scan s.Customers, source: CubeTable(index: by_city:1:[]:sort_on[customer_city]), fields: [c2.customer_name, c2.customer_city]");
    }

    /// Most tests in this module use this schema.
    fn default_indices() -> TestIndices {
        const SCHEMA: u64 = 0;
        let mut i = TestIndices::default();

        let customers_cols = int_columns(&[
            "customer_id",
            "customer_name",
            "customer_city",
            "customer_registered_date",
        ]);
        let customers = i.add_table(Table::new(
            "Customers".to_string(),
            SCHEMA,
            customers_cols.clone(),
            None,
            None,
        ));
        i.indices.push(
            Index::try_new(
                "by_city".to_string(),
                customers,
                put_first("customer_city", &customers_cols),
                1,
            )
            .unwrap(),
        );

        let orders_cols = int_columns(&[
            "order_id",
            "order_customer",
            "order_product",
            "order_amount",
            "order_city",
        ]);
        let orders = i.add_table(Table::new(
            "Orders".to_string(),
            SCHEMA,
            orders_cols.clone(),
            None,
            None,
        ));
        i.indices.push(
            Index::try_new(
                "by_customer".to_string(),
                orders,
                put_first("order_customer", &orders_cols),
                2,
            )
            .unwrap(),
        );
        i.indices.push(
            Index::try_new(
                "by_city".to_string(),
                customers,
                put_first("order_city", &orders_cols),
                2,
            )
            .unwrap(),
        );

        i.add_table(Table::new(
            "Products".to_string(),
            SCHEMA,
            int_columns(&["product_id", "product_name"]),
            None,
            None,
        ));

        i
    }

    fn put_first(c: &str, cols: &[Column]) -> Vec<Column> {
        let mut cols = cols.iter().cloned().collect_vec();
        let pos = cols.iter().position(|col| col.get_name() == c).unwrap();
        cols.swap(0, pos);
        cols
    }

    fn int_columns(names: &[&str]) -> Vec<Column> {
        names
            .iter()
            .enumerate()
            .map(|(i, n)| Column::new(n.to_string(), ColumnType::Int, i))
            .collect()
    }

    fn initial_plan(s: &str, i: &TestIndices) -> LogicalPlan {
        let statement;
        if let Statement::Statement(s) = CubeStoreParser::new(s).unwrap().parse_statement().unwrap()
        {
            statement = s;
        } else {
            panic!("not a statement")
        }
        let plan = SqlToRel::new(i)
            .statement_to_plan(&DFStatement::Statement(statement))
            .unwrap();
        ExecutionContext::new().optimize(&plan).unwrap()
    }

    #[derive(Debug, Default)]
    pub struct TestIndices {
        tables: Vec<Table>,
        indices: Vec<Index>,
        partitions: Vec<Partition>,
    }

    impl TestIndices {
        pub fn add_table(&mut self, t: Table) -> u64 {
            assert_eq!(t.get_schema_id(), 0);
            let table_id = self.tables.len() as u64;
            self.indices.push(
                Index::try_new(
                    "default".to_string(),
                    table_id,
                    t.get_columns().clone(),
                    t.get_columns().len() as u64,
                )
                .unwrap(),
            );
            self.tables.push(t);
            table_id
        }

        pub fn schema(&self) -> IdRow<Schema> {
            IdRow::new(0, Schema::new("s".to_string()))
        }
    }

    impl ContextProvider for TestIndices {
        fn get_table_provider(&self, name: &str) -> Option<Arc<dyn TableProvider + Send + Sync>> {
            let name = name.strip_prefix("s.")?;
            self.tables
                .iter()
                .find_position(|t| t.get_table_name() == name)
                .map(|(id, t)| -> Arc<dyn TableProvider + Send + Sync> {
                    let schema = Arc::new(ArrowSchema::new(
                        t.get_columns()
                            .iter()
                            .map(|c| c.clone().into())
                            .collect::<Vec<_>>(),
                    ));
                    Arc::new(CubeTableLogical {
                        table: TablePath {
                            table: IdRow::new(id as u64, t.clone()),
                            schema: Arc::new(self.schema()),
                        },
                        schema,
                    })
                })
        }

        fn get_function_meta(&self, _name: &str) -> Option<Arc<ScalarUDF>> {
            // Note that this is missing HLL functions.
            None
        }

        fn get_aggregate_meta(&self, _name: &str) -> Option<Arc<AggregateUDF>> {
            // Note that this is missing HLL functions.
            None
        }
    }

    #[async_trait]
    impl PlanIndexStore for TestIndices {
        async fn get_table(
            &self,
            schema_name: String,
            table_name: String,
        ) -> Result<IdRow<Table>, CubeError> {
            if schema_name != "s" {
                return Err(CubeError::internal(
                    "only 's' schema defined in tests".to_string(),
                ));
            }
            let (pos, table) = self
                .tables
                .iter()
                .find_position(|t| t.get_table_name() == &table_name)
                .ok_or_else(|| CubeError::internal(format!("table {} not found", table_name)))?;
            Ok(IdRow::new(pos as u64, table.clone()))
        }

        async fn get_schema_by_id(&self, schema_id: u64) -> Result<IdRow<Schema>, CubeError> {
            if schema_id != 0 {
                return Err(CubeError::internal(
                    "only 's' schema with id = 0 defined in tests".to_string(),
                ));
            }
            return Ok(self.schema());
        }

        async fn get_default_index(&self, table_id: u64) -> Result<IdRow<Index>, CubeError> {
            let (pos, index) = self
                .indices
                .iter()
                .find_position(|i| i.table_id() == table_id)
                .ok_or_else(|| {
                    CubeError::internal(format!("index for table {} not found", table_id))
                })?;
            Ok(IdRow::new(pos as u64, index.clone()))
        }

        async fn get_table_indexes(&self, table_id: u64) -> Result<Vec<IdRow<Index>>, CubeError> {
            Ok(self
                .indices
                .iter()
                .enumerate()
                .filter(|(_, i)| i.table_id() == table_id)
                .map(|(pos, index)| IdRow::new(pos as u64, index.clone()))
                .collect())
        }

        async fn get_active_partitions_and_chunks_by_index_id_for_select(
            &self,
            index_id: u64,
        ) -> Result<Vec<(IdRow<Partition>, Vec<IdRow<Chunk>>)>, CubeError> {
            Ok(self
                .partitions
                .iter()
                .enumerate()
                .filter(|(_, p)| p.get_index_id() == index_id)
                .map(|(id, p)| (IdRow::new(id as u64, p.clone()), vec![]))
                .collect())
        }
    }
}
