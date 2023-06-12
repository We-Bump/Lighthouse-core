use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
#[cw_serde]
pub struct CollectionsResponse<T> {
    pub collections: Vec<T>,
}
#[cw_serde]
pub struct CollectionResponseMinimal {
    pub cw721_address: Option<Addr>
}
#[cw_serde]
pub struct Creator {
    pub address: String,
    pub share: u8,
}
