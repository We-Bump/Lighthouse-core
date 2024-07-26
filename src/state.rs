use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

pub struct Lighthouse<'a>{
    pub config: Item<'a, Config>,
    pub collections: Map<'a, String, Collection>,
    pub global_mint_info: Map<'a, String, Uint128>,
    pub mint_info: Map<'a, String, MintInfo>,
    //pub gated_mint_info: Map<'a, String, String>,
    pub mint_logs: Map<'a, String, String>,
    pub partners: Map<'a, String, Partner>,
    pub instantiates: Map<'a, u64, Collection>,
}

impl Default for Lighthouse<'static> {
    fn default() -> Lighthouse<'static> {
        Self::new()
    }
}

impl<'a> Lighthouse<'a> {
    pub fn new() -> Lighthouse<'a> {
        Lighthouse {
            config: Item::new("config"),
            collections: Map::new("collections"),
            global_mint_info: Map::new("global_mint_info"),
            mint_info: Map::new("mint_info"),
            //gated_mint_info: Map::new("gated_mint_info"),
            mint_logs: Map::new("mint_logs"),
            partners: Map::new("partners"),
            instantiates: Map::new("instantiates"),
        }
    }
}

//CONFIG
#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub fee: Uint128,
    pub registeration_open: bool,
    pub next_reply_id: u64
}

//COLLECTION
#[cw_serde]
pub struct Collection {
    pub admin: Addr,
    pub chain: String, //v1,v2
    pub collection_type: String, //721,404
    pub collection_address: Option<String>,
    pub name: String,
    pub symbol: String,
    pub supply: Uint128,
    pub next_token: Uint128,
    pub mint_groups: Vec<MintGroup>,
    pub start_order: Option<Uint128>,
    pub frozen: bool,
    pub unfreeze_time: Option<u64>,
    pub hidden_metadata: bool,
    pub placeholder_token_uri: Option<String>,
    pub partner: Option<Addr>,
    pub cw404_info: Option<Cw404Info>,
}

#[cw_serde]
pub struct Cw404Info {
    pub decimals: u8,
    pub max_supply: Uint128,
    pub max_edition: u64,
    pub tokens_per_nft: Uint128,
    pub marketing: Option<Marketing>,
}

#[cw_serde]
pub struct Marketing {
    pub project: Option<String>,
    pub description: Option<String>,
    pub logo: Option<String>,
}

#[cw_serde]
pub struct MintGroup {
    pub name: String,
    pub merkle_root: Option<Vec<u8>>,
    pub max_mints_per_wallet: Uint128,
    pub reserved_supply: Uint128,
    pub start_time: u64,
    pub end_time: u64,
    pub payments: Vec<Payment>,
    pub batch_size: Option<Uint128>, //for 404
    pub gates: Vec<Gate>, // TODO: implement
    pub gates_optional: Option<bool>, // if true, any gate must be passed. if false, all gates must be passed
}

#[cw_serde]
pub enum PaymentType {
    Native,
    Cw20,
    Cw721, // TODO: implement
    Cw20Burn, 
    Cw721Burn, // TODO: implement
    Other(String),
}

#[cw_serde]
pub struct Payment {
    pub payment_type: PaymentType,
    pub amount: Option<Uint128>,
    pub args: Vec<String>,
}

impl Default for Payment {
    fn default() -> Payment {
        Payment {
            payment_type: PaymentType::Native,
            amount: None,
            args: vec![],
        }
    }
}

#[cw_serde]
pub enum GateType {
    Cw20Gate, // TODO: implement
    Cw721Gate, // TODO: implement
    Other(String),
}

#[cw_serde]
pub struct Gate {
    pub gate_type: GateType,
    pub amount: Option<Uint128>,
    pub args: Vec<String>,
}

impl Default for Gate {
    fn default() -> Gate {
        Gate {
            gate_type: GateType::Cw20Gate,
            amount: None,
            args: vec![],
        }
    }
}

//MINTINFO
#[cw_serde]
pub struct MintInfo {
    pub mints: Vec<Uint128>
}

//PARTNER
#[cw_serde]
pub struct Partner {
    pub partner: Addr,
    pub fee_percent: Uint128
}