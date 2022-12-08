cube(`CountryRegions`, {
    sql: `SELECT * FROM "SCHEMA_MANAGER"."DIM_COUNTRY_REGION"`,
    
    preAggregations: {
      // Pre-Aggregations definitions go here
      // Learn more here: https://cube.dev/docs/caching/pre-aggregations/getting-started  
    },
    
    joins: {
      
    },
    
    measures: {
      count: {
        type: `count`,
        drillMembers: [countryname, isocountryid, regionName]
      }
    },
    
    dimensions: {
      ansupported: {
        sql: `${CUBE}."ANSUPPORTED"`,
        type: `string`
      },
      
      countryname: {
        sql: `${CUBE}."COUNTRYNAME"`,
        type: `string`
      },
      
      currency: {
        sql: `${CUBE}."CURRENCY"`,
        type: `string`
      },
      
      isoa2: {
        sql: `${CUBE}."ISOA2"`,
        type: `string`
      },
      
      isoa3: {
        sql: `${CUBE}."ISOA3"`,
        type: `string`
      },
      
      isocountryid: {
        sql: `${CUBE}."ISOCOUNTRYID"`,
        type: `string`
      },
      
      phone: {
        sql: `${CUBE}."PHONE"`,
        type: `string`
      },
      
      regionName: {
        sql: `${CUBE}."REGION_NAME"`,
        type: `string`
      },
      
      sap365supported: {
        sql: `${CUBE}."SAP365SUPPORTED"`,
        type: `string`
      }
    },
    
    dataSource: `default`
  });