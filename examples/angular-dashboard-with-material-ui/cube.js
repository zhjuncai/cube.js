// Server options go here: https://cube.dev/docs/config#options-reference
module.exports = {
    driverFactory: ({ dataSource }) => {
        return {
            type: 'hana',
            serverNode: '4b3b5138-3c21-404d-89f0-8f942db8db09.hana.prod-us20.hanacloud.ondemand.com:443',
            uid: 'SCHEMA_MANAGER',
            pwd: 'SchemaManager@2022',
        }
    },
    queryRewrite: (query, { securityContext }) => {
        const user = securityContext;
        if (!user.an_org_id) {
          throw new Error('No an_org_id found in Security Context!');
        }
       
        query.filters.push({
            member: 'PurchaseOrders.supplierOrg',
            operator: 'equals',
            values: [user.an_org_id],
        });
        
        return query;
    }
};
