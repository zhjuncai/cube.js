cube(`Orgs`, {
    sql: `SELECT * FROM "SCHEMA_MANAGER"."DIM_ORG"`,
    measures: {
      count: {
        type: `count`,
        drillMembers: [anid, country, id, name, dbcreated, dbupdated]
      }
    },
    dimensions: {
      accountType: {
        sql: `${CUBE}."ACCOUNT_TYPE"`,
        type: `string`
      },
      
      anid: {
        sql: `${CUBE}."ANID"`,
        type: `string`
      },
      
      businessSize: {
        sql: `${CUBE}."BUSINESS_SIZE"`,
        type: `string`
      },
      
      community: {
        sql: `${CUBE}."COMMUNITY"`,
        type: `string`
      },
      
      country: {
        sql: `${CUBE}."COUNTRY"`,
        type: `string`
      },
      
      currencyCode: {
        sql: `${CUBE}."CURRENCY_CODE"`,
        type: `string`
      },
      
      id: {
        sql: `${CUBE}."ID"`,
        type: `number`,
        primaryKey: true
      },
      
      language: {
        sql: `${CUBE}."LANGUAGE"`,
        type: `string`
      },
      
      name: {
        sql: `${CUBE}."NAME"`,
        type: `string`
      },
      
      status: {
        sql: `${CUBE}."STATUS"`,
        type: `string`
      },
      
      typeOfOrganization: {
        sql: `${CUBE}."TYPE_OF_ORGANIZATION"`,
        type: `string`
      },
      
      dbcreated: {
        sql: `${CUBE}."DBCREATED"`,
        type: `time`
      },
      
      dbupdated: {
        sql: `${CUBE}."DBUPDATED"`,
        type: `time`
      }
    },
    
    dataSource: `default`
  });