use cosmwasm_schema::{cw_serde};
use cosmwasm_std::{Uint128, Addr};
use cw721_base::Extension;

use crate::state::MintGroup;

#[cw_serde]
pub struct InstantiateMsg {
    pub extension: Extension,
    pub fee: Uint128,
    pub registeration_open: bool,
    pub denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        extension: Option<Extension>,
        fee: Option<Uint128>,
        registeration_open: Option<bool>,
    },
    RegisterCollection {
        cw721_code: u64,
        name: String,
        symbol: String,
        supply: u32,
        token_uri: String,
        royalty_percent: u64,
        royalty_wallet: String,
        mint_groups: Vec<MintGroup>,
        extension: Extension,
        iterated_uri: bool,
    },
    UpdateCollection {
        collection: String,
        name: String,
        symbol: String,
        supply: u32,
        token_uri: String,
        royalty_percent: u64,
        royalty_wallet: String,
        mint_groups: Vec<MintGroup>,
        iterated_uri: bool,
    },
    MintNative {
        collection: String,
        group: String,
        recipient: Option<Addr>,
        merkle_proof: Option<Vec<Vec<u8>>>,
        hashed_address: Option<Vec<u8>>,
    }
    /*MintCw20 {
        group: String,
        recipient: Option<Addr>,
        amount: Uint128,
    },*/
}

#[cw_serde]
pub enum QueryMsg {
    GetConfig {},
    GetCollection {
        collection: String,
    },
    BalanceOf {
        address: Addr,
        collection: String,
    },
    GetCollections {
        start_after: Option<String>,
        limit: Option<u32>,
        result_type: Option<String>, // "full" or "minimal"
    }
}
