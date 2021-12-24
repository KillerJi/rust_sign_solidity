use crate::entity::claims;
use crate::Claims;
use sea_orm::{
    entity::*, Condition, DbConn, DeriveColumn, EnumIter, QueryFilter, QueryOrder, QuerySelect,
};

pub async fn get_nonce(db: &DbConn, token: String, address: String, chain_id: String) -> u64 {
    #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
    enum QueryAs {
        Nonce,
    }
    Claims::find()
        .filter(
            Condition::all()
                .add(claims::Column::Address.contains(&address))
                .add(claims::Column::Token.contains(&token))
                .add(claims::Column::ChainId.contains(&chain_id)),
        )
        .order_by_desc(claims::Column::Nonce)
        .select_only()
        .column_as(claims::Column::Nonce, QueryAs::Nonce)
        .into_values::<_, QueryAs>()
        .one(db)
        .await
        .map_or_else(|_| 0, |v| v.unwrap_or(0))
}
