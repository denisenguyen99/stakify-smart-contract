#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, to_binary, Uint128};
use cw2::set_contract_version;
// use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{CAMPAIGN_INFO, CampaignInfo};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:campaign";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(_deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let campaign = &CampaignInfo {
        collection: _deps.api.addr_validate(&_msg.collection)?.to_string(),
        reward_token: _msg.reward_token.clone(),
        reward_per_second: Uint128::zero(),
        start_time: _msg.start_time,
        end_time: _msg.end_time,
    };

    CAMPAIGN_INFO.save(_deps.storage, campaign)?;

    Ok(Response::new().add_attributes([
        ("action", "instantiate"),
        ("start_time", &_msg.start_time.to_string()),
        ("end_time", &_msg.end_time.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Campaign {} => Ok(to_binary(&query_campaign_info(deps)?)?),
    }
}

fn query_campaign_info(deps: Deps) -> Result<CampaignInfo, ContractError> {
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
    Ok(campaign_info)
}

#[cfg(test)]
mod tests {}
