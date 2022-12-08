cube(`PoItemCommunities`, {
    sql: `SELECT * FROM "SCHEMA_MANAGER"."FACT_PO_ITEM_COMMUNITY"`,
    
    preAggregations: {
      // Pre-Aggregations definitions go here
      // Learn more here: https://cube.dev/docs/caching/pre-aggregations/getting-started  
    },
    
    joins: {
      
    },
    
    measures: {
      count: {
        type: `count`,
        drillMembers: [id, shipToCountry, created, lastUpdated, documentDate, requestedDeliveryDate]
      },
      
      averageUnitPrice: {
        sql: `${CUBE}."AVERAGE_UNIT_PRICE"`,
        type: `sum`
      },
      
      buyerCount: {
        sql: `${CUBE}."BUYER_COUNT"`,
        type: `sum`
      },
      
      orderAmount: {
        sql: `${CUBE}."ORDER_AMOUNT"`,
        type: `sum`
      },
      
      poCount: {
        sql: `${CUBE}."PO_COUNT"`,
        type: `sum`
      },
      
      poItemCount: {
        sql: `${CUBE}."PO_ITEM_COUNT"`,
        type: `sum`
      },
      
      supplierCount: {
        sql: `${CUBE}."SUPPLIER_COUNT"`,
        type: `sum`
      },

      maxDeliveryDate: {
        sql: `${CUBE}."REQUESTED_DELIVERY_DATE"`,
        type: `max`
      },

      minDeliveryDate: {
        sql: `${CUBE}."REQUESTED_DELIVERY_DATE"`,
        type: `min`
      },

      orderUnits: {
        sql: `${CUBE}."ORDER_UNITS"`,
        type: `sum`
      },

      orderAmountLastMonth: {
        sql: `${CUBE}."ORDER_AMOUNT"`,
        type: `sum`,
        rollingWindow: {
          trailing: `1 month`,
          offset: `start`
        },
      },

      orderAmountDiffLastMonth: {
        type: `number`,
        sql: `${CUBE.orderAmount} - ${CUBE.orderAmountLastMonth}`
      },

      orderAmountDiffPercentageLastMonth: {
        sql: `100 * (${CUBE.orderAmount} - ${CUBE.orderAmountLastMonth}) / ${CUBE.orderAmountLastMonth}`,
        type: `number`,
        format: `percent`
      },
  
      orderAmountLastYear: {
        sql: `${CUBE}."ORDER_AMOUNT"`,
        type: `sum`,
        rollingWindow: {
          trailing: `1 year`,
          offset: `start`
        },
      },

      orderAmountDiffLastYear: {
        type: `number`,
        sql: `${CUBE.orderAmount} - ${CUBE.orderAmountLastYear}`
      },

      orderAmountDiffPercentageLastYear: {
        sql: `100 * (${CUBE.orderAmount} - ${CUBE.orderAmountLastYear}) / ${CUBE.orderAmountLastYear}`,
        type: `number`,
        format: `percent`
      },

      orderAmountLastQuarter: {
        sql: `${CUBE}."ORDER_AMOUNT"`,
        type: `sum`,
        rollingWindow: {
          trailing: `3 month`,
          offset: `start`
        },
      },

      orderAmountDiffLastQuarter: {
        type: `number`,
        sql: `${CUBE.orderAmount} - ${CUBE.orderAmountLastQuarter}`
      },

      orderAmountDiffPercentageLastQuarter: {
        sql: `100 * (${CUBE.orderAmount} - ${CUBE.orderAmountLastQuarter}) / ${CUBE.orderAmountLastQuarter}`,
        type: `number`,
        format: `percent`
      },

    },
    
    dimensions: {
      id: {
        sql: `${CUBE}."ID"`,
        type: `number`,
        primaryKey: true
      },
      
      shipToCountry: {
        sql: `${CUBE}."SHIP_TO_COUNTRY"`,
        type: `string`
      },
      
      unitOfMeasure: {
        sql: `${CUBE}."UNIT_OF_MEASURE"`,
        type: `string`
      },
      
      created: {
        sql: `${CUBE}."CREATED"`,
        type: `time`
      },
      
      lastUpdated: {
        sql: `${CUBE}."LAST_UPDATED"`,
        type: `time`
      },
      
      documentDate: {
        sql: `${CUBE}."DOCUMENT_DATE"`,
        type: `time`
      },
      
      requestedDeliveryDate: {
        sql: `${CUBE}."REQUESTED_DELIVERY_DATE"`,
        type: `time`
      },

      segment: {
        sql: `${CUBE}."UNSPSC_SEGMENT"`,
        type: `string`
      },

      family: {
        sql: `${CUBE}."UNSPSC_FAMILY"`,
        type: `string`
      },

      class: {
        sql: `${CUBE}."UNSPSC_CLASS"`,
        type: `string`
      },

      commodity: {
        sql: `${CUBE}."UNSPSC_COMMODITY"`,
        type: `string`
      }
    },
    
    dataSource: `default`
  });