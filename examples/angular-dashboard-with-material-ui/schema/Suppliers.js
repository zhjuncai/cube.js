cube(`Suppliers`, {
    extends: Orgs,
    sql: `SELECT * FROM ${Orgs.sql()} WHERE IS_TXN_SUP = true OR IS_SRC_SUP = true OR IS_MKT_SUP = true`,
});