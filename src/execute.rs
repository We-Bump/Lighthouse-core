use std::marker::PhantomData;

use cosmwasm_std::{
    DepsMut,
    Env,
    MessageInfo,
    Response,
    WasmMsg,
    SubMsg,
    ReplyOn,
    to_binary,
    Empty,
    Addr,
    Uint128,
    BankMsg,
    coins,
};
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::helpers::{ create_group_key, create_token_uri, validate_merkle_proof, hash };

use crate::state::{
    CONFIG,
    Config,
    MintGroup,
    Collection,
    COLLECTIONS,
    INSTANTIATE_INFO,
    MintInfo,
    MINT_INFO,
};
use cw721_base::{ helpers::Cw721Contract, msg::InstantiateMsg as Cw721InstantiateMsg, Extension };

use cw2981_royalties::{ ExecuteMsg as Cw2981ExecuteMsg, Metadata as Cw2981Metadata };

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    extension: Option<Extension>,
    fee: Option<Uint128>,
    registeration_open: Option<bool>
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if config.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if extension.is_some() {
        config.extension = extension.unwrap();
    }

    if fee.is_some() {
        config.fee = fee.unwrap();
    }

    if registeration_open.is_some() {
        config.registeration_open = registeration_open.unwrap();
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

pub fn register_collection(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw721_code: u64,
    name: String,
    symbol: String,
    supply: u32,
    token_uri: String,
    royalty_percent: u64,
    royalty_wallet: String,
    mint_groups: Vec<MintGroup>,
    iterated_uri: bool,
    extension: Extension
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    if config.registeration_open == false {
        return Err(ContractError::RegisterationClose {});
    }

    let collection = Collection {
        admin: info.sender,
        cw721_address: None,
        name: name.clone(),
        symbol: symbol.clone(),
        supply,
        token_uri,
        royalty_percent,
        royalty_wallet,
        next_token_id: 0,
        mint_groups,
        extension,
        iterated_uri: iterated_uri
    };

    for group in collection.mint_groups.clone() {
        let mut total_share = 0;
        for share in group.creators.clone() {
            total_share += share.share;
        }
        if total_share != 100 {
            return Err(ContractError::InvalidShares {});
        }
    }

    INSTANTIATE_INFO.save(deps.storage, config.next_reply_id.clone(), &collection)?;

    let sub_msg: Vec<SubMsg> = vec![SubMsg {
        msg: (WasmMsg::Instantiate {
            code_id: cw721_code,
            msg: to_binary(
                &(Cw721InstantiateMsg {
                    name: name.clone(),
                    symbol: symbol.clone(),
                    minter: env.contract.address.to_string(),
                })
            )?,
            funds: vec![],
            admin: None,
            label: String::from("Instantiate CW721"),
        }).into(),
        id: config.next_reply_id.clone(),
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }];

    config.next_reply_id += 1;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_submessages(sub_msg))
}

pub fn update_collection(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_addr: String,
    name: String,
    symbol: String,
    supply: u32,
    token_uri: String,
    royalty_percent: u64,
    royalty_wallet: String,
    mint_groups: Vec<MintGroup>,
    iterated_uri: bool
) -> Result<Response, ContractError> {
    let mut collection = COLLECTIONS.load(deps.storage, collection_addr.clone())?;

    if collection.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if supply < collection.next_token_id {
        return Err(ContractError::SupplyLowerThanMinted {});
    }

    for group in collection.mint_groups.clone() {
        let mut total_share = 0;
        for share in group.creators.clone() {
            total_share += share.share;
        }
        if total_share != 100 {
            return Err(ContractError::InvalidShares {});
        }
    }

    collection.name = name;
    collection.symbol = symbol;
    collection.supply = supply;
    collection.token_uri = token_uri;
    collection.royalty_percent = royalty_percent;
    collection.royalty_wallet = royalty_wallet;
    collection.mint_groups = mint_groups;
    collection.iterated_uri = iterated_uri;

    COLLECTIONS.save(deps.storage, collection_addr.clone(), &collection)?;

    Ok(Response::default())
}

pub fn mint_native(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_addr: String,
    group: String,
    recipient_addr: Option<Addr>,
    merkle_proof: Option<Vec<Vec<u8>>>,
    hashed_address: Option<Vec<u8>>
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let mut collection = COLLECTIONS.load(deps.storage, collection_addr.clone())?;

    let recipient = recipient_addr.unwrap_or(info.sender.clone());

    // Check if sold out
    if collection.next_token_id >= collection.supply {
        return Err(ContractError::SoldOut {});
    }

    // Find mint group
    let group_check = collection.mint_groups.iter().find(|&g| g.name == group);
    if group_check.is_none() {
        return Err(ContractError::InvalidMintGroup {});
    }

    let group = group_check.unwrap();

    // Check if the mint group is open (unix timestamp)
    if group.start_time > env.block.time.seconds() {
        return Err(ContractError::GroupNotOpenToMint {});
    }

    if group.end_time != 0 && group.end_time < env.block.time.seconds() {
        return Err(ContractError::GroupNotOpenToMint {});
    }

    // Validate merkle proof (if any merkle root is set)
    if group.merkle_root.is_some() {
        if merkle_proof.is_none() || hashed_address.is_none() {
            return Err(ContractError::InvalidMerkleProof {});
        }

        // Get the hashed address from the recipients address
        let sender_address_hash = hash(&recipient.to_string());

        if sender_address_hash != hashed_address.clone().unwrap() {
            return Err(ContractError::InvalidSender {});
        }
        
        // Check that the merkle proof and root is valid
        let merkle_root = group.merkle_root.clone().unwrap();
        if
            !validate_merkle_proof(
                merkle_proof.unwrap(),
                merkle_root,
                hashed_address.unwrap()
            )
        {
            return Err(ContractError::InvalidMerkleProof {});
        }
    }

    // Get the mint info for the group (if any) (mint count)
    let key = create_group_key(&recipient, &collection_addr, &group.name);
    let mut mint_info = MINT_INFO.load(deps.storage, key.clone()).unwrap_or(MintInfo {
        mints: Vec::new(),
    });

    // Check if the sender already minted the max tokens
    if group.max_tokens != 0 && (mint_info.mints.len() as u32) >= group.max_tokens {
        return Err(ContractError::MaxTokensMinted {});
    }

    if !group.unit_price.is_zero(){
        // Check if the sender have enough funds
        if
            info.funds.len() != 1 ||
            info.funds[0].denom != config.denom ||
            info.funds[0].amount != group.unit_price + config.fee
        {
            return Err(ContractError::InvalidFunds {});
        }
    } else{
        // Check if the sender have enough funds
        if
            info.funds.len() != 1 ||
            info.funds[0].denom != config.denom ||
            info.funds[0].amount != config.fee
        {
            return Err(ContractError::InvalidFunds {});
        }
    }

    let mut response = Response::new();

    if !group.unit_price.is_zero(){
        // Transfer the funds to the collection creator wallet
        for share in group.creators.clone() {
            let creator_funds = BankMsg::Send {
                to_address: share.address.to_string(),
                amount: coins(
                    group.unit_price.u128() * (share.share as u128) / 100,
                    config.denom.clone()
                ),
            };

            response = response.add_message(creator_funds);
        }
    }

    // Transfer the fee contract admin
    let admin_funds = BankMsg::Send {
        to_address: config.admin.to_string(),
        amount: coins(config.fee.u128(), config.denom.clone()),
    };

    // Init royalty extension
    let extension = Some(Cw2981Metadata {
        royalty_payment_address: Some(collection.royalty_wallet.clone().to_string()),
        royalty_percentage: Some(collection.royalty_percent),
        ..Cw2981Metadata::default()
    });

    // Prepare the mint message
    let mint_msg = Cw2981ExecuteMsg::Mint {
        token_id: collection.next_token_id.to_string(),
        owner: recipient.to_string(),
        token_uri: Some(
            create_token_uri(&collection.token_uri, &collection.next_token_id.to_string(), &collection.iterated_uri)
        ),
        extension,
    };

    // Send the mint message
    let callback = Cw721Contract::<Empty, Empty>(
        collection.cw721_address.clone().unwrap(),
        PhantomData,
        PhantomData
    ).call(mint_msg)?;

    // Update the next token id
    collection.next_token_id += 1;
    COLLECTIONS.save(deps.storage, collection_addr.clone(), &collection)?;

    // Update the mint info
    mint_info.mints.push(collection.next_token_id - 1);
    MINT_INFO.save(deps.storage, key, &mint_info)?;

    // Return the response
    Ok(
        response
            .add_message(callback)
            .add_message(admin_funds)
            .add_attribute("action", "mint")
            .add_attribute("collection", collection_addr)
            .add_attribute("group", group.name.clone())
            .add_attribute("recipient", recipient.to_string())
            .add_attribute("token_id", (collection.next_token_id.clone() - 1).to_string())
            .add_attribute("price", group.unit_price.to_string())
    )
}
