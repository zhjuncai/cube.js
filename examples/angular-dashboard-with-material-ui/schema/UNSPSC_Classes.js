cube(`UNSPSC_Classes`, {
    sql: `SELECT * FROM "SCHEMA_MANAGER"."FACT_PO_ITEM_COMMUNITY" where UNSPSC_CLASS <> UNSPSC_FAMILY`,
    
    preAggregations: {
      // Pre-Aggregations definitions go here
      // Learn more here: https://cube.dev/docs/caching/pre-aggregations/getting-started  
    },
    
    joins: {
      Commodities: {
        sql: `${CUBE}.UNSPSC_CLASS = ${Commodities.code}`,
        relationship: `hasOne`
      }  
    },
    
    measures: {
      count: {
        type: `count`,
        drillMembers: [classId]
      }
    },
    
    dimensions: {
      classId: {
        sql: `${CUBE}."UNSPSC_CLASS"`,
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

      description: {
        sql: `${Commodities.name}`,
        type: `string`
      }    
    },
    
    dataSource: `default`
  });