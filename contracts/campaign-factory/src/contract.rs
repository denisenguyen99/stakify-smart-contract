use crate::error::ContractError;
use crate::state::{
     Config, ConfigResponse, StakedCampaign, ADDR_CAMPAIGNS, CONFIG,
};
use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::CAMPAIGNS,
};
// use campaign::msg::ExecuteMsg as CampaignExecuteMsg;
use campaign::msg::InstantiateMsg as CampaignInstantiateMsg;
use campaign::msg::QueryMsg as CampaignQueryMsg;
use campaign::state::{AssetTokenInfo, CampaignInfoResult, LockupTerm};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QuerierWrapper,
    QueryRequest, Reply, ReplyOn, Response, StdResult, SubMsg, WasmMsg, WasmQuery, Uint128,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:campaign-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender,
        campaign_code_id: msg.campaign_code_id,
    };

    // init ADDR_CAMPAIGNS to vec![]
    ADDR_CAMPAIGNS.save(deps.storage, &vec![])?;

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            campaign_code_id,
        } => execute_update_config(deps, env, info, owner, campaign_code_id),
        ExecuteMsg::CreateCampaign {
            owner,
            campaign_name,
            campaign_image,
            campaign_description,
            start_time,
            end_time,
            limit_per_staker,
            reward_token_info,
            allowed_collection,
            lockup_term,
        } => execute_create_campaign(
            deps,
            env,
            info,
            owner,
            campaign_name,
            campaign_image,
            campaign_description,
            start_time,
            end_time,
            limit_per_staker,
            reward_token_info,
            allowed_collection,
            lockup_term,
        ),
    }
}

// Only owner can execute it
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: Option<String>,
    campaign_code_id: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    // permission check
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // update owner if provided
    if let Some(owner) = owner {
        config.owner = deps.api.addr_validate(&owner)?;
    }

    // update campgaign_code_id if provided
    if let Some(campaign_code_id) = campaign_code_id {
        config.campaign_code_id = campaign_code_id;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

// Anyone can execute it to create a new pool
#[allow(clippy::too_many_arguments)]
pub fn execute_create_campaign(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: String,
    campaign_name: String,
    campaign_image: String,
    campaign_description: String,
    start_time: u64,
    end_time: u64,
    limit_per_staker: u64,
    reward_token_info: AssetTokenInfo,
    allowed_collection: String,
    lockup_term: Vec<LockupTerm>,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    // validate address format
    // let _ = deps.api.addr_validate(&owner)?;

    Ok(Response::new()
        .add_attributes(vec![
            ("action", "create_campaign"),
            ("campaign_owner", owner.clone().to_string().as_str()),
            ("campaign_name", campaign_name.to_string().as_str()),
            ("campaign_image", campaign_image.to_string().as_str()),
            (
                "campaign_description",
                campaign_description.to_string().as_str(),
            ),
            ("start_time", start_time.to_string().as_str()),
            ("end_time", end_time.to_string().as_str()),
            ("limit_per_staker", limit_per_staker.to_string().as_str()),
            ("reward_token_info", &format!("{}", reward_token_info)),
            (
                "allowed_collection",
                allowed_collection.to_string().as_str(),
            ),
            ("lockup_term", &format!("{:?}", &lockup_term)),
        ])
        .add_submessage(SubMsg {
            id: 1,
            gas_limit: None,
            msg: CosmosMsg::Wasm(WasmMsg::Instantiate {
                code_id: config.campaign_code_id,
                funds: vec![],
                admin: Some(env.contract.address.to_string()),
                label: "pair".to_string(),
                msg: to_binary(&CampaignInstantiateMsg {
                    owner: owner.clone(),
                    campaign_name: campaign_name.clone(),
                    campaign_image: campaign_image.clone(),
                    campaign_description: campaign_description.clone(),
                    limit_per_staker: limit_per_staker.clone(),
                    reward_token_info: reward_token_info.clone(),
                    allowed_collection: allowed_collection.clone().to_string(),
                    lockup_term: lockup_term.clone(),
                    start_time: start_time.clone(),
                    end_time: end_time.clone(),
                })?,
            }),
            reply_on: ReplyOn::Success,
        }))
}


/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    let reply = parse_reply_instantiate_data(msg).unwrap();

    let campaign_contract = reply.contract_address;
    let campaign_info: CampaignInfoResult =
        query_pair_info_from_pair(&deps.querier, Addr::unchecked(&campaign_contract))?;


    let contract_addr = deps.api.addr_validate(&campaign_contract)?;

    let campaign_info = StakedCampaign {
        owner: campaign_info.owner.clone(),
        campaign_addr: contract_addr.clone(),
    };
    CAMPAIGNS.save(deps.storage, contract_addr, &campaign_info)?;

    let mut addr_campaigns = ADDR_CAMPAIGNS.load(deps.storage)?;
    addr_campaigns.push(campaign_contract.clone());
    ADDR_CAMPAIGNS.save(deps.storage, &addr_campaigns)?;

    Ok(Response::new().add_attributes([
        ("action", "reply_on_create_campaign_success"),
        ("campaign_contract_addr", &campaign_contract),
        ("owner", &campaign_info.owner.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Campaign { campaign_addr } => {
            to_binary(&query_campaign_info(deps, campaign_addr)?)
        }
        QueryMsg::Campaigns {owner, limit } => to_binary(&query_campaigns(deps,owner, limit)?),
        QueryMsg::CampaignAddrs {} => to_binary(&query_addr_campaigns(deps)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state: Config = CONFIG.load(deps.storage)?;
    let resp = ConfigResponse {
        owner: state.owner.to_string(),
        campaign_code_id: state.campaign_code_id,
    };

    Ok(resp)
}

pub fn query_addr_campaigns(deps: Deps) -> StdResult<Vec<String>> {
    let addr_campaigns = ADDR_CAMPAIGNS.load(deps.storage)?;
    Ok(addr_campaigns)
}

pub fn query_campaign_info(deps: Deps, campaign_addr: Addr) -> StdResult<StakedCampaign> {
    let campaign_info = CAMPAIGNS.load(deps.storage, campaign_addr)?;
    Ok(campaign_info)
}

pub fn query_campaigns(deps: Deps,owner: Option<Addr>, limit: Option<u32>) -> StdResult<Vec<StakedCampaign>> {
    let addr_campaigns = ADDR_CAMPAIGNS.load(deps.storage)?;
    let limit = limit.unwrap_or(addr_campaigns.len() as u32) as usize;

    let mut campaigns = addr_campaigns.iter()
        .map(|addr| CAMPAIGNS.load(deps.storage, deps.api.addr_validate(addr)?))
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    if let Some(addr) = owner{
        campaigns = campaigns.iter().filter(|&campaign| campaign.owner == addr)
        .cloned()
        .collect::<Vec<_>>();
    }

    Ok(campaigns)
}

fn query_pair_info_from_pair(
    querier: &QuerierWrapper,
    pair_contract: Addr,
) -> StdResult<CampaignInfoResult> {
    let pair_info: CampaignInfoResult = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pair_contract.to_string(),
        msg: to_binary(&CampaignQueryMsg::CampaignInfo {})?,
    }))?;

    Ok(pair_info)
}

pub fn query_total_staked(querier: &QuerierWrapper, contract_addr: Addr) -> StdResult<u64> {
    let total = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&CampaignQueryMsg::TotalStaked {})?,
    }))?;

    Ok(total)
}

pub fn query_reward_per_second_campaign(querier: &QuerierWrapper, contract_addr: Addr) -> StdResult<Uint128> {
    let reward_per_second = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&CampaignQueryMsg::RewardPerSecond {})?,
    }))?;

    Ok(reward_per_second)
}