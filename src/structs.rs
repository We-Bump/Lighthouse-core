use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CosmosMsg, CustomMsg, Uint128};
use cw_utils::Expiration;
use schemars::JsonSchema;
use serde::{Serialize,Deserialize};

use crate::logo::Logo;

#[cw_serde]
pub struct Cw2981InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub minter: String,
    pub is_immutable: bool, // if true, UpdateTokenURI is not allowed
    pub frozen:Option<bool>,
    pub unfreeze_time: Option<u64>,
    pub hidden_metadata:Option<bool>,
    pub placeholder_token_uri: Option<String>,
    pub base_uri: Option<String>,
    pub fixed_uri: bool,
    pub base_uri_suffix: Option<String>,
    pub royalty_percentage: Option<u64>,
    pub royalty_payment_address: Option<String>,
}

#[cw_serde]
pub enum Cw2981LHExecuteMsg {
    Extension {
        msg: Cw2981LHExecuteExtension
    }
}

#[cw_serde]
pub enum Cw2981LHExecuteExtension {
    Unfreeze {},
    Reveal {},
    UpdateLighthouseData {
        placeholder_token_uri: Option<String>,
        base_uri: Option<String>,
        fixed_uri: bool,
        base_uri_suffix: Option<String>,
        default_royalty_percentage: Option<u64>,
        default_royalty_payment_address: Option<String>,
    },
}

impl CustomMsg for Cw2981LHExecuteMsg {}

#[cw_serde]
pub struct Cw404InstantiateMsg {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub is_immutable: bool,
    pub base_uri: String,
    pub base_uri_suffix: Option<String>,
    pub max_edition: u64,
    pub tokens_per_nft: Uint128,
    pub mint: Option<MinterResponse>,
    pub frozen_data: Option<FrozenData>,
    pub admin: Option<String>,
    pub royalty_percentage: Option<u64>,
    pub royalty_payment_address: Option<String>,
    pub marketing: Option<InstantiateMarketingInfo>,
}


#[cw_serde]
pub struct MinterResponse {
    pub minter: String,
    pub cap: Option<Uint128>,
}

#[cw_serde]
pub struct FrozenData {
    pub frozen: bool,
    pub unfreeze_time: Option<u64>,
    pub whitelisted: Vec<Addr>,
}

#[cw_serde]
pub struct InstantiateMarketingInfo {
    pub project: Option<String>,
    pub description: Option<String>,
    pub marketing: Option<String>,
    pub logo: Option<Logo>,
}

#[cw_serde]
pub enum Cw404ExecuteMsg {
    Mint {
        recipient: String,
        amount: Uint128,
    },
    Unfreeze {},
    Update404Data {
        base_uri: String,
        base_uri_suffix: Option<String>,
        max_edition: u64,
        royalty_percentage: Option<u64>,
        royalty_payment_address: Option<String>,
        frozen_whitelist: Option<Vec<Addr>>,
    },
    TransferOwnership {
        new_admin: String,
    },
}
impl CustomMsg for Cw404ExecuteMsg {}

#[cw_serde]
pub enum Cw20ExecuteMsg {
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
    },
    BurnFrom { owner: String, amount: Uint128 },
}

#[cw_serde]
pub enum Cw20QueryMsg {
    Balance { address: String },
}

#[cw_serde]
pub struct Cw20BalanceResponse {
    pub balance: Uint128,
}

#[cw_serde]
pub enum Cw721QueryMsg {
    OwnerOf {
        token_id: String,
        include_expired: Option<bool>,
    },
}

#[cw_serde]
pub struct Cw721OwnerOfResponse {
    pub owner: String,
    pub approvals: Vec<Approval>,
}

#[cw_serde]
pub struct Approval {
    pub spender: String,
    pub expires: Expiration,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvmQuery {
    StaticCall {
        from: String,
        to: String,
        data: String, // base64
    },
    GetEvmAddress {
        sei_address: String,
    },
    GetSeiAddress {
        evm_address: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetEvmAddressResponse {
    pub evm_address: String,
    pub associated: bool
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetSeiAddressResponse {
    pub sei_address: String,
    pub associated: bool
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvmMsg {
    CallEvm {
        to: String,
        value: Uint128,
        data: String, // base64 encoded
    },
}

// this is a helper to be able to return these as CosmosMsg easier
impl From<EvmMsg> for CosmosMsg<EvmMsg> {
    fn from(original: EvmMsg) -> Self {
        CosmosMsg::Custom(original)
    }
}

impl CustomMsg for EvmMsg {}

#[cw_serde]
pub struct MintInfo {
    pub mints: Vec<u32>
}

#[cw_serde]
pub struct GatedInfo {
    pub contract_addr: String,
    pub token_id: String,
}


#[cw_serde]
pub struct GatedQueryResponse {
    pub token_id: String,
    pub minted: bool,
}

#[cw_serde]
pub struct GatedQuery {
    pub contract_address: String,
    pub token_ids: Vec<String>,
}