cube(`Relationship`, {
    sql: `SELECT * FROM "SCHEMA_MANAGER"."DIM_RELATIONSHIP"`,
    
    preAggregations: {
      // Pre-Aggregations definitions go here
      // Learn more here: https://cube.dev/docs/caching/pre-aggregations/getting-started
    },
    
    joins: {
      MTimeDimension:{
        relationship:`belongsTo`,
        sql:`TO_DATE(${CUBE.lastModified})=${MTimeDimension.dateSql}`
      }
    },
    
    measures: {
      count: {
        type: `count`,
        drillMembers: [id, statusName, typeName, created]
      },
      count_ytd:{
        type: `count`,
        filters: [
          {sql:`TO_DATE(${CUBE.lastModified}) >= TO_DATE(CONCAT(YEAR(CURRENT_DATE),'-01-01'))`},
          {sql:`TO_DATE(${CUBE.lastModified}) <= TO_DATE(CURRENT_DATE)`}
          ]
      },
      count_ytd_lastyear:{
        type: `count`,
        filters: [
          {sql:`TO_DATE(${CUBE.lastModified}) >= TO_DATE(CONCAT(YEAR(CURRENT_DATE)-1,'-01-01'))`},
          {sql:`TO_DATE(${CUBE.lastModified}) <= ADD_YEARS(CURRENT_DATE, -1)`}
          ]
      },
        count_ytd_thisyear_vs_lastyear_rate:{
        type: `number`,
        sql: `(${count_ytd} - ${count_ytd_lastyear}) / ${count_ytd_lastyear} * 100`,
        format: `percent`
      } ,
    },
    
    dimensions: {
      id: {
        sql: `${CUBE}."ID"`,
        type: `number`,
        primaryKey: true
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