cube(`UNSPSC_Segments`, {
    sql: `SELECT * FROM "SCHEMA_MANAGER"."FACT_PO_ITEM_COMMUNITY"`,
    
    preAggregations: {
      // Pre-Aggregations definitions go here
      // Learn more here: https://cube.dev/docs/caching/pre-aggregations/getting-started  
    },
    
    joins: {
      Commodities: {
        sql: `${CUBE}.UNSPSC_SEGMENT = ${Commodities.code}`,
        relationship: `hasOne`
      }  
    },
    
    measures: {
      count: {
        type: `count`,
        drillMembers: [segmentId]
      }
    },
    
    dimensions: {
      segmentId: {
        sql: `${CUBE}."UNSPSC_SEGMENT"`,
        type: `string`,
        primaryKey: true,
        shown: true
      },

      description: {
        sql: `${Commodities.name}`,
        type: `string`
      }    
    },
    
    dataSource: `default`
  });