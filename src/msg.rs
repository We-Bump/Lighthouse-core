use cosmwasm_schema::cw_serde;
use cosmwasm_std::{ Addr, CustomQuery, Uint128 };
use schemars::JsonSchema;
use serde::{ Deserialize, Serialize };

use crate::{ state::{Cw404Info, MintGroup}, structs::EvmQuery };

#[cw_serde]
pub struct InstantiateMsg {
    pub fee: Uint128,
    pub registeration_open: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateConfig {
    pub fee: Uint128,
    pub registeration_open: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RegisterCollection {
    pub cw721_code: Option<u64>,
    pub cw404_code: Option<u64>,
    pub erc721_address: Option<String>,
    pub chain: String, //v1,v2
    pub collection_type: String, //721,404
    pub name: String,
    pub symbol: String,
    pub supply: Uint128,
    pub token_uri: String,
    pub royalty_percent: Option<u64>,
    pub royalty_wallet: Option<String>,
    pub mint_groups: Vec<MintGroup>,
    pub is_immutable: bool,
    pub fixed_uri: bool,
    pub start_order: Option<Uint128>,
    pub uri_suffix: Option<String>,
    pub frozen: bool,
    pub unfreeze_time: Option<u64>,
    pub frozen_whitelist: Option<Vec<Addr>>,
    pub hidden_metadata: bool,
    pub placeholder_token_uri: Option<String>,
    pub partner: Option<Addr>,
    pub cw404_info: Option<Cw404Info>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateCollection {
    pub collection: String,
    pub supply: Uint128,
    pub token_uri: String,
    pub royalty_percent: Option<u64>,
    pub royalty_wallet: Option<String>,
    pub mint_groups: Vec<MintGroup>,
    pub fixed_uri: bool,
    pub uri_suffix: Option<String>,
    pub start_order: Option<Uint128>,
    pub placeholder_token_uri: Option<String>,
    pub max_edition: Option<u64>,
    pub frozen_whitelist: Option<Vec<Addr>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Mint {
    pub collection: String,
    pub group: String,
    pub amount: Uint128,
    pub merkle_proof: Option<Vec<Vec<u8>>>,
    pub merkle_proof_address_type: Option<String>, // hex or bech32
    pub payment_args: Option<Vec<MintArg>>,
    pub gate_args: Option<Vec<MintArg>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintArg {
    pub index: usize,
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnfreezeCollection {
    pub collection: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RevealCollectionMetadata {
    pub collection: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateAdmin {
    pub collection: String,
    pub admin: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AddPartner {
    pub address: Addr,
    pub percent: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateNftContractCwOwnableOwner {
    pub collection: String,
    pub new_admin: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UpdateNftContractAdmin {
    pub collection: String,
    pub new_admin: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig(UpdateConfig),
    RegisterCollection(RegisterCollection),
    UpdateCollection(UpdateCollection),
    Mint(Mint),
    UnfreezeCollection(UnfreezeCollection),
    RevealCollectionMetadata(RevealCollectionMetadata),
    UpdateAdmin(UpdateAdmin),
    AddPartner(AddPartner),
    UpdateNftContractCwOwnableOwner(UpdateNftContractCwOwnableOwner),
    UpdateNftContractAdmin(UpdateNftContractAdmin),
}

#[cw_serde]
pub enum QueryMsg {
    GetConfig {},
    GetCollection {
        collection: String,
    },
    MintsOf {
        address: String,
        collection: String,
    },
    GetMinterOf {
        collection: String,
        token_id: String,
    },
    GetGlobalMintInfo {
        collection: String,
        group_name: String,
    },
    GetEvmAddressOfBech32Address {
        address: String,
    },
    GetBech32AddressOfEvmAddress {
        address: String,
    },
    /*GetGatedMintsOf {
        collection: String,
        group_name: String,
        contract_address: String,
        token_ids: Vec<String>,
    },*/
}

//evm stuff

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Route {
    Evm,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct EvmQueryWrapper {
    pub route: Route,
    pub query_data: EvmQuery,
}

// implement custom query
impl CustomQuery for EvmQueryWrapper {}
