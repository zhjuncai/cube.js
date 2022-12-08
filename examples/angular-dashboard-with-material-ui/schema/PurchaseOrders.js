cube(`PurchaseOrders`, {
    sql: `SELECT * FROM "SCHEMA_MANAGER"."FACT_TXN_SUPPLIER_PO_HDR"`,
    preAggregations: {

    },
    joins: {
      Buyers: {
        relationship: `belongsTo`,
        sql: `${CUBE.buyerOrg} = ${Buyers.id}`
      },
      Suppliers: {
        relationship: `belongsTo`,
        sql: `${CUBE.supplierOrg} = ${Suppliers.id}`
      },
      MTimeDimension: {
        relationship: `belongsTo`,
        sql: `${CUBE.formatDate} = ${MTimeDimension.dateSql} and ${CUBE.updatedAt} >= ADD_YEARS (TO_DATE (CURRENT_DATE),- 3)`,
      },
      Catelog: {
        relationship: `belongsTo`,
        sql: `${CUBE.dashboardStatusCode} = ${Catelog.id}`,
      }
    },
    measures: {
      count: {
        type: `count`,
        drillMembers: [id, shipToCountry, createdAt, createdDate, updatedAt, documentDate]
      },
      amountUsd: {
        type: `sum`,
        sql: `AMOUNT_USD`,
        drillMembers: [shipToCountry, dashboardStatusCode, documentDate]
      },
      amountOriginal: {
        type: `sum`,
        sql: `AMOUNT_ORIGINAL`,
        drillMembers: [shipToCountry, dashboardStatusCode, documentDate]
      },
      amount_usd_ytd: {
        type: `sum`,
        sql: `${CUBE}."AMOUNT_USD"`,
        filters: [
          { sql: `TO_DATE(${CUBE.documentDate}) >= TO_DATE(CONCAT(YEAR(CURRENT_DATE),'-01-01'))` },
          { sql: `TO_DATE(${CUBE.documentDate}) <= TO_DATE(CURRENT_DATE)` }
        ]
      },
      amount_usd_ytd_lastyear: {
        type: `sum`,
        sql: `${CUBE}."AMOUNT_USD"`,
        filters: [
          { sql: `TO_DATE(${CUBE.documentDate}) >= TO_DATE(CONCAT(YEAR(CURRENT_DATE)-1,'-01-01'))` },
          { sql: `TO_DATE(${CUBE.documentDate}) <= ADD_YEARS(CURRENT_DATE, -1)` }
        ]
      },
      amount_usd_ytd_thisyear_vs_lastyear_rate: {
        type: `number`,
        sql: `(${amount_usd_ytd} - ${amount_usd_ytd_lastyear}) / ${amount_usd_ytd_lastyear} * 100`,
        format: `percent`
      },
      count_ytd: {
        type: `count`,

        filters: [
          { sql: `TO_DATE(${CUBE.documentDate}) >= TO_DATE(CONCAT(YEAR(CURRENT_DATE),'-01-01'))` },
          { sql: `TO_DATE(${CUBE.documentDate}) <= TO_DATE(CURRENT_DATE)` }
        ]
      },
      count_ytd_lastyear: {
        type: `count`,

        filters: [
          { sql: `TO_DATE(${CUBE.documentDate}) >= TO_DATE(CONCAT(YEAR(CURRENT_DATE)-1,'-01-01'))` },
          { sql: `TO_DATE(${CUBE.documentDate}) <= ADD_YEARS(CURRENT_DATE, -1)` }
        ]
      },
      count_ytd_thisyear_vs_lastyear_rate: {
        type: `number`,
        sql: `(${count_ytd} - ${count_ytd_lastyear}) / ${count_ytd_lastyear} * 100`,
        format: `percent`
      }
    },
    dimensions: {
      id: {
        sql: `${CUBE}."ID"`,
        type: `number`,
        primaryKey: true
      },
      supplierOrg: {
        sql: `${CUBE}."SUPPLIER_ORG"`,
        type: `string`
      },
      buyerOrg: {
        sql: `${CUBE}."BUYER_ORG"`,
        type: `string`
      },
      companyCode: {
        sql: `${CUBE}."COMPANY_CODE"`,
        type: `string`
      },
      companyCodeDesc: {
        sql: `${CUBE}."COMPANY_CODE_DESC"`,
        type: `string`
      },
      currencyOriginal: {
        sql: `${CUBE}."CURRENCY_ORIGINAL"`,
        type: `string`
      },
      dashboardStatusCode: {
        sql: `${CUBE}."DASHBOARD_STATUS_CODE"`,
        type: `string`
      },
      purchasingGroupCode: {
        sql: `${CUBE}."PURCHASING_GROUP_CODE"`,
        type: `string`
      },
      purchasingGroupDesc: {
        sql: `${CUBE}."PURCHASING_GROUP_DESC"`,
        type: `string`
      },
      purchasingOrgCode: {
        sql: `${CUBE}."PURCHASING_ORG_CODE"`,
        type: `string`
      },
      purchasingOrgDesc: {
        sql: `${CUBE}."PURCHASING_ORG_DESC"`,
        type: `string`
      },
      shipToCountry: {
        sql: `${CUBE}."SHIP_TO_COUNTRY"`,
        type: `string`
      },
      createdAt: {
        sql: `${CUBE}."CREATED_AT"`,
        type: `time`
      },
      createdDate: {
        sql: `${CUBE}."CREATED_DATE"`,
        type: `time`
      },
      updatedAt: {
        sql: `${CUBE}."UPDATED_AT"`,
        type: `time`
      },
      documentDate: {
        sql: `${CUBE}."DOCUMENT_DATE"`,
        type: `time`
      },
      formatDate: {
        sql: `TO_DATE(${CUBE.documentDate})`,
        type: `time`
      }
    },
    dataSource: `default`
  });