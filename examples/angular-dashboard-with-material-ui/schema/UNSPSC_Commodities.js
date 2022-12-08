cube(`UNSPSC_Commodities`, {
    sql: `SELECT * FROM "SCHEMA_MANAGER"."FACT_PO_ITEM_COMMUNITY" where UNSPSC_COMMODITY <> UNSPSC_CLASS`,
    
    preAggregations: {
      // Pre-Aggregations definitions go here
      // Learn more here: https://cube.dev/docs/caching/pre-aggregations/getting-started  
    },
    
    joins: {
      Commodities: {
        sql: `${CUBE}.UNSPSC_COMMODITY = ${Commodities.code}`,
        relationship: `hasOne`
      }  
    },
    
    measures: {
      count: {
        type: `count`,
        drillMembers: [commodityId]
      }
    },
    
    dimensions: {
      commodityId: {
        sql: `${CUBE}."UNSPSC_COMMODITY"`,
        type: `string`,
        primaryKey: true,
        shown: true
      },
      
      segmentId: {
        sql: `${CUBE}."UNSPSC_SEGMENT"`,
        type: `string`
      },
      
      familyId: {
        sql: `${CUBE}."UNSPSC_FAMILY"`,
        type: `string`
      },

      classId: {
        sql: `${CUBE}."UNSPSC_CLASS"`,
        type: `string`
      },    

      description: {
        sql: `${Commodities.name}`,
        type: `string`
      }    
    },
    
    dataSource: `default`
  });