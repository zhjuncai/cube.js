// Server options go here: https://cube.dev/docs/config#options-reference
module.exports = {
    driverFactory: ({ dataSource }) => {
        return {
            type: 'hana',
            serverNode: '4b3b5138-3c21-404d-89f0-8f942db8db09.hana.prod-us20.hanacloud.ondemand.com:443',
            uid: 'SCHEMA_MANAGER',
            pwd: 'SchemaManager@2022',
        }
    }
};
