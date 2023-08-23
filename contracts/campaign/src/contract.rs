#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    has_coins, to_binary, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Order,
    QuerierWrapper, QueryRequest, Response, StdError, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    AssetTokenInfo, CampaignInfo, CampaignInfoResult, LockupTerm, NftInfo, NftStake,
    StakedInfoResult, StakerRewardAssetInfo, TokenInfo, UnStakeNft, CAMPAIGN_INFO, NFTS,
    NUMBER_OF_NFTS, STAKERS_INFO,
};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg, TokenInfoResponse};
use cw721::{Cw721ExecuteMsg, Cw721QueryMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:campaign";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_TIME_VALID: u64 = 94608000; // 3 years
const MAX_LENGTH_NAME: usize = 100;
const MAX_LENGTH_DESCRIPTION: usize = 500;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // set version to contract
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // validate token contract address
    match msg.reward_token_info.info.clone() {
        TokenInfo::Token { contract_addr } => {
            let _ = deps.api.addr_validate(&contract_addr);
        }
        TokenInfo::NativeToken { denom: _ } => {}
    }

    // campaign during max 3 years
    if (msg.end_time - msg.start_time) > MAX_TIME_VALID {
        return Err(ContractError::LimitStartDate {});
    }

    // validate limit character campaign name & campaign description
    if msg.campaign_name.len() > MAX_LENGTH_NAME {
        return Err(ContractError::LimitCharacter {
            max: MAX_LENGTH_NAME.to_string(),
        });
    }
    if msg.campaign_description.len() > MAX_LENGTH_DESCRIPTION {
        return Err(ContractError::LimitCharacter {
            max: MAX_LENGTH_DESCRIPTION.to_string(),
        });
    }

    // // TODO: lockup_term must be 15days, 30days, 60days
    // let lockup_term = &msg.lockup_term;
    // for term in lockup_term {
    //     if Term::from_value(&term.value).is_none() {
    //         return Err(ContractError::InvalidLockupTerm {});
    //     }
    // }

    // campaign info
    let campaign = CampaignInfo {
        owner: deps.api.addr_validate(&msg.owner).unwrap(),
        campaign_name: msg.campaign_name.clone(),
        campaign_image: msg.campaign_image.clone(),
        campaign_description: msg.campaign_description.clone(),
        total_reward_claimed: Uint128::zero(),
        total_reward: Uint128::zero(),
        limit_per_staker: msg.limit_per_staker,
        reward_token_info: AssetTokenInfo {
            info: msg.reward_token_info.info.clone(),
            amount: Uint128::zero(),
        },
        allowed_collection: deps.api.addr_validate(&msg.allowed_collection).unwrap(),
        lockup_term: msg.lockup_term.clone(),
        reward_per_second: Uint128::zero(),
        end_calc_nft: vec![],
        time_calc_nft: 0,
        start_time: msg.start_time,
        end_time: msg.end_time,
    };

    // save campaign info
    CAMPAIGN_INFO.save(deps.storage, &campaign)?;

    // init NUMBER_OF_NFTS to 0
    NUMBER_OF_NFTS.save(deps.storage, &0u64)?;

    // we need emit the information of reward token to response
    let reward_token_info_str = match msg.reward_token_info.info {
        TokenInfo::Token { contract_addr } => contract_addr,
        TokenInfo::NativeToken { denom } => denom,
    };

    // emit the information of instantiated campaign
    Ok(Response::new().add_attributes([
        ("action", "instantiate"),
        ("owner", &msg.owner),
        ("campaign_name", &msg.campaign_name),
        ("campaign_image", &msg.campaign_image),
        ("campaign_description", &msg.campaign_description),
        ("limit_per_staker", &msg.limit_per_staker.to_string()),
        ("reward_token_info", &reward_token_info_str),
        ("allowed_collection", &msg.allowed_collection),
        ("lockup_term", &format!("{:?}", &msg.lockup_term)),
        ("start_time", &msg.start_time.to_string()),
        ("end_time", &msg.end_time.to_string()),
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
        ExecuteMsg::UnstakeNfts { nfts } => execute_unstake_nft(deps, env, info, nfts),
        ExecuteMsg::ClaimReward { amount } => execute_claim_reward(deps, env, info, amount),
        ExecuteMsg::WithdrawReward {} => execute_withdraw_reward(deps, env, info),
        ExecuteMsg::UpdateCampaign {
            campaign_name,
            campaign_image,
            campaign_description,
            start_time,
            end_time,
            limit_per_staker,
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

    let current_time = env.block.time.seconds();

    // only owner can add reward token to campaign
    if campaign_info.owner.clone() != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    // only reward_per_second == 0 || start_time > current_time can add reward
    if campaign_info.reward_per_second != Uint128::zero()
        && campaign_info.start_time <= current_time
    {
        return Err(ContractError::InvalidTimeToAddReward {});
    }

    // we need determine the reward token is native token or cw20 token
    match campaign_info.reward_token_info.info.clone() {
        TokenInfo::Token { contract_addr } => {
            // check balance
            let query_balance_msg = Cw20QueryMsg::Balance {
                address: info.sender.to_string(),
            };
            let balance_response: StdResult<cw20::BalanceResponse> =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: contract_addr.to_string(),
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

            // update amount, reward_per_second token in campaign
            campaign_info.reward_token_info.amount += amount;
            campaign_info.reward_per_second = campaign_info.reward_token_info.amount
                / Uint128::from((campaign_info.end_time - campaign_info.start_time) as u128);
            campaign_info.total_reward += amount;

            // save campaign
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
        TokenInfo::NativeToken { denom } => {
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
    // load campaign info
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    let current_time = env.block.time.seconds();

    // only start_time < current_time && current_time < end_time && amount != 0 can stake nft
    if campaign_info.start_time >= current_time
        || campaign_info.end_time <= current_time
        || campaign_info.reward_token_info.amount == Uint128::zero()
    {
        return Err(ContractError::InvalidTimeToStakeNft {});
    }

    // load staker_info or default if staker has not staked nft
    let mut staker_info = STAKERS_INFO
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or(StakerRewardAssetInfo {
            nft_keys: vec![],
            reward_debt: Uint128::zero(),
            reward_claimed: Uint128::zero(),
        });

    // if limit per staker > 0 then check amount nft staked
    // if limit_per_staker = 0, then no limit nft stake
    if campaign_info.limit_per_staker > 0 {
        // the length of token_ids + length nft staked should be smaller than limit per staker
        if nfts.len() + staker_info.nft_keys.len() > campaign_info.limit_per_staker as usize {
            return Err(ContractError::LimitPerStake {});
        }
    }

    // prepare response
    let mut res = Response::new();

    // check the owner of token_ids, all token_ids should be owned by info.sender
    for nft in &nfts {
        // check invalid lockup_term
        if !campaign_info
            .clone()
            .lockup_term
            .iter()
            .any(|t| t.value == nft.lockup_term)
        {
            return Err(ContractError::InvalidLockupTerm {});
        }

        // check owner of nft
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
            contract_addr: campaign_info.allowed_collection.clone().to_string(),
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: env.contract.address.clone().to_string(),
                token_id: nft.token_id.clone(),
            })?,
            funds: vec![],
        };

        // load lockup_term in campaign info
        let lockup_term = campaign_info
            .lockup_term
            .iter()
            .find(|&term| term.value == nft.lockup_term)
            .cloned()
            .unwrap();

        let nft_key = NUMBER_OF_NFTS.load(deps.storage)? + 1;
        let nft_info = NftInfo {
            key: nft_key,
            token_id: nft.token_id.clone(),
            owner_nft: info.sender.clone(),
            pending_reward: Uint128::zero(),
            lockup_term: lockup_term.clone(),
            is_end_reward: false,
            is_unstake: false,
            start_time: env.block.time.seconds(),
            end_time: (env.block.time.seconds() + lockup_term.value),
        };
        // save info nft
        NFTS.save(deps.storage, nft_key, &nft_info)?;

        // increase NUMBER_OF_NFTS
        NUMBER_OF_NFTS.save(deps.storage, &nft_key)?;

        // save staker_info
        staker_info.nft_keys.push(nft_key);
        STAKERS_INFO.save(deps.storage, info.sender.clone(), &staker_info)?;

        let mut update_campaign = campaign_info.clone();

        if campaign_info.time_calc_nft != 0 {
            // update pending reward for previous staking nft
            let terms = campaign_info.clone().lockup_term;
            let time_calc_nft = campaign_info.time_calc_nft;
            let end_calc_nft = &campaign_info.end_calc_nft;
            let reward_per_second = campaign_info.reward_per_second;

            let all_nft = (1..nft_key)
                .filter(|&key| !end_calc_nft.contains(&key))
                .filter_map(|key| NFTS.load(deps.storage, key).ok())
                .collect::<Vec<_>>();

            for term in terms {
                let mut nft_list = all_nft
                    .clone()
                    .into_iter()
                    .filter(|nft| nft.lockup_term.value == term.value)
                    .collect::<Vec<_>>();
                nft_list.sort_by(|a, b| a.end_time.cmp(&b.end_time));

                let mut time_calc: u64 = time_calc_nft;
                let mut nft_count = nft_list.len() as u128;
                let mut reward = Uint128::zero();
                for nft in nft_list.iter_mut() {
                    if nft.end_time <= current_time {
                        // reward in time_calc -> nft end_time
                        reward += Uint128::from((nft.end_time - time_calc) as u128)
                            * reward_per_second
                            * term.percent
                            / Uint128::new(100)
                            / Uint128::from(nft_count);
                        nft.pending_reward += reward;
                        nft_count -= 1;
                        time_calc = nft.end_time; // update time_calc
                        nft.is_end_reward = true; // nft stake timeout
                        update_campaign.end_calc_nft.push(nft.key);
                    } else {
                        let accumulate_reward = Uint128::from((current_time - time_calc) as u128)
                            * reward_per_second
                            * term.percent
                            / Uint128::new(100)
                            / Uint128::from(nft_count)
                            + reward;
                        nft.pending_reward += accumulate_reward;
                    }
                    // save nfts
                    NFTS.save(deps.storage, nft.key, nft)?;
                }
            }
        }

        // update time calc pending reward for nft
        update_campaign.time_calc_nft = current_time;
        CAMPAIGN_INFO.save(deps.storage, &update_campaign)?;
        res = res.add_message(transfer_nft_msg);
    }

    Ok(res.add_attributes([
        ("action", "stake_nft"),
        ("owner", info.sender.as_ref()),
        (
            "allowed_collection",
            campaign_info.allowed_collection.as_ref(),
        ),
        ("nfts", &format!("{:?}", &nfts)),
    ]))
}

pub fn execute_unstake_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    nfts: Vec<UnStakeNft>,
) -> Result<Response, ContractError> {
    // load campaign info
    let mut campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
    // prepare response
    let mut res = Response::new();

    // max time calc pending reward is campaign_info.end_time
    let mut current_time = env.block.time.seconds();
    if campaign_info.end_time < env.block.time.seconds() {
        current_time = campaign_info.end_time;
    }

    // update pending reward for previous staking nft
    let nft_key = NUMBER_OF_NFTS.load(deps.storage)? + 1;
    let terms = campaign_info.clone().lockup_term;
    let time_calc_nft = campaign_info.time_calc_nft;
    let end_calc_nft = &campaign_info.end_calc_nft;
    let reward_per_second = campaign_info.reward_per_second;

    let all_nft = (1..nft_key)
        .filter(|&key| !end_calc_nft.contains(&key))
        .filter_map(|key| NFTS.load(deps.storage, key).ok())
        .collect::<Vec<_>>();

    for term in terms {
        let mut nft_list = all_nft
            .clone()
            .into_iter()
            .filter(|nft| nft.lockup_term.value == term.value)
            .collect::<Vec<_>>();
        nft_list.sort_by(|a, b| a.end_time.cmp(&b.end_time));

        let mut time_calc: u64 = time_calc_nft;
        let mut staker_count = nft_list.len() as u128;
        let mut reward = Uint128::zero();
        for nft in nft_list.iter_mut() {
            if nft.end_time <= current_time {
                // reward in time_calc -> nft end_time
                reward += Uint128::from((nft.end_time - time_calc) as u128)
                    * reward_per_second
                    * term.percent
                    / Uint128::new(100)
                    / Uint128::from(staker_count);
                nft.pending_reward += reward;
                staker_count -= 1;
                time_calc = nft.end_time; // update time_calc
                nft.is_end_reward = true; // nft stake timeout
                campaign_info.end_calc_nft.push(nft.key);
            } else {
                let accumulate_reward = Uint128::from((current_time - time_calc) as u128)
                    * reward_per_second
                    * term.percent
                    / Uint128::new(100)
                    / Uint128::from(staker_count)
                    + reward;
                nft.pending_reward += accumulate_reward;
            }
            if env.block.time.seconds() >= campaign_info.end_time {
                nft.is_end_reward = true;
            }
            // save nfts
            NFTS.save(deps.storage, nft.key, nft)?;
        }
    }

    // update time calc pending reward for nft
    campaign_info.time_calc_nft = current_time;
    CAMPAIGN_INFO.save(deps.storage, &campaign_info)?;

    // check the owner of token_ids, all token_ids should be owned by the contract
    for nft in nfts.iter() {
        let nft_info = NFTS.load(deps.storage, nft.nft_key)?;

        // check time unstake and owner nft
        if !nft_info.is_end_reward {
            return Err(ContractError::InvalidTimeToUnStake {});
        }

        let query_owner_msg = Cw721QueryMsg::OwnerOf {
            token_id: nft.token_id.clone(),
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

        // prepare message to transfer nft back to the owner
        let transfer_nft_msg = WasmMsg::Execute {
            contract_addr: campaign_info.allowed_collection.to_string(),
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: info.sender.to_string(),
                token_id: nft.token_id.clone(),
            })?,
            funds: vec![],
        };

        // update reward for staker
        let mut staker = STAKERS_INFO.load(deps.storage, info.sender.clone())?;
        staker.reward_debt += nft_info.pending_reward;
        staker.nft_keys.retain(|&key| key != nft.nft_key); // remove nft for staker
        STAKERS_INFO.save(deps.storage, info.sender.clone(), &staker)?;

        // update nft
        let mut update_nft = nft_info.clone();
        update_nft.is_unstake = true;
        update_nft.pending_reward = Uint128::zero();
        NFTS.save(deps.storage, nft.nft_key, &update_nft)?;

        res = res.add_message(transfer_nft_msg);
    }

    Ok(res.add_attributes([
        ("action", "unstake_nft"),
        ("owner", info.sender.as_ref()),
        (
            "allowed_collection",
            campaign_info.allowed_collection.as_ref(),
        ),
        ("nfts", &format!("{:?}", &nfts)),
    ]))
}

pub fn execute_claim_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // load campaign info
    let mut campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    // Only stakers could claim rewards in this campaign
    if STAKERS_INFO
        .may_load(deps.storage, info.sender.clone())?
        .is_none()
    {
        return Err(ContractError::InvalidClaim {});
    }

    // load staker_info
    let mut staker_info = STAKERS_INFO.load(deps.storage, info.sender.clone())?;

    // max time calc pending reward is campaign_info.end_time
    let mut current_time = env.block.time.seconds();
    if campaign_info.end_time < env.block.time.seconds() {
        current_time = campaign_info.end_time;
    }

    // update pending reward for previous staking nft
    let nft_key = NUMBER_OF_NFTS.load(deps.storage)? + 1;
    let terms = campaign_info.clone().lockup_term;
    let time_calc_nft = campaign_info.time_calc_nft;
    let end_calc_nft = &campaign_info.end_calc_nft;
    let reward_per_second = campaign_info.reward_per_second;

    let all_nft = (1..nft_key)
        .filter(|&key| !end_calc_nft.contains(&key))
        .filter_map(|key| NFTS.load(deps.storage, key).ok())
        .collect::<Vec<_>>();

    for term in terms {
        let mut nft_list = all_nft
            .clone()
            .into_iter()
            .filter(|nft| nft.lockup_term.value == term.value)
            .collect::<Vec<_>>();
        nft_list.sort_by(|a, b| a.end_time.cmp(&b.end_time));

        let mut time_calc: u64 = time_calc_nft;
        let mut staker_count = nft_list.len() as u128;
        let mut reward = Uint128::zero();
        for nft in nft_list.iter_mut() {
            if nft.end_time <= current_time {
                // reward in time_calc -> nft end_time
                reward += Uint128::from((nft.end_time - time_calc) as u128)
                    * reward_per_second
                    * term.percent
                    / Uint128::new(100)
                    / Uint128::from(staker_count);
                nft.pending_reward += reward;
                staker_count -= 1;
                time_calc = nft.end_time; // update time_calc
                nft.is_end_reward = true; // nft stake timeout
                campaign_info.end_calc_nft.push(nft.key);
            } else {
                let accumulate_reward = Uint128::from((current_time - time_calc) as u128)
                    * reward_per_second
                    * term.percent
                    / Uint128::new(100)
                    / Uint128::from(staker_count)
                    + reward;
                nft.pending_reward += accumulate_reward;
            }
            if env.block.time.seconds() >= campaign_info.end_time {
                nft.is_end_reward = true;
            }
            // save nfts
            NFTS.save(deps.storage, nft.key, nft)?;
        }
    }

    // update time calc pending reward for nft
    campaign_info.time_calc_nft = current_time;

    // calc pending reward for staker
    for key in staker_info.nft_keys.iter() {
        let mut nft = NFTS.load(deps.storage, *key)?;
        if !nft.clone().is_unstake {
            staker_info.reward_debt += nft.pending_reward;

            //update pending reward for nft = 0 because pending reward in nft are transferred to staker
            nft.pending_reward = Uint128::zero();
        }
        NFTS.save(deps.storage, *key, &nft)?;
    }

    // amount reward claim must be less than or equal reward in staker
    if amount > staker_info.reward_debt {
        return Err(ContractError::InsufficientBalance {});
    }

    match campaign_info.reward_token_info.info.clone() {
        TokenInfo::Token { contract_addr } => {
            // check balance
            let query_balance_msg = Cw20QueryMsg::Balance {
                address: env.contract.address.to_string(),
            };
            let balance_response: StdResult<cw20::BalanceResponse> =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: contract_addr.to_string(),
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
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: info.sender.to_string(),
                    amount,
                })?,
                funds: vec![],
            });

            // update staker info
            staker_info.reward_claimed += amount;
            staker_info.reward_debt -= amount;
            STAKERS_INFO.save(deps.storage, info.sender, &staker_info)?;

            // update reward total and reward claimed for campaign
            campaign_info.reward_token_info.amount -= amount;
            campaign_info.total_reward_claimed += amount;

            // save campaign info
            CAMPAIGN_INFO.save(deps.storage, &campaign_info)?;

            Ok(Response::new()
                .add_message(transfer_reward)
                .add_attributes([
                    ("action", "claim_reward"),
                    ("owner", campaign_info.owner.as_ref()),
                    ("reward_token_info", contract_addr.as_ref()),
                    ("reward_claim_amount", &amount.to_string()),
                ]))
        }
        TokenInfo::NativeToken { denom } => {
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
                ("action", "claim_reward"),
                ("owner", campaign_info.owner.as_ref()),
                ("denom", &denom),
                ("reward_claim_amount", &amount.to_string()),
            ]))
        }
    }
}

pub fn execute_withdraw_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // load campaign info
    let mut campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    // permission check
    if info.sender != campaign_info.owner {
        return Err(ContractError::Unauthorized {});
    }

    // campaing must be ended then withdraw remaining reward
    if campaign_info.end_time > env.block.time.seconds() {
        return Err(ContractError::InvalidTimeToWithdrawReward {});
    }

    // calc total pending reward that user owns
    let mut total_pending_reward = Uint128::zero();

    let current_time = campaign_info.end_time;

    // update pending reward for all nft
    let nft_key = NUMBER_OF_NFTS.load(deps.storage)? + 1;
    let terms = campaign_info.clone().lockup_term;
    let time_calc_nft = campaign_info.time_calc_nft;
    // let end_calc_nft = &campaign_info.end_calc_nft;
    let reward_per_second = campaign_info.reward_per_second;

    let all_nft = (1..nft_key)
        .filter_map(|key| NFTS.load(deps.storage, key).ok())
        .collect::<Vec<_>>();

    for term in terms {
        let mut nft_list = all_nft
            .clone()
            .into_iter()
            .filter(|nft| nft.lockup_term.value == term.value)
            .collect::<Vec<_>>();
        nft_list.sort_by(|a, b| a.end_time.cmp(&b.end_time));

        let mut time_calc: u64 = time_calc_nft;
        let mut nft_count = nft_list
            .clone()
            .into_iter()
            .filter(|nft| !nft.is_end_reward)
            .collect::<Vec<_>>()
            .len() as u128;
        let mut reward = Uint128::zero();
        for nft in nft_list.iter_mut() {
            if !nft.is_end_reward {
                if nft.end_time <= current_time {
                    // reward in time_calc -> nft end_time
                    reward += Uint128::from((nft.end_time - time_calc) as u128)
                        * reward_per_second
                        * term.percent
                        / Uint128::new(100)
                        / Uint128::from(nft_count);
                    nft.pending_reward += reward;
                    nft_count -= 1;
                    time_calc = nft.end_time; // update time_calc
                    nft.is_end_reward = true; // nft stake timeout
                    campaign_info.end_calc_nft.push(nft.key);
                } else {
                    let accumulate_reward = Uint128::from((current_time - time_calc) as u128)
                        * reward_per_second
                        * term.percent
                        / Uint128::new(100)
                        / Uint128::from(nft_count)
                        + reward;
                    nft.pending_reward += accumulate_reward;
                }
                nft.is_end_reward = true;
            }

            // pending reward in nft
            total_pending_reward += nft.pending_reward;

            // save nfts
            NFTS.save(deps.storage, nft.key, nft)?;
        }
    }

    // pending reward in staker
    let stakers_info = STAKERS_INFO.range(deps.storage, None, None, Order::Ascending);
    for item in stakers_info {
        let (_, value) = item?;
        total_pending_reward += value.reward_debt;
    }

    // update time calc pending reward for nft
    campaign_info.time_calc_nft = current_time;
    CAMPAIGN_INFO.save(deps.storage, &campaign_info)?;

    // reward remaining = reward in campaign - total pending reward
    let withdraw_reward = campaign_info.reward_token_info.amount - total_pending_reward;

    match campaign_info.reward_token_info.info.clone() {
        TokenInfo::Token { contract_addr } => {
            // check balance
            let query_balance_msg = Cw20QueryMsg::Balance {
                address: env.contract.address.to_string(),
            };
            let balance_response: StdResult<cw20::BalanceResponse> =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: contract_addr.to_string(),
                    msg: to_binary(&query_balance_msg)?,
                }));
            match balance_response {
                Ok(balance) => {
                    if balance.balance < withdraw_reward {
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
                    amount: withdraw_reward,
                })?,
                funds: vec![],
            });

            // update reward total and reward claimed for campaign
            campaign_info.reward_token_info.amount -= withdraw_reward;
            CAMPAIGN_INFO.save(deps.storage, &campaign_info)?;

            Ok(Response::new()
                .add_message(transfer_reward)
                .add_attributes([
                    ("action", "withdraw_reward"),
                    ("owner", campaign_info.owner.as_ref()),
                    ("reward_token_info", contract_addr.as_ref()),
                    ("withdraw_reward_amount", &withdraw_reward.to_string()),
                ]))
        }
        TokenInfo::NativeToken { denom } => {
            // check the amount of native token in funds
            if !has_coins(
                &info.funds,
                &Coin {
                    denom: denom.clone(),
                    amount: withdraw_reward,
                },
            ) {
                return Err(ContractError::InvalidFunds {});
            }

            Ok(Response::new().add_attributes([
                ("action", "claim_reward"),
                ("owner", campaign_info.owner.as_ref()),
                ("denom", &denom),
                ("reward_claim_amount", &withdraw_reward.to_string()),
            ]))
        }
    }
}

pub fn execute_update_campaign(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    campaign_name: Option<String>,
    campaign_image: Option<String>,
    campaign_description: Option<String>,
    start_time: Option<u64>, // start time must be from T + 1
    end_time: Option<u64>,   // max 3 years
    limit_per_staker: Option<u64>,
    lockup_term: Option<Vec<LockupTerm>>, // flexible, 15days, 30days, 60days
) -> Result<Response, ContractError> {
    // load campaign info
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    let current_time = env.block.time.seconds();

    // permission check
    if info.sender != campaign_info.owner {
        return Err(ContractError::Unauthorized {});
    }

    // only campaign not yet add reward can update,
    if campaign_info.total_reward != Uint128::zero() {
        return Err(ContractError::InvalidTimeToUpdate {});
    }

    let update_start_time = if let Some(st) = start_time {
        st
    } else {
        campaign_info.start_time
    };
    let update_end_time = if let Some(et) = end_time {
        et
    } else {
        campaign_info.end_time
    };

    let update_name = if let Some(name) = campaign_name {
        name
    } else {
        campaign_info.campaign_name
    };
    let update_image = if let Some(image) = campaign_image {
        image
    } else {
        campaign_info.campaign_image
    };
    let update_description = if let Some(description) = campaign_description {
        description
    } else {
        campaign_info.campaign_description
    };

    let update_limit_per_staker = if let Some(limit_nft) = limit_per_staker {
        limit_nft
    } else {
        campaign_info.limit_per_staker
    };

    let update_lockup_term = if let Some(lockup_term) = lockup_term {
        lockup_term
    } else {
        campaign_info.lockup_term
    };

    // campaign during max 3 years
    if (update_end_time - update_start_time) > MAX_TIME_VALID {
        return Err(ContractError::LimitStartDate {});
    }

    // validate character limit campaign name & campaign description
    if update_name.len() > MAX_LENGTH_NAME {
        return Err(ContractError::LimitCharacter {
            max: MAX_LENGTH_NAME.to_string(),
        });
    }
    if update_description.len() > MAX_LENGTH_DESCRIPTION {
        return Err(ContractError::LimitCharacter {
            max: MAX_LENGTH_DESCRIPTION.to_string(),
        });
    }

    // Not allow start time is greater than end time
    if update_start_time >= update_end_time {
        return Err(ContractError::Std(StdError::generic_err(
            "## Start time is greater than end time ##",
        )));
    }

    // Not allow to create a campaign when current time is greater than start time
    if current_time > update_start_time {
        return Err(ContractError::Std(StdError::generic_err(
            "## Current time is greater than start time ##",
        )));
    }

    let campaign_info = CampaignInfo {
        owner: campaign_info.owner.clone(),
        campaign_name: update_name,
        campaign_image: update_image,
        campaign_description: update_description,
        time_calc_nft: campaign_info.time_calc_nft,
        end_calc_nft: campaign_info.end_calc_nft,
        start_time: update_start_time,
        end_time: update_end_time,
        total_reward_claimed: campaign_info.total_reward_claimed,
        total_reward: campaign_info.total_reward,
        limit_per_staker: update_limit_per_staker,
        reward_token_info: campaign_info.reward_token_info,
        allowed_collection: campaign_info.allowed_collection,
        lockup_term: update_lockup_term,
        reward_per_second: campaign_info.reward_per_second,
    };

    // save update campaign info
    CAMPAIGN_INFO.save(deps.storage, &campaign_info)?;

    Ok(Response::new().add_attributes([
        ("action", "update_campaign"),
        ("owner", campaign_info.owner.as_ref()),
        ("campaign_name", &campaign_info.campaign_name),
        ("campaign_image", &campaign_info.campaign_image),
        ("campaign_description", &campaign_info.campaign_description),
        (
            "limit_per_staker",
            &campaign_info.limit_per_staker.to_string(),
        ),
        (
            "reward_token_info",
            &format!("{:?}", &campaign_info.reward_token_info),
        ),
        (
            "allowed_collection",
            campaign_info.allowed_collection.as_ref(),
        ),
        ("lockup_term", &format!("{:?}", &campaign_info.lockup_term)),
        ("start_time", &update_start_time.to_string()),
        ("end_time", &update_end_time.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::CampaignInfo {} => Ok(to_binary(&query_campaign_info(deps)?)?),
        QueryMsg::NftInfo { nft_id } => Ok(to_binary(&query_nft_info(deps, nft_id)?)?),
        QueryMsg::NftStaked { owner } => Ok(to_binary(&query_staker_info(deps, env, owner)?)?),
        QueryMsg::Nfts {
            start_after,
            limit,
            owner,
        } => Ok(to_binary(&query_staker_nfts(
            deps,
            start_after,
            limit,
            owner,
        )?)?),
        QueryMsg::TotalPendingReward {} => Ok(to_binary(&query_total_pending_reward(deps, env)?)?),
    }
}

fn query_campaign_info(deps: Deps) -> Result<CampaignInfoResult, ContractError> {
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    let nft_key = NUMBER_OF_NFTS.load(deps.storage)?;

    let nft_list = (0..nft_key)
        .map(|key| NFTS.load(deps.storage, key + 1))
        .collect::<StdResult<Vec<_>>>()?
        .into_iter()
        .filter(|nft| !nft.is_end_reward)
        .collect::<Vec<_>>();
    let nft_staked: u64 = nft_list.len() as u64;

    let campaign_result = CampaignInfoResult {
        owner: campaign_info.owner,
        campaign_name: campaign_info.campaign_name,
        campaign_image: campaign_info.campaign_image,
        campaign_description: campaign_info.campaign_description,
        start_time: campaign_info.start_time,
        end_time: campaign_info.end_time,
        total_reward_claimed: campaign_info.total_reward_claimed,
        total_reward: campaign_info.total_reward,
        limit_per_staker: campaign_info.limit_per_staker,
        reward_token_info: campaign_info.reward_token_info,
        allowed_collection: campaign_info.allowed_collection,
        lockup_term: campaign_info.lockup_term,
        reward_per_second: campaign_info.reward_per_second,
        total_nft_staked: nft_staked,
    };
    Ok(campaign_result)
}

fn query_nft_info(deps: Deps, nft_id: u64) -> Result<NftInfo, ContractError> {
    let nft_info: NftInfo = NFTS.load(deps.storage, nft_id)?;
    Ok(nft_info)
}

fn query_staker_info(deps: Deps, env: Env, owner: Addr) -> Result<StakedInfoResult, ContractError> {
    let staker_asset: StakerRewardAssetInfo = STAKERS_INFO
        .may_load(deps.storage, owner)?
        .unwrap_or(StakerRewardAssetInfo {
            nft_keys: vec![],
            reward_debt: Uint128::zero(),
            reward_claimed: Uint128::zero(),
        });

    let mut staked_info = StakedInfoResult {
        nfts: vec![],
        reward_debt: staker_asset.reward_debt,
        reward_claimed: staker_asset.reward_claimed,
    };

    // update all pending reward
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
    let mut current_time = env.block.time.seconds();
    if campaign_info.end_time < env.block.time.seconds() {
        current_time = campaign_info.end_time;
    }

    // update pending reward for all nft
    let nft_key = NUMBER_OF_NFTS.load(deps.storage)? + 1;
    let terms = campaign_info.clone().lockup_term;
    let time_calc_nft = campaign_info.time_calc_nft;
    let reward_per_second = campaign_info.reward_per_second;

    let all_nft = (1..nft_key)
        .filter_map(|key| NFTS.load(deps.storage, key).ok())
        .collect::<Vec<_>>();

    for term in terms {
        let mut nft_list = all_nft
            .clone()
            .into_iter()
            .filter(|nft| nft.lockup_term.value == term.value)
            .collect::<Vec<_>>();
        nft_list.sort_by(|a, b| a.end_time.cmp(&b.end_time));

        let mut time_calc: u64 = time_calc_nft;
        let mut nft_count = nft_list
            .clone()
            .into_iter()
            .filter(|nft| !nft.is_end_reward)
            .collect::<Vec<_>>()
            .len() as u128;
        let mut reward = Uint128::zero();
        for nft in nft_list.iter_mut() {
            if !nft.is_end_reward {
                if nft.end_time <= current_time {
                    // reward in time_calc -> nft end_time
                    reward += Uint128::from((nft.end_time - time_calc) as u128)
                        * reward_per_second
                        * term.percent
                        / Uint128::new(100)
                        / Uint128::from(nft_count);
                    nft.pending_reward += reward;
                    nft_count -= 1;
                    time_calc = nft.end_time; // update time_calc
                    nft.is_end_reward = true; // nft stake timeout
                } else {
                    let accumulate_reward = Uint128::from((current_time - time_calc) as u128)
                        * reward_per_second
                        * term.percent
                        / Uint128::new(100)
                        / Uint128::from(nft_count)
                        + reward;
                    nft.pending_reward += accumulate_reward;
                }
                if env.block.time.seconds() >= campaign_info.end_time {
                    nft.is_end_reward = true;
                }
            }
            if staker_asset.nft_keys.contains(&nft.key) {
                staked_info.nfts.push(nft.clone());
                staked_info.reward_debt += nft.pending_reward;
            }
        }
    }

    Ok(staked_info)
}

fn query_staker_nfts(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
    owner: Option<Addr>,
) -> Result<Vec<NftInfo>, ContractError> {
    let start_after = start_after.unwrap_or(0);
    let nft_count = NUMBER_OF_NFTS.load(deps.storage)?;
    let limit = limit.unwrap_or(nft_count as u32) as usize;

    let mut nfts = (start_after..nft_count)
        .map(|nft_key| NFTS.load(deps.storage, nft_key + 1))
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;
    if let Some(owner) = owner {
        nfts.retain(|x| x.owner_nft == owner);
    }
    Ok(nfts)
}

fn query_total_pending_reward(deps: Deps, env: Env) -> Result<Uint128, ContractError> {
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
    let mut total_pending_reward: Uint128 = Uint128::zero(); // total = pending in nft + pending in staker

    let mut current_time = env.block.time.seconds();
    if campaign_info.end_time < env.block.time.seconds() {
        current_time = campaign_info.end_time;
    }
    // update pending reward for all nft
    let nft_key = NUMBER_OF_NFTS.load(deps.storage)? + 1;
    let terms = campaign_info.clone().lockup_term;
    let time_calc_nft = campaign_info.time_calc_nft;
    let reward_per_second = campaign_info.reward_per_second;

    let all_nft = (1..nft_key)
        .filter_map(|key| NFTS.load(deps.storage, key).ok())
        .collect::<Vec<_>>();

    for term in terms {
        let mut nft_list = all_nft
            .clone()
            .into_iter()
            .filter(|nft| nft.lockup_term.value == term.value)
            .collect::<Vec<_>>();
        nft_list.sort_by(|a, b| a.end_time.cmp(&b.end_time));

        let mut time_calc: u64 = time_calc_nft;
        let mut nft_count = nft_list
            .clone()
            .into_iter()
            .filter(|nft| !nft.is_end_reward)
            .collect::<Vec<_>>()
            .len() as u128;
        let mut reward = Uint128::zero();
        for nft in nft_list.iter_mut() {
            if !nft.is_end_reward {
                if nft.end_time <= current_time {
                    // reward in time_calc -> nft end_time
                    reward += Uint128::from((nft.end_time - time_calc) as u128)
                        * reward_per_second
                        * term.percent
                        / Uint128::new(100)
                        / Uint128::from(nft_count);
                    nft.pending_reward += reward;
                    nft_count -= 1;
                    time_calc = nft.end_time; // update time_calc
                    nft.is_end_reward = true; // nft stake timeout
                } else {
                    let accumulate_reward = Uint128::from((current_time - time_calc) as u128)
                        * reward_per_second
                        * term.percent
                        / Uint128::new(100)
                        / Uint128::from(nft_count)
                        + reward;
                    nft.pending_reward += accumulate_reward;
                }
            }
            // pending reward in nft
            total_pending_reward += nft.pending_reward;
        }
    }
    // get pending reward in staker
    let stakers_info = STAKERS_INFO.range(deps.storage, None, None, Order::Ascending);
    for item in stakers_info {
        let (_, value) = item?;
        total_pending_reward += value.reward_debt;
    }

    Ok(total_pending_reward)
}

pub fn query_token_info(
    querier: &QuerierWrapper,
    contract_addr: Addr,
) -> StdResult<TokenInfoResponse> {
    let info = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: contract_addr.to_string(),
        msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
    }))?;

    Ok(info)
}
