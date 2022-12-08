cube(`Buyers`, {
    extends: Orgs,
    sql: `SELECT * FROM ${Orgs.sql()} WHERE IS_TXN_BUY = true OR IS_SRC_BUY = true OR IS_MKT_BUY = true`,
});