cube(`POChangeRate`, {
    sql: `SELECT * FROM "SCHEMA_MANAGER"."PO_CHANGE_RATE"`,
    
    preAggregations: {
      poCount1: {
        measures: [poCount],
        dimensions: [buyerAnid]
      }
    },
    
    joins: {
      
    },
    
    measures: {
      count: {
        type: `count`,
        drillMembers: [buyerAnid, createdByUser, id, supplierAnid, timeCreated, timeUpdated]
      },
      
      poCount: {
        sql: `${CUBE}."PO_COUNT"`,
        type: `sum`
      },
  
      rollingCountMonth: {
        sql: `PO_COUNT`,
        type: `sum`,
        rollingWindow: {
          trailing: `1 month`,
          offset: `start`
        },
      },
      diffVsLastMonth: {
        type: `number`,
        sql: `${CUBE.poCount} - ${CUBE.rollingCountMonth}`
      },
  
      diffVsLastMonthPercentage: {
        sql: `100 * (${CUBE.poCount} - ${CUBE.rollingCountMonth}) / ${CUBE.poCount}`,
        type: `number`,
        format: `percent`
      },
  
      deleteCount: {
        sql: `ID`,
        type: `count`,
        filters: [
          {
            sql: `${CUBE}.OPERATION_TYPE = 'delete'`
          }
        ]
      },
      updateCount: {
        sql: `ID`,
        type: `count`,
        filters: [
          {
            sql: `${CUBE}.OPERATION_TYPE = 'update'`
          }
        ]
      },
      newCount: {
        sql: `ID`,
        type: `count`,
        filters: [
          {
            sql: `${CUBE}.OPERATION_TYPE = 'new'`
          }
        ]
      },
  
      newRate: {
        sql: `(((${newCount}) * 100) / ${count})`,
        type: `number`,
        format: `percent`
      },
  
      updateRate: {
        sql: `(((${updateCount}) * 100) / ${count})`,
        type: `number`,
        format: `percent`
      },
  
      deleteRate: {
        sql: `(((${deleteCount}) * 100) / ${count})`,
        type: `number`,
        format: `percent`
      },
  
      changeRate: {
        sql: `(((${deleteCount} + ${updateCount}) * 100) / ${newCount})`,
        type: `number`,
        format: `percent`
      }
    },
    
    dimensions: {
      buyerAnid: {
        sql: `${CUBE}."BUYER_ANID"`,
        type: `string`
      },
      
      createdByUser: {
        sql: `${CUBE}."CREATED_BY_USER"`,
        type: `string`
      },
      
      id: {
        sql: `${CUBE}."ID"`,
        type: `string`,
        primaryKey: true
      },
      
      modifiedByUser: {
        sql: `${CUBE}."MODIFIED_BY_USER"`,
        type: `string`
      },
      
      operationType: {
        sql: `${CUBE}."OPERATION_TYPE"`,
        type: `string`
      },
      
      quarterExtraction: {
        sql: `${CUBE}."QUARTER_EXTRACTION"`,
        type: `string`
      },
      
      supplierAnid: {
        sql: `${CUBE}."SUPPLIER_ANID"`,
        type: `string`
      },
      
      timeCreated: {
        sql: `${CUBE}."TIME_CREATED"`,
        type: `time`
      },
      
      timeUpdated: {
        sql: `${CUBE}."TIME_UPDATED"`,
        type: `time`
      },
      
      timeExtraction: {
        sql: `${CUBE}."TIME_EXTRACTION"`,
        type: `time`
      }
    },
  
    refreshKey: {
      every: `5 second`,
    },
    
    dataSource: `hana`
  });
  