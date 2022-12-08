cube(`MTimeDimension`, {
    sql: `SELECT * FROM "SCHEMA_MANAGER"."M_TIME_DIMENSION"`,
    
    preAggregations: {
      // Pre-Aggregations definitions go here
      // Learn more here: https://cube.dev/docs/caching/pre-aggregations/getting-started  
    },
    
    joins: {
      
    },
    
    measures: {
      count: {
        type: `count`,
        drillMembers: [datetimeSap, dateSap, datetimestamp, dateSql]
      }
    },
    
    dimensions: {
      calmonth: {
        sql: `${CUBE}."CALMONTH"`,
        type: `string`
      },
      
      calquarter: {
        sql: `${CUBE}."CALQUARTER"`,
        type: `string`
      },
      
      calweek: {
        sql: `${CUBE}."CALWEEK"`,
        type: `string`
      },
      
      datetimeSap: {
        sql: `${CUBE}."DATETIME_SAP"`,
        type: `string`
      },
      
      dateSap: {
        sql: `${CUBE}."DATE_SAP"`,
        type: `string`
      },
      
      day: {
        sql: `${CUBE}."DAY"`,
        type: `string`
      },
      
      dayOfWeek: {
        sql: `${CUBE}."DAY_OF_WEEK"`,
        type: `string`
      },
      
      hour: {
        sql: `${CUBE}."HOUR"`,
        type: `string`
      },
      
      minute: {
        sql: `${CUBE}."MINUTE"`,
        type: `string`
      },
      
      month: {
        sql: `${CUBE}."MONTH"`,
        type: `string`
      },
      
      quarter: {
        sql: `${CUBE}."QUARTER"`,
        type: `string`
      },
      
      second: {
        sql: `${CUBE}."SECOND"`,
        type: `string`
      },
      
      week: {
        sql: `${CUBE}."WEEK"`,
        type: `string`
      },
      
      weekYear: {
        sql: `${CUBE}."WEEK_YEAR"`,
        type: `string`
      },
      
      year: {
        sql: `${CUBE}."YEAR"`,
        type: `string`
      },
      
      datetimestamp: {
        sql: `${CUBE}."DATETIMESTAMP"`,
        type: `time`,
        primaryKey: true
      },
      
      dateSql: {
        sql: `${CUBE}."DATE_SQL"`,
        type: `time`
      }
    },
    
    dataSource: `default`
  });