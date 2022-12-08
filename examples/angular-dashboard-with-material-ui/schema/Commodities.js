cube(`Commodities`, {
    sql: `SELECT * FROM "SCHEMA_MANAGER"."DIM_COMMODITY"`,
    
    preAggregations: {
      // Pre-Aggregations definitions go here
      // Learn more here: https://cube.dev/docs/caching/pre-aggregations/getting-started  
    },
    
    joins: {
      
    },
    
    measures: {
      count: {
        type: `count`,
        drillMembers: [id, levelName, name]
      }
    },
    
    dimensions: {
      id: {
        sql: `${CUBE}."ID"`,
        type: `number`
      },
      
      levelName: {
        sql: `${CUBE}."LEVEL_NAME"`,
        type: `string`
      },
      
      name: {
        sql: `${CUBE}."NAME"`,
        type: `string`
      },

      code: {
        sql: `${CUBE}."CODE"`,
        type: `string`,
        primaryKey: true
      }    
    },
    
    dataSource: `default`
  });