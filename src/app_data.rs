use std::collections::HashMap;

use sea_orm::{
    entity::*, Condition, ConnectionTrait, DatabaseBackend, DatabaseConnection, DbConn,
    DeriveColumn, EnumIter, QueryFilter, QuerySelect, Statement,
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
                id bigint(20) NOT NULL AUTO_INCREMENT,
                address varchar(42) DEFAULT NULL,
                chain_id varchar(255) DEFAULT NULL,
                nonce varchar(32) DEFAULT '0',
                number varchar(64) DEFAULT '0',
                PRIMARY KEY (id),
                UNIQUE KEY address_chain_nonce (address,chain_id,nonce) USING BTREE
              ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
		"#
            .to_owned(),
        ))
        .await?;

        txn.commit().await.map_err(|e| e.into())
    }
}

pub async fn get_number(db: &DbConn, address: String, chain_id: String, nonce: String) -> String {
    #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
    enum QueryAs {
        Number,
    }
    println!("GET ");
    println!("chain_id {:?}", chain_id);
    println!("address {:?}", address);
    println!("nonce {:?}", nonce);
    Vault::find()
        .filter(
            Condition::all()
                .add(vault::Column::Address.eq(address))
                .add(vault::Column::ChainId.eq(chain_id))
                .add(vault::Column::Nonce.eq(nonce)),
        )
        .select_only()
        .column_as(vault::Column::Number, QueryAs::Number)
        .into_values::<_, QueryAs>()
        .one(db)
        .await
        .map_or_else(|_| 0.to_string(), |v| v.unwrap_or(0.to_string()))
}
