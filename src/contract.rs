#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{ Binary, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response, StdResult };
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ EvmQueryWrapper, ExecuteMsg, InstantiateMsg, QueryMsg };
use crate::state::Lighthouse;
use crate::structs::EvmMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lighthouse";
const CONTRACT_VERSION: &str = "2.0.0";

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let tract = Lighthouse::default();
    tract.instantiate(deps, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<EvmQueryWrapper>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg
) -> Result<Response<EvmMsg>, ContractError> {
    let tract = Lighthouse::default();
    tract.execute(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    let tract = Lighthouse::default();
    tract.reply(deps, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<EvmQueryWrapper>, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let tract = Lighthouse::default();
    tract.query(deps, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}
