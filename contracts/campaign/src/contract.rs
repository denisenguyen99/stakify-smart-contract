#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, SubMsg, to_binary, Uint128, WasmMsg};
use cw2::set_contract_version;
use cw20::Cw20ExecuteMsg;
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
        owner: _deps.api.addr_validate(&_msg.owner)?.to_string(),
        collection: _deps.api.addr_validate(&_msg.collection)?.to_string(),
        reward_per_second: Uint128::zero(),
        start_time: _msg.start_time,
        end_time: _msg.end_time,
        reward_token_amount: _msg.reward_token_amount,
        reward_token_available: Uint128::zero(),
        reward_token_address: _msg.reward_token_address.to_string(),
    };

    CAMPAIGN_INFO.save(_deps.storage, campaign)?;

    Ok(Response::new().add_attributes([
        ("action", "instantiate"),
        ("collection", &_msg.collection.to_string()),
        ("reward_token_address", &_msg.reward_token_address.to_string()),
        ("reward_token_amount", &_msg.reward_token_amount.to_string()),
        ("start_time", &_msg.start_time.to_string()),
        ("end_time", &_msg.end_time.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddRewardToken {} => execute_add_reward_balance(deps, env, info),
    }
}

pub fn execute_add_reward_balance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo
) -> Result<Response, ContractError> {
    let mut campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    if campaign_info.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut res = Response::new();

    let transfer = SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: campaign_info.reward_token_address,
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: env.contract.address.clone().to_string(),
            amount: campaign_info.reward_token_amount,
        })?,
        funds: vec![],
    }));

    res = res.add_submessages(transfer);

    campaign_info.reward_token_available = campaign_info.reward_token_amount;

    CAMPAIGN_INFO.save(deps.storage, &campaign_info)?;

    Ok(res)
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
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage, mock_info
    };

    #[test]
    fn test_instantiate() {
        // Tạo các đối tượng giả lập cho môi trường thử nghiệm
        let api = MockApi::default();
        let storage = MockStorage::new();
        let querier = MockQuerier::new(&[]);
        let mut deps = mock_dependencies();

        // Tạo một môi trường thử nghiệm giả lập
        let env = mock_env();

        // Gọi hàm trong smart contract và lấy kết quả

        let result = instantiate(
            deps.as_mut(),
            env, mock_info("tai", &[]),
            InstantiateMsg{
                collection: "".to_string(),
                reward_token_address: "".to_string(),
                reward_token_amount: Default::default(),
                reward_token_available: Default::default(),
                start_time: 0,
                end_time: 0,
                owner: "".to_string(),
            }
        ).unwrap();

        // Kiểm tra kết quả hợp lệ
        assert_eq!(result, true);
        // Nếu cần, kiểm tra giá trị trả về
        // assert_eq!(result.unwrap().value, expected_value);
    }
}

