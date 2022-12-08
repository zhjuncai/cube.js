cube(`Relationships`, {
    sql: `SELECT * FROM "SCHEMA_MANAGER"."DIM_RELATIONSHIP"`,
    
    preAggregations: {
      // Pre-Aggregations definitions go here
      // Learn more here: https://cube.dev/docs/caching/pre-aggregations/getting-started  
    },
    
    joins: {
      Buyers: {
        relationship: `belongsTo`,
        sql: `${CUBE.party0} = ${Buyers.id}`
      },
      Suppliers: {
        relationship: `belongsTo`,
        sql: `${CUBE.party1} = ${Suppliers.id}`
      }
    },
    
    measures: {
      count: {
        type: `count`,
        drillMembers: [id, statusName, typeName, created, party0, party1]
      }
    },
    
    dimensions: {
      id: {
        sql: `${CUBE}."ID"`,
        type: `number`,
        primaryKey: true
      },

      party0: {
        sql: `${CUBE}."PARTY0"`,
        type: `number`
      },

      party1: {
        sql: `${CUBE}."PARTY1"`,
        type: `number`
      },
      
      statusName: {
        sql: `${CUBE}."STATUS_NAME"`,
        type: `string`
      },
      
      typeName: {
        sql: `${CUBE}."TYPE_NAME"`,
        type: `string`
      },
      
      created: {
        sql: `${CUBE}."CREATED"`,
        type: `time`
      },
      
      lastModified: {
        sql: `${CUBE}."LAST_MODIFIED"`,
        type: `time`
      }
    },
    
    dataSource: `default`
  });