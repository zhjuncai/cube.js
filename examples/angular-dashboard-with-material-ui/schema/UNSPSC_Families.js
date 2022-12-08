cube(`UNSPSC_Families`, {
    sql: `SELECT * FROM "SCHEMA_MANAGER"."FACT_PO_ITEM_COMMUNITY" where UNSPSC_FAMILY <> UNSPSC_SEGMENT`,
    
    preAggregations: {
      // Pre-Aggregations definitions go here
      // Learn more here: https://cube.dev/docs/caching/pre-aggregations/getting-started  
    },
    
    joins: {
      Commodities: {
        sql: `${CUBE}.UNSPSC_FAMILY = ${Commodities.code}`,
        relationship: `hasOne`
      }  
    },
    
    measures: {
      count: {
        type: `count`,
        drillMembers: [familyId]
      }
    },
    
    dimensions: {
      familyId: {
        sql: `${CUBE}."UNSPSC_FAMILY"`,
        type: `string`,
        primaryKey: true,
        shown: true
      },
      
      segmentId: {
        sql: `${CUBE}."UNSPSC_SEGMENT"`,
        type: `string`
      },

      description: {
        sql: `${Commodities.name}`,
        type: `string`
      }    
    },
    
    dataSource: `default`
  });