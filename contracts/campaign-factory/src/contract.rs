use crate::error::ContractError;
use crate::state::{Config, ConfigResponse, FactoryCampaignInfo, CONFIG, Info, CampaignItemInfo};
use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    state::{NUMBER_OF_CAMPAIGNS, CAMPAIGNS},
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QuerierWrapper,
    QueryRequest, Reply, ReplyOn, Response, StdResult, SubMsg, Uint128, WasmMsg, WasmQuery, 
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;
use campaign::msg::InstantiateMsg as CampaignInstantiateMsg;
use campaign::msg::QueryMsg as CampaignQueryMsg;
use campaign::state::{CampaignInfo, RewardTokenInfo};

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

    // init NUMBER_OF_POOLS to 0
    NUMBER_OF_CAMPAIGNS.save(deps.storage, &0u64)?;

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
            allowed_collection,
            reward_token_info,
            reward_per_second,
            staking_duration,
            start_time,
            end_time,
        } => execute_create_campaign(
            deps,
            env,
            info,
            owner,
            allowed_collection,
            reward_token_info,
            reward_per_second,
            staking_duration,
            start_time,
            end_time,
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
    allowed_collection: Addr,
    reward_token_info: RewardTokenInfo,
    reward_per_second:Uint128,
    staking_duration: u64,
    start_time: Uint128,
    end_time: Uint128,
) -> Result<Response, ContractError> {
    let config: Config = CONFIG.load(deps.storage)?;

    // permission check
    if info.sender != config.owner {
        return Err(ContractError::Unauthorized {});
    }

    // validate address format
    let _ = deps.api.addr_validate(&owner)?;

    Ok(Response::new()
        .add_attributes(vec![
            ("action", "create_pool"),
            ("campaign_factory_owner", info.sender.as_str()),
            ("campaign_owner", owner.to_string().as_str()),
            ("allowed_collection", allowed_collection.to_string().as_str()),
            ("reward_token_info", &format!("{}", reward_token_info)),
            ("reward_per_second", reward_per_second.to_string().as_str()),
            ("staking_duration", staking_duration.to_string().as_str()),
            ("start_time", start_time.to_string().as_str()),
            ("end_time", end_time.to_string().as_str()),
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
                    owner,
                    allowed_collection:allowed_collection.clone().to_string(),
                    reward_token_info,
                    staking_duration,
                    reward_per_second: Uint128::zero(),
                    start_time,
                    end_time,
                })?,
            }),
            reply_on: ReplyOn::Success,
        }))
}

/// This just stores the result for future query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply, info: MessageInfo) -> StdResult<Response> {
    let reply = parse_reply_instantiate_data(msg).unwrap();

    let campaign_contract = &reply.contract_address;
    let campaign_info: CampaignInfo = query_pair_info_from_pair(&deps.querier, Addr::unchecked(campaign_contract))?;

    let campaign_key = NUMBER_OF_CAMPAIGNS.load(deps.storage)? + 1;
    
    // let factory_info = Info{
    //     owner: campaign_info.owner.clone(),
    //     allowed_collection: campaign_info.allowed_collection.clone(),
    //     reward_token_info: campaign_info.reward_token_info.clone(),
    //     reward_per_second: campaign_info.reward_per_second,
    //     staking_duration: campaign_info.staking_duration,
    //     start_time: campaign_info.start_time,
    //     end_time: campaign_info.end_time,
    // };

    if CAMPAIGNS.may_load(deps.storage, info.sender.clone())?.is_none(){
        let mut campaign_list: Vec<CampaignItemInfo> = vec![];
        let factory_info = Info{
            owner: campaign_info.owner.clone(),
            allowed_collection: campaign_info.allowed_collection.clone(),
            reward_token_info: campaign_info.reward_token_info.clone(),
            reward_per_second: campaign_info.reward_per_second,
            staking_duration: campaign_info.staking_duration,
            start_time: campaign_info.start_time,
            end_time: campaign_info.end_time,
        };
        campaign_list.push(CampaignItemInfo { campaign_addr: deps.api.addr_validate(campaign_contract)?, campaign_info: factory_info });

        CAMPAIGNS.save(deps.storage, info.sender.clone(), &FactoryCampaignInfo { campaigns: campaign_list })?;
    }else{
        let mut campaigns: FactoryCampaignInfo = CAMPAIGNS.load(deps.storage,info.sender.clone())?;
 
        let factory_info = Info{
            owner: campaign_info.owner.clone(),
            allowed_collection: campaign_info.allowed_collection.clone(),
            reward_token_info: campaign_info.reward_token_info.clone(),
            reward_per_second: campaign_info.reward_per_second,
            staking_duration: campaign_info.staking_duration,
            start_time: campaign_info.start_time,
            end_time: campaign_info.end_time,
        };

        campaigns.campaigns.push(CampaignItemInfo { campaign_addr: deps.api.addr_validate(campaign_contract)?, campaign_info: factory_info });


        CAMPAIGNS.save(deps.storage, info.sender.clone(), &campaigns)?;
    }

    // CAMPAIGNS.save(
    //     deps.storage,
    //     campaign_key,
    //     &factory_info,
    // )?;

    // increase pool count
    NUMBER_OF_CAMPAIGNS.save(deps.storage, &(&campaign_key))?;


    Ok(Response::new().add_attributes([
        ("action", "reply_on_create_pool_success"),
        ("campaign_id", campaign_key.to_string().as_str()),
        ("campgain_contract_addr", campaign_contract),
        ("owner", &campaign_info.owner.to_string()),
        ("allowed_collection", &campaign_info.allowed_collection.clone().to_string()),
        ("reward_token_info", &format!("{}", &campaign_info.reward_token_info)),
        ("reward_per_second", &campaign_info.reward_per_second.to_string()),
        ("staking_duration", &campaign_info.staking_duration.to_string()),
        ("start_time", &campaign_info.start_time.to_string()),
        ("end_time", &campaign_info.end_time.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Campaign { campaign_owner } => to_binary(&query_campaign_info(deps, campaign_owner)?),
        // QueryMsg::Campaigns { campaign_owner,start_after, limit } => {
        //     to_binary(&query_campaigns(deps,start_after, limit)?)
        // }
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

pub fn query_campaign_info(deps: Deps, campaign_owner: Addr) -> StdResult<FactoryCampaignInfo> {
    CAMPAIGNS.load(deps.storage, campaign_owner)
}

// pub fn query_campaigns(
//     deps: Deps,
//     start_after: Option<u64>,
//     limit: Option<u32>,
// ) -> StdResult<Vec<FactoryCampaignInfo>> {
//     let start_after = start_after.unwrap_or(0);
//     let limit = limit.unwrap_or(30) as usize;
//     let campaign_count = NUMBER_OF_CAMPAIGNS.load(deps.storage)?;

//     let pools = (start_after..campaign_count)
//         .map(|campaign_id| CAMPAIGNS.load(deps.storage, campaign_id + 1))
//         .take(limit)
//         .collect::<StdResult<Vec<_>>>()?;

//     Ok(pools)
// }

fn query_pair_info_from_pair(querier: &QuerierWrapper, pair_contract: Addr) -> StdResult<CampaignInfo> {
    let pair_info: CampaignInfo = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pair_contract.to_string(),
        msg: to_binary(&CampaignQueryMsg::Campaign {})?,
    }))?;

    Ok(pair_info)
}
