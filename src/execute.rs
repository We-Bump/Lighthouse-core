use cosmwasm_std::{
    coins,
    to_json_binary,
    Attribute,
    BankMsg,
    CosmosMsg,
    DepsMut,
    Env,
    MessageInfo,
    ReplyOn,
    Response,
    StdError,
    SubMsg,
    Uint128,
    WasmMsg,
};
use data_encoding::BASE64;
use sha3::{ Digest, Keccak256 };
use cw2981_royalties::{ ExecuteMsg as Cw2981ExecuteMsg, Metadata as Cw2981Metadata };

use crate::{
    helpers::{
        convert_bech32_to_hex,
        create_global_mint_info_key,
        create_group_key,
        create_mint_log_key,
        create_mint_log_key_404,
        pad_address_to_bytes32,
        validate_groups,
        validate_merkle_proof,
        validate_payments,
    },
    logo::Logo,
    msg::{
        AddPartner,
        EvmQueryWrapper,
        ExecuteMsg,
        InstantiateMsg,
        Mint,
        RegisterCollection,
        RevealCollectionMetadata,
        UnfreezeCollection,
        UpdateAdmin,
        UpdateCollection,
        UpdateConfig,
        UpdateNftContractAdmin,
        UpdateNftContractCwOwnableOwner,
    },
    querier::EvmQuerier,
    state::{ Collection, Config, Lighthouse, MintInfo, Partner, PaymentType },
    structs::{
        Cw20ExecuteMsg,
        Cw2981InstantiateMsg,
        Cw2981LHExecuteExtension,
        Cw2981LHExecuteMsg,
        Cw404ExecuteMsg,
        Cw404InstantiateMsg,
        EvmMsg,
        FrozenData,
        InstantiateMarketingInfo,
        MinterResponse,
    },
    ContractError,
};

impl<'a> Lighthouse<'a> {
    pub fn instantiate(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        msg: InstantiateMsg
    ) -> Result<Response, ContractError> {
        self.config.save(
            deps.storage,
            &(Config {
                admin: info.sender,
                fee: msg.fee,
                registeration_open: msg.registeration_open,
                next_reply_id: 0,
            })
        )?;

        Ok(Response::new().add_attribute("action", "instantiate"))
    }

    pub fn execute(
        &self,
        deps: DepsMut<EvmQueryWrapper>,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg
    ) -> Result<Response<EvmMsg>, ContractError> {
        match msg {
            ExecuteMsg::UpdateConfig(params) => self.update_config(deps, info, env, params),
            ExecuteMsg::RegisterCollection(params) =>
                self.register_collection(deps, env, info, params),
            ExecuteMsg::UpdateCollection(params) => self.update_collection(deps, env, info, params),
            ExecuteMsg::Mint(params) => self.mint(deps, env, info, params),
            ExecuteMsg::UnfreezeCollection(params) =>
                self.unfreeze_collection(deps, env, info, params),
            ExecuteMsg::RevealCollectionMetadata(params) =>
                self.reveal_collection_metadata(deps, env, info, params),
            ExecuteMsg::UpdateAdmin(params) => self.update_admin(deps, env, info, params),
            ExecuteMsg::AddPartner(params) => self.add_partner(deps, env, info, params),
            ExecuteMsg::UpdateNftContractCwOwnableOwner(params) =>
                self.update_nft_contract_cw_ownable_owner(deps, env, info, params),
            ExecuteMsg::UpdateNftContractAdmin(params) =>
                self.update_nft_contract_admin(deps, env, info, params),
        }
    }

    fn update_config(
        &self,
        deps: DepsMut<EvmQueryWrapper>,
        info: MessageInfo,
        _env: Env,
        msg: UpdateConfig
    ) -> Result<Response<EvmMsg>, ContractError> {
        let mut config = self.config.load(deps.storage)?;

        if config.admin != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        config.fee = msg.fee;
        config.registeration_open = msg.registeration_open;
        self.config.save(deps.storage, &config)?;

        Ok(Response::new().add_attribute("action", "update_config"))
    }

    fn register_collection(
        &self,
        deps: DepsMut<EvmQueryWrapper>,
        env: Env,
        info: MessageInfo,
        msg: RegisterCollection
    ) -> Result<Response<EvmMsg>, ContractError> {
        let mut config = self.config.load(deps.storage)?;

        if config.registeration_open == false {
            return Err(ContractError::RegisterationClose {});
        }

        if msg.chain == "v1" && msg.collection_type == "721" {
            if msg.cw721_code.is_none() {
                return Err(
                    ContractError::Std(StdError::generic_err("cw721_code is required for v1"))
                );
            }
        } else if msg.chain == "v2" && msg.collection_type == "721" {
            if msg.erc721_address.is_none() {
                return Err(
                    ContractError::Std(StdError::generic_err("erc721_address is required for v2"))
                );
            }
        } else if msg.chain == "v1" && msg.collection_type == "404" {
            if msg.cw404_code.is_none() {
                return Err(
                    ContractError::Std(StdError::generic_err("cw404_code is required for v1"))
                );
            }
            if msg.start_order.is_some() {
                return Err(
                    ContractError::Std(StdError::generic_err("start_order must be None for 404"))
                );
            }
        } else {
            return Err(ContractError::InvalidChain {});
        }

        // Validate payments and groups
        validate_payments(&deps, &msg.mint_groups)?;
        validate_groups(&msg.collection_type, &msg.mint_groups)?;

        let mut collection = Collection {
            admin: info.sender.clone(),
            chain: msg.chain.clone(),
            collection_type: msg.collection_type.clone(),
            collection_address: None,
            name: msg.name.clone(),
            symbol: msg.symbol.clone(),
            supply: msg.supply,
            next_token: msg.start_order.unwrap_or(Uint128::zero()),
            mint_groups: msg.mint_groups.clone(),
            start_order: msg.start_order,
            frozen: msg.frozen,
            unfreeze_time: msg.unfreeze_time.clone(),
            hidden_metadata: msg.hidden_metadata,
            placeholder_token_uri: msg.placeholder_token_uri.clone(),
            partner: msg.partner.clone(),
            cw404_info: msg.cw404_info.clone(),
        };

        if msg.partner.is_some() {
            let partner = msg.partner.unwrap();
            if !self.partners.has(deps.storage, partner.to_string()) {
                return Err(ContractError::PartnerNotFound {});
            }
        }

        if msg.chain == "v1" && msg.collection_type == "721" {
            self.instantiates.save(deps.storage, config.next_reply_id.clone(), &collection)?;

            let sub_msg: Vec<SubMsg<EvmMsg>> = vec![SubMsg {
                msg: (WasmMsg::Instantiate {
                    code_id: msg.cw721_code.unwrap(),
                    msg: to_json_binary(
                        &(Cw2981InstantiateMsg {
                            name: msg.name.clone(),
                            symbol: msg.symbol.clone(),
                            minter: env.contract.address.to_string(),
                            is_immutable: msg.is_immutable,
                            frozen: Some(msg.frozen),
                            unfreeze_time: msg.unfreeze_time,
                            hidden_metadata: Some(msg.hidden_metadata),
                            placeholder_token_uri: msg.placeholder_token_uri,
                            base_uri: Some(msg.token_uri.clone()),
                            fixed_uri: msg.fixed_uri,
                            base_uri_suffix: msg.uri_suffix.clone(),
                            royalty_percentage: msg.royalty_percent,
                            royalty_payment_address: msg.royalty_wallet.clone(),
                        })
                    )?,
                    funds: vec![],
                    admin: Some(info.sender.to_string()),
                    label: String::from("Instantiate CW721"),
                }).into(),
                id: config.next_reply_id.clone(),
                gas_limit: None,
                reply_on: ReplyOn::Success,
            }];

            config.next_reply_id += 1;
            self.config.save(deps.storage, &config)?;
            Ok(Response::<EvmMsg>::new().add_submessages(sub_msg))
        } else if msg.chain == "v1" && msg.collection_type == "404" {
            self.instantiates.save(deps.storage, config.next_reply_id.clone(), &collection)?;

            let cw404_info = msg.cw404_info.unwrap();

            let marketing_data: Option<InstantiateMarketingInfo>;
            let logo: Option<Logo>;
            if
                cw404_info.marketing.is_some() &&
                cw404_info.marketing.clone().unwrap().logo.is_some()
            {
                logo = Some(Logo::Url(cw404_info.marketing.clone().unwrap().logo.unwrap()));
            } else {
                logo = None;
            }

            if let Some(marketing) = cw404_info.marketing.clone() {
                marketing_data = Some(InstantiateMarketingInfo {
                    project: marketing.project,
                    description: marketing.description,
                    marketing: Some(info.sender.to_string()),
                    logo,
                });
            } else {
                marketing_data = None;
            }

            let sub_msg: Vec<SubMsg<EvmMsg>> = vec![SubMsg {
                msg: (WasmMsg::Instantiate {
                    code_id: msg.cw404_code.unwrap(),
                    msg: to_json_binary(
                        &(Cw404InstantiateMsg {
                            name: msg.name.clone(),
                            symbol: msg.symbol.clone(),
                            decimals: cw404_info.decimals,
                            is_immutable: msg.is_immutable,
                            base_uri: msg.token_uri.clone(),
                            base_uri_suffix: msg.uri_suffix.clone(),
                            max_edition: cw404_info.max_edition,
                            tokens_per_nft: cw404_info.tokens_per_nft,
                            mint: Some(MinterResponse {
                                minter: env.contract.address.to_string(),
                                cap: Some(cw404_info.max_supply),
                            }),
                            frozen_data: Some(FrozenData {
                                frozen: msg.frozen,
                                unfreeze_time: msg.unfreeze_time,
                                whitelisted: msg.frozen_whitelist.clone().unwrap_or(Vec::new()),
                            }),
                            admin: Some(info.sender.to_string()),
                            royalty_percentage: msg.royalty_percent,
                            royalty_payment_address: msg.royalty_wallet.clone(),
                            marketing: marketing_data,
                        })
                    )?,
                    funds: vec![],
                    admin: Some(info.sender.to_string()),
                    label: String::from("Instantiate CW404"),
                }).into(),
                id: config.next_reply_id.clone(),
                gas_limit: None,
                reply_on: ReplyOn::Success,
            }];

            config.next_reply_id += 1;
            self.config.save(deps.storage, &config)?;
            Ok(Response::<EvmMsg>::new().add_submessages(sub_msg))
        } else if msg.chain == "v2" {
            if self.collections.has(deps.storage, msg.erc721_address.clone().unwrap()) {
                return Err(ContractError::CollectionExists {});
            }

            let querier = EvmQuerier::new(&deps.querier);
            let user_evm_address_query = querier.query_evm_address(info.sender.to_string())?;

            if !user_evm_address_query.associated {
                return Err(ContractError::NotAssociatedAddress {});
            }

            let user_evm_address = user_evm_address_query.evm_address;

            let has_admin_role = querier.query_has_role(
                env.contract.address.to_string(),
                user_evm_address.clone(),
                msg.erc721_address.clone().unwrap(),
                vec![0u8; 32] //DEFAULT_ADMIN_ROLE
            )?;
            let has_minter_role = querier.query_has_role(
                env.contract.address.to_string(),
                convert_bech32_to_hex(env.contract.address.to_string().as_str())?,
                msg.erc721_address.clone().unwrap(),
                Keccak256::digest(b"MINTER_ROLE").to_vec()
            )?;

            if !has_admin_role {
                return Err(ContractError::Unauthorized {});
            }

            if !has_minter_role {
                return Err(ContractError::NotMinter {});
            }

            collection.collection_address = msg.erc721_address.clone();
            self.collections.save(deps.storage, msg.erc721_address.clone().unwrap(), &collection)?;

            Ok(
                Response::new()
                    .add_attribute("register_collection", "success")
                    .add_attribute("collection", msg.erc721_address.unwrap())
                    .add_attribute("type", collection.collection_type)
                    .add_attribute("chain", msg.chain)
                /*.add_attribute("user_evm_address", user_evm_address)
                    .add_attribute("has_admin_role", has_admin_role.to_string())
                    .add_attribute("has_minter_role", has_minter_role.to_string())*/
            )
        } else {
            return Err(ContractError::InvalidChain {});
        }
    }

    pub fn update_collection(
        &self,
        deps: DepsMut<EvmQueryWrapper>,
        _env: Env,
        info: MessageInfo,
        msg: UpdateCollection
    ) -> Result<Response<EvmMsg>, ContractError> {
        let mut collection = self.collections.load(deps.storage, msg.collection.clone())?;

        if collection.admin != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        if msg.supply < collection.next_token - collection.start_order.unwrap_or(Uint128::zero()) {
            return Err(ContractError::SupplyLowerThanMinted {});
        }

        // Validate payments and groups
        validate_payments(&deps, &msg.mint_groups)?;
        validate_groups(&collection.collection_type, &msg.mint_groups)?;

        if msg.start_order.is_some() && msg.start_order.unwrap() == collection.next_token {
            collection.next_token = msg.start_order.unwrap();
            collection.start_order = msg.start_order;
        }

        collection.supply = msg.supply;
        collection.mint_groups = msg.mint_groups;

        if collection.chain == "v1" && collection.collection_type == "721" {
            let execute: CosmosMsg<EvmMsg> = (WasmMsg::Execute {
                contract_addr: msg.collection.clone(),
                msg: to_json_binary(
                    &(Cw2981LHExecuteMsg::Extension {
                        msg: Cw2981LHExecuteExtension::UpdateLighthouseData {
                            placeholder_token_uri: msg.placeholder_token_uri,
                            base_uri: Some(msg.token_uri.clone()),
                            fixed_uri: msg.fixed_uri,
                            base_uri_suffix: msg.uri_suffix.clone(),
                            default_royalty_percentage: msg.royalty_percent,
                            default_royalty_payment_address: msg.royalty_wallet,
                        },
                    })
                )?,
                funds: vec![],
            }).into();

            self.collections.save(deps.storage, msg.collection.clone(), &collection)?;

            return Ok(Response::<EvmMsg>::new().add_message(execute));
        } else if collection.chain == "v1" && collection.collection_type == "404" {
            let execute: CosmosMsg<EvmMsg> = (WasmMsg::Execute {
                contract_addr: msg.collection.clone(),
                msg: to_json_binary(
                    &(Cw404ExecuteMsg::Update404Data {
                        base_uri: msg.token_uri.clone(),
                        base_uri_suffix: msg.uri_suffix.clone(),
                        max_edition: msg.max_edition.unwrap(),
                        royalty_percentage: msg.royalty_percent,
                        royalty_payment_address: msg.royalty_wallet,
                        frozen_whitelist: msg.frozen_whitelist,
                    })
                )?,
                funds: vec![],
            }).into();

            self.collections.save(deps.storage, msg.collection.clone(), &collection)?;

            return Ok(Response::<EvmMsg>::new().add_message(execute));
        }

        self.collections.save(deps.storage, msg.collection.clone(), &collection)?;

        Ok(Response::default())
    }

    pub fn mint(
        &self,
        deps: DepsMut<EvmQueryWrapper>,
        env: Env,
        info: MessageInfo,
        msg: Mint
    ) -> Result<Response<EvmMsg>, ContractError> {
        let config = self.config.load(deps.storage)?;
        let mut collection = self.collections.load(deps.storage, msg.collection.clone())?;

        let recipient = info.sender.clone();

        // Find mint group
        let group_check = collection.mint_groups.iter().find(|&g| g.name == msg.group);
        if group_check.is_none() {
            return Err(ContractError::InvalidMintGroup {});
        }

        let group = group_check.unwrap();

        // Check if sold out
        if
            collection.collection_type == "721" &&
            collection.next_token + msg.amount - collection.start_order.unwrap_or(Uint128::zero()) >
                collection.supply
        {
            return Err(ContractError::SoldOut {});
        } else if
            collection.collection_type == "404" &&
            collection.next_token + msg.amount * group.batch_size.unwrap() > collection.supply
        {
            return Err(ContractError::SoldOut {});
        }

        // Check if the mint group is open (unix timestamp)
        if group.start_time > env.block.time.seconds() {
            return Err(ContractError::GroupNotOpenToMint {});
        }

        if group.end_time != 0 && group.end_time < env.block.time.seconds() {
            return Err(ContractError::GroupNotOpenToMint {});
        }

        // Get the mint info for the group (if any) (mint count)
        let mint_info_key = create_group_key(&recipient.to_string(), &msg.collection, &group.name);
        let mut mint_info = self.mint_info
            .load(deps.storage, mint_info_key.clone())
            .unwrap_or(MintInfo {
                mints: Vec::new(),
            });

        // Check if the sender already minted the max tokens
        if
            group.max_mints_per_wallet.u128() != 0 &&
            (mint_info.mints.len() as u128) + msg.amount.u128() > group.max_mints_per_wallet.u128()
        {
            return Err(ContractError::MaxTokensMinted {});
        }

        // Get the global mint info for the group (if any) (reserved supply)
        let global_mint_info_key = create_global_mint_info_key(&msg.collection, &group.name);
        let mut global_mint_info = self.global_mint_info
            .load(deps.storage, global_mint_info_key.clone())
            .unwrap_or(Uint128::zero());

        // Check if the reserved supply ran out for the group
        if
            group.reserved_supply != Uint128::zero() &&
            global_mint_info + msg.amount > group.reserved_supply
        {
            return Err(ContractError::ReservedSupplyRanOut {});
        }

        //query recipient evm address
        let querier: EvmQuerier<'_> = EvmQuerier::new(&deps.querier);
        let recipient_evm_address_query = querier.query_evm_address(recipient.to_string())?;

        if !recipient_evm_address_query.associated {
            return Err(ContractError::NotAssociatedAddress {});
        }

        let recipient_evm_address = recipient_evm_address_query.evm_address;

        // Validate merkle proof (if any merkle root is set)
        if group.merkle_root.is_some() {
            if msg.merkle_proof.is_none() {
                return Err(ContractError::InvalidMerkleProof {});
            }

            let mut recipient_addr = recipient.clone().to_string();

            if msg.merkle_proof_address_type.is_some() {
                if msg.merkle_proof_address_type.unwrap() == "hex" {
                    recipient_addr = recipient_evm_address.to_lowercase();
                }
            }

            // Check that the merkle proof and root is valid
            let merkle_root = group.merkle_root.clone().unwrap();
            if
                !validate_merkle_proof(
                    msg.merkle_proof.unwrap(),
                    merkle_root,
                    Keccak256::digest(&recipient_addr.as_bytes()).to_vec()
                )
            {
                return Err(ContractError::InvalidMerkleProof {});
            }
        }

        // get the total native and cw20 payments
        let total_native_payment =
            group.payments
                .iter()
                .filter(|&p| p.payment_type == PaymentType::Native)
                .fold(Uint128::zero(), |acc, p| acc + p.amount.unwrap()) * msg.amount;

        let total_cw20_payment =
            group.payments
                .iter()
                .filter(|&p| p.payment_type == PaymentType::Cw20)
                .fold(Uint128::zero(), |acc, p| acc + p.amount.unwrap()) * msg.amount;

        if total_native_payment > Uint128::zero() {
            // Check if the sender have enough funds
            if
                info.funds.len() != 1 ||
                info.funds[0].denom != "usei" ||
                info.funds[0].amount != total_native_payment + config.fee * msg.amount
            {
                return Err(ContractError::InvalidFunds {});
            }
        }

        // Prepare the response
        let mut response: Response<EvmMsg> = Response::new();
        let mut attrs: Vec<Attribute> = Vec::new();

        for payment in group.payments.clone() {
            match payment.payment_type {
                PaymentType::Native => {
                    // Transfer the funds to the destination wallet
                    let total = payment.amount.unwrap().u128() * msg.amount.u128();
                    response = response.add_message(BankMsg::Send {
                        to_address: payment.args[0].clone(),
                        amount: coins(total, "usei"),
                    });

                    attrs.push(Attribute {
                        key: format!("paid_sei"),
                        value: total.to_string(),
                    });
                }
                PaymentType::Cw20 => {
                    // Transfer the funds to the destination wallet
                    let total = payment.amount.unwrap() * msg.amount;
                    response = response.add_message(WasmMsg::Execute {
                        contract_addr: payment.args[1].clone(),
                        msg: to_json_binary(
                            &(Cw20ExecuteMsg::TransferFrom {
                                owner: recipient.to_string(),
                                recipient: payment.args[0].clone(),
                                amount: total,
                            })
                        )?,
                        funds: vec![],
                    });
                    attrs.push(Attribute {
                        key: format!("paid_{}", payment.args[1]),
                        value: total.to_string(),
                    });
                }
                PaymentType::Cw20Burn => {
                    // Burn the funds
                    let total = payment.amount.unwrap() * msg.amount;
                    response = response.add_message(WasmMsg::Execute {
                        contract_addr: payment.args[0].clone(),
                        msg: to_json_binary(
                            &(Cw20ExecuteMsg::BurnFrom {
                                owner: recipient.to_string(),
                                amount: total,
                            })
                        )?,
                        funds: vec![],
                    });
                    attrs.push(Attribute {
                        key: format!("burned_{}", payment.args[0]),
                        value: total.to_string(),
                    });
                }
                _ => {
                    return Err(ContractError::NotImplemented {});
                }
            }
        }

        if total_native_payment > Uint128::zero() || total_cw20_payment > Uint128::zero() {
            let mut admin_fee = config.fee.u128() * msg.amount.u128();
            // Transfer the admin fee to the collection admin and partner (if any)
            if collection.partner.is_some() {
                let partner = self.partners.may_load(
                    deps.storage,
                    collection.partner.clone().unwrap().to_string()
                )?;
                if partner.is_some() {
                    let partner_data = partner.unwrap();
                    if partner_data.fee_percent > Uint128::zero() {
                        let partner_fee =
                            (config.fee.u128() * partner_data.fee_percent.u128()) / 100; //calculate partner fee
                        response = response.add_message(BankMsg::Send {
                            to_address: partner_data.partner.to_string(),
                            amount: coins(partner_fee, "usei"),
                        });

                        admin_fee -= partner_fee; //deduct partner fee from admin fee
                    }
                }
            }

            response = response.add_message(BankMsg::Send {
                to_address: config.admin.to_string(),
                amount: coins(admin_fee, "usei"),
            });
        }

        //mint
        if collection.collection_type == "721" {
            if collection.chain == "v1" {
                // Init royalty extension
                let extension = Some(Cw2981Metadata {
                    ..Cw2981Metadata::default()
                });

                for order in 0..msg.amount.u128() {
                    // Prepare the mint message
                    let mint_msg = Cw2981ExecuteMsg::Mint {
                        token_id: (collection.next_token.u128() + order).to_string(),
                        owner: recipient.to_string(),
                        token_uri: None,
                        extension: extension.clone(),
                    };

                    //call cw721 contract
                    let callback = WasmMsg::Execute {
                        contract_addr: msg.collection.clone(),
                        msg: to_json_binary(&mint_msg)?,
                        funds: vec![],
                    };

                    response = response.add_message(callback.clone());
                }
            } else {
                //call collection contract
                let selector = &Keccak256::digest(b"mint(address,uint256)")[0..4];

                let account_padded = pad_address_to_bytes32(recipient_evm_address.as_str())?;

                for order in 0..msg.amount.u128() {
                    let mut token_id_bytes = [0u8; 32];
                    (collection.next_token.u128() + order)
                        .to_be_bytes()
                        .iter()
                        .enumerate()
                        .for_each(|(i, &b)| {
                            token_id_bytes[32 - 16 + i] = b; //pad token id to 32 bytes
                        });

                    let data = [&selector[..], &account_padded[..], &token_id_bytes[..]].concat();
                    let data_base64 = BASE64.encode(&data);

                    response = response.add_message(EvmMsg::CallEvm {
                        to: collection.collection_address.clone().unwrap(),
                        value: Uint128::zero(),
                        data: data_base64.clone(),
                    });
                }
            }
            // save states
            for i in 0..msg.amount.u128() {
                self.mint_logs.save(
                    deps.storage,
                    create_mint_log_key(
                        &msg.collection,
                        &(collection.next_token.u128() + i).to_string()
                    ),
                    &recipient.to_string()
                )?;

                mint_info.mints.push(collection.next_token + Uint128::from(i));
            }

            self.mint_info.save(deps.storage, mint_info_key, &mint_info)?;

            global_mint_info += msg.amount;
            self.global_mint_info.save(deps.storage, global_mint_info_key, &global_mint_info)?;

            attrs.push(Attribute {
                key: "token_ids".to_string(),
                value: (collection.next_token.u128()..collection.next_token.u128() +
                    msg.amount.u128())
                    .map(|i| i.to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
            });
            collection.next_token += msg.amount;
        } else {
            let batch = group.batch_size.clone().unwrap();
            let mint_msg = Cw404ExecuteMsg::Mint {
                recipient: recipient.to_string(),
                amount: batch,
            };

            let callback = WasmMsg::Execute {
                contract_addr: msg.collection.clone(),
                msg: to_json_binary(&mint_msg)?,
                funds: vec![],
            };

            for i in 0..msg.amount.u128() {
                response = response.add_message(callback.clone());

                // save states
                self.mint_logs.save(
                    deps.storage,
                    create_mint_log_key_404(
                        &msg.collection,
                        &group.name,
                        &(global_mint_info.u128() + i).to_string()
                    ),
                    &recipient.to_string()
                )?;

                mint_info.mints.push(batch);
            }

            self.mint_info.save(deps.storage, mint_info_key, &mint_info)?;

            global_mint_info += msg.amount;
            self.global_mint_info.save(deps.storage, global_mint_info_key, &global_mint_info)?;

            attrs.push(Attribute {
                key: "amount".to_string(),
                value: (batch * msg.amount).to_string(),
            });
            collection.next_token += batch * msg.amount;
        }

        // save collection
        self.collections.save(deps.storage, msg.collection.clone(), &collection)?;

        Ok(
            response
                .add_attribute("action", "mint_lighthouse")
                .add_attribute("chain", collection.chain)
                .add_attribute("collection", msg.collection)
                .add_attribute("group", msg.group)
                .add_attribute("recipient", recipient.to_string())
                .add_attribute("mint_amount", msg.amount.to_string())
                .add_attributes(attrs)
        )
    }

    pub fn unfreeze_collection(
        &self,
        deps: DepsMut<EvmQueryWrapper>,
        _env: Env,
        info: MessageInfo,
        msg: UnfreezeCollection
    ) -> Result<Response<EvmMsg>, ContractError> {
        let mut collection = self.collections.load(deps.storage, msg.collection.clone())?;

        if collection.admin != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        collection.frozen = false;
        self.collections.save(deps.storage, msg.collection.clone(), &collection)?;

        if collection.chain == "v1" && collection.collection_type == "721" {
            let execute: CosmosMsg<EvmMsg> = (WasmMsg::Execute {
                contract_addr: msg.collection.clone(),
                msg: to_json_binary(
                    &(Cw2981LHExecuteMsg::Extension { msg: Cw2981LHExecuteExtension::Unfreeze {} })
                )?,
                funds: vec![],
            }).into();

            Ok(
                Response::<EvmMsg>
                    ::new()
                    .add_message(execute)
                    .add_attribute("action", "unfreeze")
                    .add_attribute("collection", msg.collection)
            )
        } else if collection.chain == "v1" && collection.collection_type == "404" {
            let execute: CosmosMsg<EvmMsg> = (WasmMsg::Execute {
                contract_addr: msg.collection.clone(),
                msg: to_json_binary(&(Cw404ExecuteMsg::Unfreeze {}))?,
                funds: vec![],
            }).into();

            Ok(
                Response::<EvmMsg>
                    ::new()
                    .add_message(execute)
                    .add_attribute("action", "unfreeze")
                    .add_attribute("collection", msg.collection)
            )
        } else {
            let selector = &Keccak256::digest(b"unfreeze()")[0..4];
            let data_base64 = BASE64.encode(selector);

            let execute = EvmMsg::CallEvm {
                to: collection.collection_address.clone().unwrap(),
                value: Uint128::zero(),
                data: data_base64,
            };

            Ok(
                Response::<EvmMsg>
                    ::new()
                    .add_message(execute)
                    .add_attribute("action", "unfreeze")
                    .add_attribute("collection", msg.collection)
            )
        }
    }

    pub fn reveal_collection_metadata(
        &self,
        deps: DepsMut<EvmQueryWrapper>,
        _env: Env,
        info: MessageInfo,
        msg: RevealCollectionMetadata
    ) -> Result<Response<EvmMsg>, ContractError> {
        let collection = self.collections.load(deps.storage, msg.collection.clone())?;

        if collection.admin != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        if collection.chain == "v1" {
            let execute: CosmosMsg<EvmMsg> = (WasmMsg::Execute {
                contract_addr: msg.collection.clone(),
                msg: to_json_binary(
                    &(Cw2981LHExecuteMsg::Extension { msg: Cw2981LHExecuteExtension::Reveal {} })
                )?,
                funds: vec![],
            }).into();

            Ok(
                Response::<EvmMsg>
                    ::new()
                    .add_message(execute)
                    .add_attribute("action", "reveal")
                    .add_attribute("collection", msg.collection)
            )
        } else {
            let selector = &Keccak256::digest(b"reveal()")[0..4];
            let data_base64 = BASE64.encode(selector);

            let execute = EvmMsg::CallEvm {
                to: collection.collection_address.clone().unwrap(),
                value: Uint128::zero(),
                data: data_base64,
            };

            Ok(
                Response::<EvmMsg>
                    ::new()
                    .add_message(execute)
                    .add_attribute("action", "reveal")
                    .add_attribute("collection", msg.collection)
            )
        }
    }

    pub fn update_admin(
        &self,
        deps: DepsMut<EvmQueryWrapper>,
        _env: Env,
        info: MessageInfo,
        msg: UpdateAdmin
    ) -> Result<Response<EvmMsg>, ContractError> {
        let mut collection = self.collections.load(deps.storage, msg.collection.clone())?;

        if collection.admin != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        collection.admin = msg.admin.clone();

        self.collections.save(deps.storage, msg.collection.clone(), &collection)?;

        Ok(
            Response::new()
                .add_attribute("action", "update_admin")
                .add_attribute("collection", msg.collection)
                .add_attribute("new_admin", msg.admin)
        )
    }

    pub fn add_partner(
        &self,
        deps: DepsMut<EvmQueryWrapper>,
        _env: Env,
        info: MessageInfo,
        msg: AddPartner
    ) -> Result<Response<EvmMsg>, ContractError> {
        let config = self.config.load(deps.storage)?;

        if config.admin != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        if msg.percent > Uint128::from(99u128) {
            return Err(ContractError::InvalidFeePercent {});
        }

        self.partners.save(
            deps.storage,
            msg.address.to_string(),
            &(Partner {
                partner: msg.address.clone(),
                fee_percent: msg.percent,
            })
        )?;

        Ok(
            Response::new()
                .add_attribute("action", "add_partner")
                .add_attribute("address", msg.address)
                .add_attribute("percent", msg.percent)
        )
    }

    pub fn update_nft_contract_admin(
        &self,
        deps: DepsMut<EvmQueryWrapper>,
        _env: Env,
        info: MessageInfo,
        msg: UpdateNftContractAdmin
    ) -> Result<Response<EvmMsg>, ContractError> {
        let collection = self.collections.load(deps.storage, msg.collection.clone())?;

        if collection.admin != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        if collection.chain == "v1" {
            let execute: CosmosMsg<EvmMsg> = (WasmMsg::UpdateAdmin {
                contract_addr: msg.collection.clone(),
                admin: msg.new_admin.clone(),
            }).into();

            Ok(
                Response::<EvmMsg>
                    ::new()
                    .add_message(execute)
                    .add_attribute("action", "update_nft_contract_admin")
                    .add_attribute("collection", msg.collection)
                    .add_attribute("new_admin", msg.new_admin)
            )
        } else {
            let selector = &Keccak256::digest(b"updateMinter(address)")[0..4];
            let mut new_admin = msg.new_admin.clone();

            //if new minter starts with "sei" then convert it to evm address
            if new_admin.starts_with("sei") {
                let querier = EvmQuerier::new(&deps.querier);
                let new_admin_query = querier.query_evm_address(new_admin)?;
                if !new_admin_query.associated {
                    return Err(ContractError::NotAssociatedAddress {});
                }
                new_admin = new_admin_query.evm_address;
            }

            let account_padded = pad_address_to_bytes32(&new_admin)?;

            let data = [&selector[..], &account_padded[..]].concat();
            let data_base64 = BASE64.encode(&data);

            let execute = EvmMsg::CallEvm {
                to: collection.collection_address.clone().unwrap(),
                value: Uint128::zero(),
                data: data_base64,
            };

            Ok(
                Response::<EvmMsg>
                    ::new()
                    .add_message(execute)
                    .add_attribute("action", "update_nft_contract_admin")
                    .add_attribute("collection", msg.collection)
                    .add_attribute("new_minter", new_admin)
            )
        }
    }

    pub fn update_nft_contract_cw_ownable_owner(
        &self,
        deps: DepsMut<EvmQueryWrapper>,
        _env: Env,
        info: MessageInfo,
        msg: UpdateNftContractCwOwnableOwner
    ) -> Result<Response<EvmMsg>, ContractError> {
        let collection = self.collections.load(deps.storage, msg.collection.clone())?;

        if collection.admin != info.sender {
            return Err(ContractError::Unauthorized {});
        }

        if collection.chain == "v2" {
            return Err(ContractError::NotAvailableForEVM {});
        }

        if collection.collection_type == "721" {
            let execute: CosmosMsg<EvmMsg> = (WasmMsg::Execute {
                contract_addr: msg.collection.clone(),
                msg: to_json_binary(
                    &Cw2981ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                        new_owner: msg.new_admin.clone(),
                        expiry: None,
                    })
                )?,
                funds: vec![],
            }).into();

            Ok(
                Response::<EvmMsg>
                    ::new()
                    .add_message(execute)
                    .add_attribute("action", "update_nft_contract_cw_ownable_owner")
                    .add_attribute("collection", msg.collection)
                    .add_attribute("new_admin", msg.new_admin)
            )
        } else {
            //404
            let execute: CosmosMsg<EvmMsg> = (WasmMsg::Execute {
                contract_addr: msg.collection.clone(),
                msg: to_json_binary(
                    &(Cw404ExecuteMsg::TransferOwnership {
                        new_admin: msg.new_admin.clone(),
                    })
                )?,
                funds: vec![],
            }).into();

            Ok(
                Response::<EvmMsg>
                    ::new()
                    .add_message(execute)
                    .add_attribute("action", "update_nft_contract_cw_ownable_owner")
                    .add_attribute("collection", msg.collection)
                    .add_attribute("new_admin", msg.new_admin)
            )
        }
    }

   
}
