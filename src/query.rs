use cosmwasm_std::{ to_json_binary, Binary, Deps, StdResult };

use crate::{
    helpers::{
        create_global_mint_info_key,
        create_group_key,
        create_mint_log_key,
    },
    msg::{ EvmQueryWrapper, QueryMsg },
    querier::EvmQuerier,
    state::{ Lighthouse, MintInfo },
};

impl<'a> Lighthouse<'a> {
    pub fn query(&self, deps: Deps<EvmQueryWrapper>, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::GetConfig {} => self.query_get_config(deps),
            QueryMsg::GetCollection { collection } => self.get_collection(deps, collection),
            QueryMsg::MintsOf { address, collection } => self.mints_of(deps, address, collection),
            QueryMsg::GetMinterOf { collection, token_id } =>
                self.get_minter_of(deps, collection, token_id),
            QueryMsg::GetGlobalMintInfo { collection, group_name } =>
                self.get_global_mint_info(deps, collection, group_name),
            QueryMsg::GetEvmAddressOfBech32Address { address } =>
                self.get_evm_address_of_bech32_address(deps, address),
            QueryMsg::GetBech32AddressOfEvmAddress { address } =>
                self.get_bech32_address_of_evm_address(deps, address),
            /* QueryMsg::GetGatedMintsOf { collection, group_name, contract_address, token_ids } =>
                self.get_gated_mints_of(deps, collection, group_name, contract_address, token_ids),*/
        }
    }

    pub fn query_get_config(&self, deps: Deps<EvmQueryWrapper>) -> StdResult<Binary> {
        let config = self.config.load(deps.storage)?;
        to_json_binary(&config)
    }

    pub fn get_collection(
        &self,
        deps: Deps<EvmQueryWrapper>,
        collection: String
    ) -> StdResult<Binary> {
        let collection = self.collections.load(deps.storage, collection)?;
        to_json_binary(&collection)
    }

    pub fn mints_of(
        &self,
        deps: Deps<EvmQueryWrapper>,
        address: String,
        collection: String
    ) -> StdResult<Binary> {
        let collection_info = self.collections.load(deps.storage, collection.clone())?;
        let mut mints = vec![];
        for group in collection_info.mint_groups {
            let key = create_group_key(&address, &collection, &group.name);
            let mint_info = self.mint_info.load(deps.storage, key).unwrap_or(MintInfo {
                mints: vec![],
            });
            mints.extend(mint_info.mints);
        }

        to_json_binary(&(MintInfo { mints }))
    }

    pub fn get_minter_of(
        &self,
        deps: Deps<EvmQueryWrapper>,
        collection: String,
        token_id: String
    ) -> StdResult<Binary> {
        let key = create_mint_log_key(&collection, &token_id);
        let minter = self.mint_logs.load(deps.storage, key)?;
        to_json_binary(&minter)
    }

    pub fn get_global_mint_info(
        &self,
        deps: Deps<EvmQueryWrapper>,
        collection: String,
        group_name: String
    ) -> StdResult<Binary> {
        let key = create_global_mint_info_key(&collection, &group_name);
        let minted = self.global_mint_info.load(deps.storage, key).unwrap_or_default();
        to_json_binary(&minted)
    }

    pub fn get_evm_address_of_bech32_address(
        &self,
        deps: Deps<EvmQueryWrapper>,
        address: String
    ) -> StdResult<Binary> {
        let querier: EvmQuerier<'_> = EvmQuerier::new(&deps.querier);
        let evm_address = querier.query_evm_address(address)?;

        to_json_binary(&evm_address)
    }

    pub fn get_bech32_address_of_evm_address(
        &self,
        deps: Deps<EvmQueryWrapper>,
        address: String
    ) -> StdResult<Binary> {
        let querier: EvmQuerier<'_> = EvmQuerier::new(&deps.querier);
        let bech32_address = querier.query_bech32_address(address)?;

        to_json_binary(&bech32_address)
    }

    /*pub fn get_gated_mints_of(
        &self,
        deps: Deps,
        collection: String,
        group_name: String,
        contract_address: String,
        token_ids: Vec<String>
    ) -> StdResult<Binary> {
        let mut mints = vec![];
        for token_id in token_ids {
            let key = create_gated_mint_log_key(
                &collection,
                &contract_address,
                &group_name,
                &token_id
            );
            let minted = self.gated_mint_info.load(deps.storage, key).is_ok();
            mints.push(GatedQueryResponse {
                token_id,
                minted,
            });
        }

        to_json_binary(
            &(GatedQueryResponse { token_id: group_name, minted: mints.iter().all(|m| m.minted) })
        )
    }*/
}
