use std::collections::HashMap;

use sea_orm::{
    entity::*, Condition, ConnectionTrait, DatabaseBackend, DatabaseConnection, DeriveColumn,
    EnumIter, QueryFilter, QuerySelect, Statement,
};
use web3::types::{H160, H256};

use crate::entity::{prelude::*, *};

pub struct AppData {
    pub chains: Vec<u8>,
    pub claims: HashMap<String, H160>,
    pub private_key: H256,
    pub pool: DatabaseConnection,
}

impl AppData {
    pub fn new(
        chains: &[u8],
        claims: HashMap<String, H160>,
        private_key: H256,
        pool: DatabaseConnection,
    ) -> Self {
        Self {
            chains: chains.to_vec(),
            claims,
            private_key,
            pool,
        }
    }

    pub async fn init_vault_db(&self) -> Result<(), Box<dyn std::error::Error>> {
        let txn = self.pool.begin().await?;

        txn.execute(Statement::from_string(
            DatabaseBackend::MySql,
            r#"
            CREATE TABLE IF NOT EXISTS `vault` (
                `rounds` bigint(50) unsigned NOT NULL AUTO_INCREMENT,
                `account` varchar(42) NOT NULL,
                `nonce` bigint(50) unsigned NOT NULL DEFAULT 0,
                PRIMARY KEY (`rounds`),
                UNIQUE KEY `atao` (`rounds`,`account`,`nonce`) USING BTREE
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
		"#
            .to_owned(),
        ))
        .await?;

        txn.commit().await.map_err(|e| e.into())
    }

    pub async fn get_nonce(&self, rounds: u64, address: String) -> u64 {
        #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
        enum QueryAs {
            Number,
        }
        Vault::find()
            .select_only()
            .filter(
                Condition::all()
                    .add(vault::Column::Rounds.eq(rounds))
                    .add(vault::Column::Account.eq(address)),
            )
            .column_as(claims::Column::Number, QueryAs::Number)
            .into_values::<_, QueryAs>()
            .one(&self.pool)
            .await
            .map_or_else(|_| 0, |v| v.unwrap_or(0))
    }
}
