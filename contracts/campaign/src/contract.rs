#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    has_coins, to_binary, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdResult, Timestamp, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    CampaignInfo, NftInfo, RewardTokenInfo, StakerRewardAssetInfo, CAMPAIGN_INFO, STAKERS_INFO,
};
use cw20::Cw20ExecuteMsg;
use cw721::{Cw721ExecuteMsg, Cw721QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:campaign";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // set version to contract
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // collect campaign info
    let campaign = CampaignInfo {
        owner: deps.api.addr_validate(&msg.owner).unwrap(),
        campaign_name: msg.campaign_name,
        campaign_image: msg.campaign_image,
        campaign_description: msg.campaign_description,
        start_time: Timestamp::from_nanos(msg.start_time.to_string().parse().unwrap()),
        end_time: Timestamp::from_nanos(msg.end_time.to_string().parse().unwrap()),
        total_reward: Uint128::zero(),
        limit_per_staker: msg.limit_per_staker.clone(),
        reward_token_info: msg.reward_token_info.clone(),
        allowed_collection: deps.api.addr_validate(&msg.allowed_collection).unwrap(),
        lockup_term: Uint128::zero(),
        reward_per_second: Uint128::zero(),
    };

    // store campaign info
    CAMPAIGN_INFO.save(deps.storage, &campaign)?;

    // we need emit the information of reward token to response
    let reward_token_info_str: String;

    match msg.reward_token_info {
        RewardTokenInfo::Token { contract_addr } => {
            reward_token_info_str = contract_addr.to_string();
        }
        RewardTokenInfo::NativeToken { denom } => {
            reward_token_info_str = denom;
        }
    }

    // emit the information of instantiated campaign
    Ok(Response::new().add_attributes([
        ("action", "instantiate"),
        ("owner", &msg.owner),
        // ("campaign_name", &msg.campaign_name),
        // ("campaign_image", &msg.campaign_image),
        // ("campaign_description", &msg.campaign_description),
        ("start_time", &msg.start_time.to_string()),
        ("end_time", &msg.end_time.to_string()),
        ("total_reward", &msg.total_reward.to_string()),
        ("limit_per_staker", &msg.limit_per_staker.to_string()),
        ("reward_token_info", &reward_token_info_str),
        ("allowed_collection", &msg.allowed_collection),
        ("reward_per_second", &msg.reward_per_second.to_string()),
        ("lockup_term", &msg.lockup_term.to_string()),
        ("reward_per_second", &msg.reward_per_second.to_string()),
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
        ExecuteMsg::AddRewardToken { amount } => execute_add_reward_token(deps, env, info, amount),
        ExecuteMsg::StakeNfts { token_ids } => execute_stake_nft(deps, env, info, token_ids),
        ExecuteMsg::UnstakeNfts { token_ids } => execute_unstake_nft(deps, env, info, token_ids),
        ExecuteMsg::ClaimReward {} => execute_claim_reward(deps, env, info),
        ExecuteMsg::UpdateCampaign {
            campaign_name,
            campaign_image,
            campaign_description,
            start_time,
            end_time,
            limit_per_staker,
            reward_token_info,
            allowed_collection,
            lockup_term,
            reward_per_second,
        } => execute_update_campaign(
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
            reward_per_second,
        ),
    }
}

pub fn execute_add_reward_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // load campaign info
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    // only owner can add reward token
    if campaign_info.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // TODO: check more condition of adding reward token

    // TODO: update campaign info if necessary

    // we need determine the reward token is native token or cw20 token
    match campaign_info.reward_token_info {
        RewardTokenInfo::Token { contract_addr } => {
            // execute cw20 transfer msg from info.sender to contract
            let transfer_reward: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                    owner: info.sender.to_string(),
                    recipient: env.contract.address.to_string(),
                    amount,
                })?,
                funds: vec![],
            });

            Ok(Response::new()
                .add_message(transfer_reward)
                .add_attributes([
                    ("action", "add_reward_token"),
                    ("owner", campaign_info.owner.as_ref()),
                    ("reward_token_info", contract_addr.as_ref()),
                    ("reward_token_amount", &amount.to_string()),
                    (
                        "reward_per_second",
                        &campaign_info.reward_per_second.to_string(),
                    ),
                    ("start_time", &campaign_info.start_time.to_string()),
                    ("end_time", &campaign_info.end_time.to_string()),
                ]))
        }
        RewardTokenInfo::NativeToken { denom } => {
            // check the amount of native token in funds
            if !has_coins(
                &info.funds,
                &Coin {
                    denom: denom.clone(),
                    amount,
                },
            ) {
                return Err(ContractError::InvalidFunds {});
            }

            Ok(Response::new().add_attributes([
                ("action", "add_reward_token"),
                ("owner", campaign_info.owner.as_ref()),
                ("reward_token_info", &denom),
                ("reward_token_amount", &amount.to_string()),
                (
                    "reward_per_second",
                    &campaign_info.reward_per_second.to_string(),
                ),
                ("start_time", &campaign_info.start_time.to_string()),
                ("end_time", &campaign_info.end_time.to_string()),
            ]))
        }
    }
}

pub fn execute_stake_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_ids: Vec<String>,
) -> Result<Response, ContractError> {
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    // check reach end date
    if campaign_info.start_time < env.block.time || campaign_info.end_time > env.block.time {
        return Err(ContractError::NotAvailableForStaking {});
    }

    let staker_info: StakerRewardAssetInfo =
        STAKERS_INFO.load(deps.storage, info.sender.clone())?;
    // if limit per staker > 0 then check amount nft staked
    if campaign_info.limit_per_staker > 0 {
        // the length of token_ids + length nft staked should be smaller than limit per staker
        if token_ids.len() + staker_info.nft_list.len() > campaign_info.limit_per_staker as usize {
            return Err(ContractError::TooManyTokenIds {});
        }
    }

    // TODO: check more condition of staking nft

    // prepare response
    let mut res = Response::new();

    // check the owner of token_ids, all token_ids should be owned by info.sender
    for token_id in token_ids.iter() {
        let query_owner_msg = Cw721QueryMsg::OwnerOf {
            token_id: token_id.clone(),
            include_expired: Some(false),
        };

        let owner_response: StdResult<cw721::OwnerOfResponse> =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: campaign_info.allowed_collection.to_string(),
                msg: to_binary(&query_owner_msg)?,
            }));
        match owner_response {
            Ok(owner) => {
                if owner.owner != info.sender {
                    return Err(ContractError::NotOwner {
                        token_id: token_id.to_string(),
                    });
                }
            }
            Err(_) => {
                return Err(ContractError::NotOwner {
                    token_id: token_id.to_string(),
                });
            }
        }

        // prepare message to transfer nft to contract
        let transfer_nft_msg: WasmMsg = WasmMsg::Execute {
            contract_addr: campaign_info.allowed_collection.to_string(),
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: env.contract.address.to_string(),
                token_id: token_id.clone(),
            })?,
            funds: vec![],
        };

        if STAKERS_INFO
            .may_load(deps.storage, info.sender.clone())?
            .is_none()
        {
            let mut nft_list: Vec<NftInfo> = vec![];
            nft_list.push(NftInfo {
                token_id: token_id.clone(),
                reward_time: env.block.time,
                stake_time: env.block.time,
            });
            let staked: StakerRewardAssetInfo = StakerRewardAssetInfo {
                nft_list,
                reward_debt: Uint128::zero(),
            };
            STAKERS_INFO.save(deps.storage, info.sender.clone(), &staked)?;
        } else {
            let mut staked: StakerRewardAssetInfo =
                STAKERS_INFO.load(deps.storage, info.sender.clone())?;
            staked.nft_list.push(NftInfo {
                token_id: token_id.clone(),
                reward_time: env.block.time,
                stake_time: env.block.time,
            });
            STAKERS_INFO.save(deps.storage, info.sender.clone(), &staked)?;
        }
        res = res.add_message(transfer_nft_msg);
    }

    // TODO: update campaign info if necessary

    Ok(res.add_attributes([
        ("action", "stake_nft"),
        ("owner", info.sender.as_ref()),
        (
            "allowed_collection",
            campaign_info.allowed_collection.as_ref(),
        ),
        ("token_ids", &token_ids.join(",")),
    ]))
}

pub fn execute_unstake_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_ids: Vec<String>,
) -> Result<Response, ContractError> {
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    // check reach end date
    // if campaign_info.start_time < env.block.time || campaign_info.end_time > env.block.time{
    //     return Err(ContractError::NotAvailableForStaking {});
    // }

    // if staker_info not found then err
    let staker_info = STAKERS_INFO.load(deps.storage, info.sender.clone())?;

    // if limit per staker > 0 then check amount nft staked
    if campaign_info.limit_per_staker > 0 {
        // the length of token_ids + length nft staked should be smaller than limit per staker
        if token_ids.len() > staker_info.nft_list.len() {
            return Err(ContractError::TooManyTokenIds {});
        }
    }

    // prepare response
    let mut res = Response::new();

    // check the owner of token_ids, all token_ids should be owned by the contract
    for token_id in token_ids.iter() {
        let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
        let staked: StakerRewardAssetInfo = STAKERS_INFO.load(deps.storage, info.sender.clone())?;
        let stake_info = staked
            .nft_list
            .iter()
            .find(|&x| x.token_id == token_id.clone());

        // check time unstake nft
        match stake_info {
            Some(stake_info) => {
                let time_unstake = env
                    .block
                    .time
                    .minus_seconds(campaign_info.lockup_term.to_string().parse().unwrap());
                if stake_info.stake_time > time_unstake {
                    return Err(ContractError::InvalidFunds {});
                }
            }
            None => return Err(ContractError::InvalidFunds {}),
        }

        let query_owner_msg = Cw721QueryMsg::OwnerOf {
            token_id: token_id.clone(),
            include_expired: Some(false),
        };

        let owner_response: StdResult<cw721::OwnerOfResponse> =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: campaign_info.allowed_collection.to_string(),
                msg: to_binary(&query_owner_msg)?,
            }));
        match owner_response {
            Ok(owner) => {
                if owner.owner != env.contract.address {
                    return Err(ContractError::NotOwner {
                        token_id: token_id.to_string(),
                    });
                }
            }
            Err(_) => {
                return Err(ContractError::NotOwner {
                    token_id: token_id.to_string(),
                });
            }
        }

        // prepare message to transfer nft back to the owner
        let transfer_nft_msg = WasmMsg::Execute {
            contract_addr: campaign_info.allowed_collection.to_string(),
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: info.sender.to_string(),
                token_id: token_id.clone(),
            })?,
            funds: vec![],
        };
        let mut staked_nft = STAKERS_INFO.load(deps.storage, info.sender.clone())?;
        staked_nft
            .nft_list
            .retain(|item| item.token_id != token_id.clone());
        STAKERS_INFO.save(deps.storage, info.sender.clone(), &staked_nft)?;

        res = res.add_message(transfer_nft_msg);
    }

    // TODO: update campaign info if necessary

    Ok(res.add_attributes([
        ("action", "unstake_nft"),
        ("owner", info.sender.as_ref()),
        (
            "allowed_collection",
            campaign_info.allowed_collection.as_ref(),
        ),
        ("token_ids", &token_ids.join(",")),
    ]))
}

pub fn execute_claim_reward(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn execute_update_campaign(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    campaign_name: String,
    campaign_image: String,
    campaign_description: String,
    start_time: Timestamp, // start time must be from T + 1
    end_time: Timestamp,   // max 3 years
    limit_per_staker: u64,
    reward_token_info: RewardTokenInfo, // reward token
    allowed_collection: String,         // staking collection nft
    lockup_term: Uint128,               // flexible, 15days, 30days, 60days
    reward_per_second: Uint128,
) -> Result<Response, ContractError> {

    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
    // permission check
    if info.sender != campaign_info.owner {
        return Err(ContractError::Unauthorized {});
    }

    // time check
    if campaign_info.start_time > env.block.time {
        return Err(ContractError::NotAvailableForUpdate {});
    }

    let campaign_info = CampaignInfo{
        owner:campaign_info.owner.clone(),
        campaign_name:campaign_name.clone(),
        campaign_image:campaign_image.clone(),
        campaign_description:campaign_description.clone(),
        start_time: start_time.clone(),
        end_time: end_time.clone(),
        total_reward:Uint128::zero(),
        limit_per_staker:limit_per_staker.clone(),
        reward_token_info: reward_token_info.clone(),
        allowed_collection: deps.api.addr_validate(&allowed_collection).unwrap(),
        lockup_term:lockup_term.clone(),
        reward_per_second: reward_per_second.clone(),
    };

    // store campaign info
    CAMPAIGN_INFO.save(deps.storage, &campaign_info)?;

    Ok(Response::new().add_attribute("action", "update_campaign"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Campaign {} => Ok(to_binary(&query_campaign_info(deps)?)?),
        QueryMsg::RewardInfo { owner } => Ok(to_binary(&query_stake_nft_info(deps, owner)?)?),
    }
}

fn query_campaign_info(deps: Deps) -> Result<CampaignInfo, ContractError> {
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
    Ok(campaign_info)
}

fn query_stake_nft_info(deps: Deps, owner: Addr) -> Result<StakerRewardAssetInfo, ContractError> {
    let staked: StakerRewardAssetInfo = STAKERS_INFO.load(deps.storage, owner)?;
    // let a = Uint128::new(100);
    Ok(staked)
}
