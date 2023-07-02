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
    CampaignInfo, NftInfo, RewardTokenInfo, StakerRewardAssetInfo, CAMPAIGN_INFO, STAKERS_INFO, NftStake, STAKERS, LockupTerm,
};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg};
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
        owner: deps.api.addr_validate(&msg.owner.clone()).unwrap(),
        campaign_name: msg.campaign_name.clone(),
        campaign_image: msg.campaign_image.clone(),
        campaign_description: msg.campaign_description.clone(),
        start_time: Timestamp::from_seconds(msg.start_time),
        end_time: Timestamp::from_seconds(msg.end_time),
        total_reward: Uint128::zero(),
        total_reward_claimed: Uint128::zero(),
        total_daily_reward: Uint128::zero(),
        limit_per_staker: msg.limit_per_staker.clone(),
        reward_token_info: msg.reward_token_info.clone(),
        allowed_collection: deps.api.addr_validate(&msg.allowed_collection).unwrap(),
        lockup_term: msg.lockup_term.clone(),
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
        ("owner", &msg.owner.to_string()),
        ("campaign_name", &msg.campaign_name),
        ("campaign_image", &msg.campaign_image),
        ("campaign_description", &msg.campaign_description),
        ("start_time", &msg.start_time.to_string()),
        ("end_time", &msg.end_time.to_string()),
        ("total_reward", &"0".to_string()),
        ("limit_per_staker", &msg.limit_per_staker.to_string()),
        ("reward_token_info", &reward_token_info_str),
        ("allowed_collection", &msg.allowed_collection),
        ("reward_per_second", &msg.reward_per_second.to_string()),
        // ("lockup_term", &format!("{}", &msg.lockup_term)),
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
        ExecuteMsg::StakeNfts { nfts } => execute_stake_nft(deps, env, info, nfts),
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
    let mut campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    // only owner can add reward token
    if campaign_info.owner.clone() != info.sender.clone() {
        return Err(ContractError::Unauthorized {});
    }

    if campaign_info.start_time.seconds() <= env.block.time.seconds()
    {
        return Err(ContractError::NotAvailableForAddReward {});
    }

    // TODO: check more condition of adding reward token

    // TODO: update campaign info if necessary

    // we need determine the reward token is native token or cw20 token
    match campaign_info.reward_token_info.clone() {
        RewardTokenInfo::Token { contract_addr } => {
            // check balance
            let query_balance_msg = Cw20QueryMsg::Balance {
                address: info.sender.clone().to_string(),
            };
            let balance_response: StdResult<cw20::BalanceResponse> =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: contract_addr.clone().to_string(),
                    msg: to_binary(&query_balance_msg)?,
                }));
            match balance_response {
                Ok(balance) => {
                    if balance.balance < amount {
                        return Err(ContractError::InsufficientBalance {});
                    }
                }
                Err(_) => {
                    return Err(ContractError::InsufficientBalance {});
                }
            }

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

            // update total_reward
            campaign_info.total_reward += amount;
            campaign_info.reward_per_second = campaign_info.total_reward / Uint128::new((campaign_info.end_time.seconds() - campaign_info.start_time.seconds()) as u128) ;
            CAMPAIGN_INFO.save(deps.storage, &campaign_info)?;

            Ok(Response::new()
                .add_message(transfer_reward)
                .add_attributes([
                    ("action", "add_reward_token"),
                    ("owner", campaign_info.owner.as_ref()),
                    ("reward_token_info", contract_addr.as_ref()),
                    ("reward_token_amount", &amount.to_string()),
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
    nfts: Vec<NftStake>,
) -> Result<Response, ContractError> {
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    // TODO: check time + lockup_term
    // if campaign_info.start_time.clone() > env.block.time.clone() || campaign_info.end_time.clone() < env.block.time.clone() {
    //     return Err(ContractError::NotAvailableForStaking {});
    // }

    let staker_info =
        STAKERS_INFO.may_load(deps.storage, info.sender.clone())?.unwrap_or(StakerRewardAssetInfo { nft_list: vec![], reward_debt: Uint128::zero(),reward_claimed:Uint128::zero() });

    // if limit per staker > 0 then check amount nft staked
    if campaign_info.limit_per_staker.clone() > 0 {
        // the length of token_ids + length nft staked should be smaller than limit per staker
        if nfts.len() + staker_info.nft_list.len() > campaign_info.limit_per_staker.clone() as usize {
            return Err(ContractError::TooManyTokenIds {});
        }
    }

    // prepare response
    let mut res = Response::new();

    // check the owner of token_ids, all token_ids should be owned by info.sender
    for nft in nfts.iter() {
        let query_owner_msg = Cw721QueryMsg::OwnerOf {
            token_id: nft.token_id.clone(),
            include_expired: Some(false),
        };

        let owner_response: StdResult<cw721::OwnerOfResponse> =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: campaign_info.allowed_collection.clone().to_string(),
                msg: to_binary(&query_owner_msg)?,
            }));
        match owner_response {
            Ok(owner) => {
                if owner.owner != info.sender {
                    return Err(ContractError::NotOwner {
                        token_id: nft.token_id.to_string(),
                    });
                }
            }
            Err(_) => {
                return Err(ContractError::NotOwner {
                    token_id: nft.token_id.to_string(),
                });
            }
        }

        // prepare message to transfer nft to contract
        let transfer_nft_msg: WasmMsg = WasmMsg::Execute {
            contract_addr: campaign_info.allowed_collection.to_string(),
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: env.contract.address.clone().to_string(),
                token_id: nft.token_id.clone(),
            })?,
            funds: vec![],
        };



        let stakers: Vec<Addr> = STAKERS.may_load(deps.storage)?.unwrap_or(vec![]);

        // update pending reward
        let res_nft_list = handle_nft_list(&deps, env.clone())?;
        for staker in stakers.clone() {
            let save_list:Vec<NftInfo> = res_nft_list.iter().filter(| nft | {nft.owner_nft == staker}).map(|nft|nft.clone()).collect();
            let total_pending_reward: Uint128 = res_nft_list.iter().fold(Uint128::zero(), |acc, nft| acc + nft.pending_reward);

            let mut load_staker = STAKERS_INFO.load(deps.storage, staker.clone())?;
            load_staker.nft_list = save_list;
            load_staker.reward_debt = total_pending_reward;
            STAKERS_INFO.save(deps.storage, staker, &load_staker)?;
        }

        let nft = NftInfo {
            owner_nft:info.sender.clone(),
            token_id: nft.token_id.clone(),
            pending_reward: Uint128::zero(),
            lockup_term:nft.lockup_term.clone(),
            start_time: env.block.time,
            end_time:env.block.time.plus_seconds(nft.lockup_term.value.clone()),
            is_end_stake:false,
            time_calc: env.block.time
        };

        // check staker is staked
        if staker_info.nft_list.len() == 0
        {
            let mut nft_list: Vec<NftInfo> = vec![];

            nft_list.push(nft);
            let staked: StakerRewardAssetInfo = StakerRewardAssetInfo {
                nft_list,
                reward_debt: Uint128::zero(),
                reward_claimed:Uint128::zero()
            };
            STAKERS_INFO.save(deps.storage, info.sender.clone(), &staked)?;

        } else {
            let mut staked: StakerRewardAssetInfo =
                STAKERS_INFO.load(deps.storage, info.sender.clone())?;
            staked.nft_list.push(nft);
            STAKERS_INFO.save(deps.storage, info.sender.clone(), &staked)?;
        }

        if !stakers.contains(&info.sender.clone()){
            let mut update_stakers = stakers.clone();
            update_stakers.push(info.sender.clone());
            STAKERS.save(deps.storage, &update_stakers)?;
        }
        res = res.add_message(transfer_nft_msg);
    }

    // TODO: update campaign info if necessary

    let mut token_ids = vec![];
    for nft in nfts.iter() {
        token_ids.push(nft.token_id.clone());
    }

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
    let stakers: Vec<Addr> = STAKERS.load(deps.storage)?;

    // check the owner of token_ids, all token_ids should be owned by the contract
    for token_id in token_ids.iter() {
        // TODO: update pending reward
        let res_nft_list = handle_nft_list(&deps, env.clone())?;
        for staker in stakers.clone() {
            let save_list:Vec<NftInfo> = res_nft_list.iter().filter(| nft | {nft.owner_nft == staker}).map(|nft|nft.clone()).collect();
            let total_pending_reward: Uint128 = res_nft_list.iter().fold(Uint128::zero(), |acc, nft| acc + nft.pending_reward);

            let mut load_staker = STAKERS_INFO.load(deps.storage, staker.clone())?;
            load_staker.nft_list = save_list;
            load_staker.reward_debt = total_pending_reward;

            STAKERS_INFO.save(deps.storage, staker, &load_staker)?;
        }

        let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
        let staked: StakerRewardAssetInfo = STAKERS_INFO.load(deps.storage, info.sender.clone())?;
        let stake_info = staked
            .nft_list
            .iter()
            .find(|&x| x.token_id == token_id.clone());

        // check time unstake nft
        // match stake_info {
        //     Some(stake_info) => {
        //         if stake_info.end_time.seconds() < env.block.time.seconds() {
        //             return Err(ContractError::NotAvailableForUnStake {});
        //         }
        //     }
        //     None => return Err(ContractError::NotAvailableForUnStake {}),
        // }

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
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // load campaign info
    let mut campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
    let staker = STAKERS_INFO.load(deps.storage, info.sender.clone())?;
    // we need determine the reward token is native token or cw20 token

    // TODO: update pending reward
    // let stakers: Vec<Addr> = STAKERS.load(deps.storage)?;
    // let res_nft_list = handle_nft_list(&deps, env.clone())?;
    // for staker in stakers.clone() {
    //     let save_list:Vec<NftInfo> = res_nft_list.iter().filter(| nft | {nft.owner_nft == staker}).map(|nft|nft.clone()).collect();
    //     let total_pending_reward: u64 = res_nft_list.iter().fold(0, |acc, nft| acc + nft.pending_reward);

    //     let mut load_staker = STAKERS_INFO.load(deps.storage, staker.clone())?;
    //     load_staker.nft_list = save_list;
    //     load_staker.reward_debt = total_pending_reward;

    //     STAKERS_INFO.save(deps.storage, staker, &load_staker)?;
    // }

    match campaign_info.reward_token_info.clone() {
        RewardTokenInfo::Token { contract_addr } => {
            // check balance
            let query_balance_msg = Cw20QueryMsg::Balance {
                address: env.contract.address.clone().to_string(),
            };
            let balance_response: StdResult<cw20::BalanceResponse> =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: contract_addr.clone().to_string(),
                    msg: to_binary(&query_balance_msg)?,
                }));
            match balance_response {
                Ok(balance) => {
                    if balance.balance < Uint128::new(1) {
                        return Err(ContractError::InsufficientBalance {});
                    }
                }
                Err(_) => {
                    return Err(ContractError::InsufficientBalance {});
                }
            }

            // execute cw20 transfer msg from info.sender to contract
            let transfer_reward: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract_addr.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: info.sender.to_string(),
                    amount:Uint128::new(10),
                    // amount:Uint128::new(staker.reward_debt.clone() as u128),
                })?,
                funds: vec![],
            });

            // update staker
            // let mut update_staker = staker.clone();
            // let mut new_nfts:Vec<NftInfo> = vec![];
            // for nft in update_staker.nft_list.clone() {
            //     let mut n = nft.clone();
            //     n.pending_reward = 0;
            //     new_nfts.push(n);
            // }
            // update_staker.reward_claimed = update_staker.reward_debt;
            // update_staker.reward_debt = 0;
            // update_staker.nft_list = new_nfts;
            // STAKERS_INFO.save(deps.storage, info.sender.clone(), &update_staker)?;

            // update total_reward
            campaign_info.total_reward -= Uint128::new(10);
            // campaign_info.total_reward -= Uint128::new(staker.reward_debt.clone() as u128);
            CAMPAIGN_INFO.save(deps.storage, &campaign_info)?;

            Ok(Response::new()
                .add_message(transfer_reward)
                .add_attributes([
                    ("action", "add_reward_token"),
                    ("owner", campaign_info.owner.as_ref()),
                    ("reward_token_info", contract_addr.as_ref()),
                    // ("reward_token_amount", &staker.reward_debt.to_string()),
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
            // if !has_coins(
            //     &info.funds,
            //     &Coin {
            //         denom: denom.clone(),
            //         amount,
            //     },
            // ) {
            //     return Err(ContractError::InvalidFunds {});
            // }

            Ok(Response::new().add_attributes([
                ("action", "add_reward_token"),
                ("owner", campaign_info.owner.as_ref()),
                ("reward_token_info", &denom),
                // ("reward_token_amount", &staker.reward_debt.to_string()),
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

pub fn execute_update_campaign(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    campaign_name: String,
    campaign_image: String,
    campaign_description: String,
    start_time: u64, // start time must be from T + 1
    end_time: u64,   // max 3 years
    limit_per_staker: u64,
    reward_token_info: RewardTokenInfo, // reward token
    allowed_collection: String,         // staking collection nft
    lockup_term: Vec<LockupTerm>,               // flexible, 15days, 30days, 60days
) -> Result<Response, ContractError> {
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
    // permission check
    if info.sender != campaign_info.owner {
        return Err(ContractError::Unauthorized {});
    }

    // time check
    // if campaign_info.start_time <= env.block.time {
    //     return Err(ContractError::NotAvailableForUpdate {});
    // }

    let campaign_info = CampaignInfo {
        owner: campaign_info.owner.clone(),
        campaign_name: campaign_name.clone(),
        campaign_image: campaign_image.clone(),
        campaign_description: campaign_description.clone(),
        start_time: Timestamp::from_seconds(start_time.clone()),
        end_time: Timestamp::from_seconds(end_time.clone()),
        total_reward: campaign_info.total_reward.clone(),
        total_reward_claimed: campaign_info.total_reward_claimed.clone(),
        total_daily_reward: Uint128::zero(),
        limit_per_staker: limit_per_staker.clone(),
        reward_token_info: reward_token_info.clone(),
        allowed_collection: deps.api.addr_validate(&allowed_collection).unwrap(),
        lockup_term: lockup_term.clone(),
        reward_per_second: campaign_info.reward_per_second.clone(),
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
    Ok(staked)
}

fn handle_nft_list(deps: &DepsMut, env:Env) -> StdResult<Vec<NftInfo>> {
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
    let stakers: Vec<Addr> = STAKERS.may_load(deps.storage)?.unwrap_or(vec![]);

    let nft_list: Vec<NftInfo>= stakers.clone().iter().map(|staker | STAKERS_INFO.load(deps.storage, staker.clone())).collect::<StdResult<Vec<_>>>()?.iter().map(|staker|staker.nft_list.clone()).flatten().collect::<Vec<_>>();

    let reward_per_second = campaign_info.reward_per_second.clone();

    let mut reward:Uint128 = Uint128::zero();
    let mut sort_nft_list = nft_list.clone();
    sort_nft_list.sort_by(|a,b|a.end_time.clone().cmp(&b.end_time.clone()));

    let mut new_list:Vec<NftInfo> = vec![];
    let mut time_calc: Timestamp = Timestamp::default();
    // let mut count = nft_list.clone().iter().filter(|nft| nft.is_end_stake == false).count() as u64;

    let mut count_staker = Uint128::new(1);
    for nft in sort_nft_list{
        let mut n = nft.clone();

        if nft.end_time.seconds() <= env.block.time.seconds(){
            // xử lý ông này đã hết thời hạn stake
            // cộng cái reward kia vào reward cộng dồn
            reward += (Uint128::new((nft.end_time.seconds() - nft.time_calc.seconds()) as u128) ) / count_staker * reward_per_second; // * lockup_term.percent
            // tính pending reward
            n.pending_reward += reward;
            // trừ đi 1 ông staker
            // count_staker -= 1;
            // gắn cái time vào
            time_calc = nft.end_time;
            // set is_end_stake
            n.is_end_stake = true;
        }
        else{
            // gắn time mới vào để tính toán (nếu có)
            if time_calc != n.time_calc {
                n.time_calc = time_calc;
            }
            // xử lý pending reward
            n.pending_reward += (Uint128::new((env.block.time.seconds() - nft.time_calc.seconds()) as u128) / count_staker * reward_per_second) + reward; // * lockup_term.percent
            // set thời gian mới nhất
            n.time_calc = env.block.time;
        }

        new_list.push(n);
    }

    Ok(new_list)
}
