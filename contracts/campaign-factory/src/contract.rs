use crate::error::ContractError;
use crate::state::{
    CampaignInfoResponse, Config, ConfigResponse, StakedCampaign, ADDR_CAMPAIGNS, CONFIG,
};
use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::CAMPAIGNS,
};
use campaign::msg::ExecuteMsg as CampaignExecuteMsg;
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
        ExecuteMsg::UpdateCampaign {
            contract_addr,
            campaign_name,
            campaign_image,
            campaign_description,
            start_time,
            end_time,
            limit_per_staker,
            reward_token_info,
            allowed_collection,
            lockup_term,
        } => execute_update_campaign(
            deps,
            env,
            info,
            contract_addr,
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
            ("campaign_owner", info.sender.clone().to_string().as_str()),
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
                    owner: info.sender.clone().to_string(),
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

// Anyone can execute it to create a new pool
#[allow(clippy::too_many_arguments)]
pub fn execute_update_campaign(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    contract_addr: String,
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
    let campaign = CAMPAIGNS.load(deps.storage, deps.api.addr_validate(&contract_addr)?)?;

    // permission check
    if campaign.owner != info.sender.clone() {
        return Err(ContractError::Unauthorized {});
    }

    Ok(Response::new()
        .add_attributes(vec![
            ("action", "create_campaign"),
            ("campaign_owner", info.sender.clone().to_string().as_str()),
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
            msg: CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr,
                msg: to_binary(&CampaignExecuteMsg::UpdateCampaign {
                    campaign_name,
                    campaign_image,
                    campaign_description,
                    limit_per_staker,
                    allowed_collection,
                    lockup_term,
                    start_time,
                    end_time,
                })?,
                funds: vec![],
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

    let addr_campaigns = ADDR_CAMPAIGNS.load(deps.storage)?;

    let contract_addr = deps.api.addr_validate(&campaign_contract)?;

    let campaign_info = StakedCampaign {
        owner: campaign_info.owner.clone(),
        campaign_addr: contract_addr.clone(),
        campaign_name: campaign_info.campaign_name.clone(),
        campaign_image: campaign_info.campaign_image.clone(),
        campaign_description: campaign_info.campaign_description.clone(),
        num_tokens: campaign_info.num_tokens.clone(),
        limit_per_staker: campaign_info.limit_per_staker.clone(),
        reward_token_info: campaign_info.reward_token_info.clone(),
        allowed_collection: campaign_info.allowed_collection.clone(),
        lockup_term: campaign_info.lockup_term.clone(),
        start_time: campaign_info.start_time.clone(),
        end_time: campaign_info.end_time.clone(),
    };
    CAMPAIGNS.save(deps.storage, contract_addr, &campaign_info)?;

    if !addr_campaigns.contains(&campaign_contract) {
        let mut addr_campaigns = ADDR_CAMPAIGNS.load(deps.storage)?;
        addr_campaigns.push(campaign_contract.clone());
        ADDR_CAMPAIGNS.save(deps.storage, &addr_campaigns)?;
    }

    Ok(Response::new().add_attributes([
        ("action", "reply_on_create_campaign_success"),
        ("campaign_contract_addr", &campaign_contract),
        ("owner", &campaign_info.owner.to_string()),
        (
            "allowed_collection",
            &campaign_info.allowed_collection.clone().to_string(),
        ),
        (
            "reward_token_info",
            &format!("{:?}", &campaign_info.reward_token_info),
        ),
        ("campaign_name", &campaign_info.campaign_name),
        ("campaign_image", &campaign_info.campaign_image),
        ("campaign_description", &campaign_info.campaign_description),
        (
            "limit_per_staker",
            &campaign_info.limit_per_staker.to_string(),
        ),
        ("start_time", &campaign_info.start_time.to_string()),
        ("end_time", &campaign_info.end_time.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Campaign { campaign_addr } => {
            to_binary(&query_campaigns_info(deps, campaign_addr)?)
        }
        QueryMsg::Campaigns { limit } => to_binary(&query_campaigns(deps, limit)?),
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

pub fn query_campaigns_info(deps: Deps, campaign_addr: Addr) -> StdResult<StakedCampaign> {
    let campaign_info = CAMPAIGNS.load(deps.storage, campaign_addr)?;
    Ok(campaign_info)
}

pub fn query_campaigns(deps: Deps, limit: Option<u32>) -> StdResult<Vec<CampaignInfoResponse>> {
    let limit = limit.unwrap_or(30) as usize;

    let addr_campaigns = ADDR_CAMPAIGNS.load(deps.storage)?;

    // let campaigns = addr_campaigns.iter()
    //     .map(|addr| CAMPAIGNS.load(deps.storage, deps.api.addr_validate(addr)?))
    //     .take(limit)
    //     .collect::<StdResult<Vec<_>>>()?;

    // real time
    let mut campaigns: Vec<CampaignInfoResponse> = vec![];
    for addr in addr_campaigns.iter().take(limit) {
        let total_staked = query_total_staked(&deps.querier, deps.api.addr_validate(addr)?)?;
        let reward_per_second = query_reward_per_second_campaign(&deps.querier, deps.api.addr_validate(addr)?)?;

        let campaign_info = CAMPAIGNS.load(deps.storage, deps.api.addr_validate(addr)?)?;
        campaigns.push(CampaignInfoResponse {
            owner: campaign_info.owner,
            campaign_addr: addr.to_string(),
            campaign_name: campaign_info.campaign_name,
            campaign_description: campaign_info.campaign_description,
            campaign_image: campaign_info.campaign_image,
            reward_per_second,
            total_nft: total_staked,
            num_tokens: campaign_info.num_tokens,
            limit_per_staker: campaign_info.limit_per_staker,
            reward_token_info: campaign_info.reward_token_info,
            allowed_collection: campaign_info.allowed_collection,
            lockup_term: campaign_info.lockup_term,
            start_time: campaign_info.start_time,
            end_time: campaign_info.end_time,
        });
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