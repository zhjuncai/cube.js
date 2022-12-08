cube(`Catelog`, {
    sql: `SELECT * FROM "SCHEMA_MANAGER"."DIM_CATELOG"`,
    
    preAggregations: {
      // Pre-Aggregations definitions go here
      // Learn more here: https://cube.dev/docs/caching/pre-aggregations/getting-started  
    },
    
    joins: {
      
    },
    
    measures: {
      count: {
        type: `count`,
        drillMembers: [id]
      }
    },
    
    dimensions: {
      catelog: {
        sql: `${CUBE}."CATELOG"`,
        type: `string`
      },
      
      id: {
        sql: `${CUBE}."ID"`,
        type: `string`,
        primaryKey: true
      }
    },
    
    dataSource: `default`
  });