use data_encoding:: BASE64 ;
use sha3::{ Digest, Keccak256 };
use cosmwasm_std::{
    from_json, to_json_vec, Binary, ContractResult, QuerierWrapper, QueryRequest, StdError, StdResult, SystemResult
};
use crate::{ helpers::pad_address_to_bytes32, msg::{ EvmQueryWrapper, Route }, structs::{EvmQuery, GetEvmAddressResponse, GetSeiAddressResponse} };

pub struct EvmQuerier<'a> {
    querier: &'a QuerierWrapper<'a, EvmQueryWrapper>,
}

impl<'a> EvmQuerier<'a> {
    pub fn new(querier: &'a QuerierWrapper<EvmQueryWrapper>) -> Self {
        EvmQuerier { querier }
    }

    pub fn query_has_role(
        &self,
        lighthouse_contract: String,
        address: String,
        target_evm_contract: String,
        role: Vec<u8>
    ) -> StdResult<bool> {
        let selector = &Keccak256::digest(b"hasRole(bytes32,address)")[0..4];

        let account_padded = pad_address_to_bytes32(
            &address
        )?;

        let data = [&selector[..], &role[..], &account_padded[..]].concat();
        let data_base64 = BASE64.encode(&data);

        let request: QueryRequest<EvmQueryWrapper> = (EvmQueryWrapper {
            route: Route::Evm,
            query_data: EvmQuery::StaticCall {
                from: lighthouse_contract,
                to: target_evm_contract,
                data: data_base64,
            },
        }).into();

        Ok(self.query(&request)? != Binary::from(vec![0u8; 32]))
    }

    /*pub fn query_address(
        &self,
        bech32_address: String,
        evm_helper: String
    ) -> StdResult<String> {
        let selector = &Keccak256::digest(b"myAddress()")[0..4];
        let data_base64 = BASE64.encode(selector);
    
        let request: QueryRequest<EvmQueryWrapper> = EvmQueryWrapper {
            route: Route::Evm,
            query_data: EvmQuery::StaticCall {
                from: bech32_address,
                to: evm_helper,
                data: data_base64,
            },
        }.into();
    
        let res = self.query(&request)?;
        //let address_bytes = &res[12..32];
        //let address_hex = HEXUPPER.encode(address_bytes);
        /*let hex_str = HEXUPPER.encode(res.as_slice());
        let address_hex = hex_str.chars().rev().take(40).collect::<String>().chars().rev().collect::<String>();*/
        let res_binary_str: String = res.iter()
        .map(|byte| format!("{:08b}", byte))
        .collect::<Vec<String>>()
        .join("");
        Ok(res_binary_str)
    }*/

    pub fn query_evm_address(
        &self,
        bech32_address: String,
    ) -> StdResult<GetEvmAddressResponse> {
    
        let request: QueryRequest<EvmQueryWrapper> = EvmQueryWrapper {
            route: Route::Evm,
            query_data: EvmQuery::GetEvmAddress { 
                sei_address: bech32_address
            }
        }.into();
    
        let res = self.query(&request)?;
        Ok(from_json(&res)?)
    }

    pub fn query_bech32_address(
        &self,
        evm_address: String,
    ) -> StdResult<GetSeiAddressResponse> {
    
        let request: QueryRequest<EvmQueryWrapper> = EvmQueryWrapper {
            route: Route::Evm,
            query_data: EvmQuery::GetSeiAddress{
                evm_address
            }
        }.into();
    
        let res = self.query(&request)?;
        Ok(from_json(&res)?)
    }

    pub fn query(&self, request: &QueryRequest<EvmQueryWrapper>) -> StdResult<Binary> {
        let raw = to_json_vec(request).map_err(|serialize_err| {
            StdError::generic_err(format!("Serializing QueryRequest: {serialize_err}"))
        })?;

        match self.querier.raw_query(&raw) {
            SystemResult::Err(system_err) =>
                Err(
                    cosmwasm_std::StdError::generic_err(
                        format!("Querier system error: {system_err}")
                    )
                ),
            SystemResult::Ok(ContractResult::Err(contract_err)) =>
                Err(
                    cosmwasm_std::StdError::generic_err(
                        format!("Querier contract error: {contract_err}")
                    )
                ),
            SystemResult::Ok(ContractResult::Ok(binary_value)) => Ok(binary_value),
        }
    }
}

