use std::collections::HashMap;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    has_coins, to_binary, Addr, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    QuerierWrapper, QueryRequest, Response, StdResult, Uint128, WasmMsg, WasmQuery,
};
use cw2::set_contract_version;
use schemars::_serde_json::value;

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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // set version to contract
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let mut symbol_token = "AURA".to_string();
    match msg.reward_token_info.info.clone() {
        TokenInfo::Token { contract_addr } => {
            let token_info =
                query_token_info(&deps.querier, deps.api.addr_validate(&contract_addr)?)?;
            symbol_token = token_info.symbol;
        }
        TokenInfo::NativeToken { denom } => {}
    }

    // get total nfts in collection
    let num_tokens_response: StdResult<cw721::NumTokensResponse> =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: msg.allowed_collection.clone(),
            msg: to_binary(&Cw721QueryMsg::NumTokens {})?,
        }));

    if (msg.end_time.clone() - msg.start_time.clone()) > 94608000
    // max 3 years
    {
        return Err(ContractError::LimitStartDate {});
    }

    if msg.campaign_name.len() > 100 || msg.campaign_description.len() > 500 {
        return Err(ContractError::LimitCharacter {});
    }

    // collect campaign info
    let campaign = CampaignInfo {
        owner: deps.api.addr_validate(&msg.owner.clone()).unwrap(),
        campaign_name: msg.campaign_name.clone(),
        campaign_image: msg.campaign_image.clone(),
        campaign_description: msg.campaign_description.clone(),
        total_reward_claimed: Uint128::zero(),
        total_reward: Uint128::zero(),
        limit_per_staker: msg.limit_per_staker.clone(),
        reward_token_info: AssetTokenInfo {
            info: msg.reward_token_info.info.clone(),
            amount: Uint128::zero(),
            symbol: symbol_token,
        },
        total_nft_collection: num_tokens_response?.count,
        allowed_collection: deps.api.addr_validate(&msg.allowed_collection).unwrap(),
        lockup_term: msg.lockup_term.clone(),
        reward_per_second: Uint128::zero(),
        start_time: msg.start_time.clone(),
        end_time: msg.end_time.clone(),
    };

    // store campaign info
    CAMPAIGN_INFO.save(deps.storage, &campaign)?;

    // init NUMBER_OF_NFTS to 0
    NUMBER_OF_NFTS.save(deps.storage, &0u64)?;

    // we need emit the information of reward token to response
    let reward_token_info_str: String;

    match msg.reward_token_info.info {
        TokenInfo::Token { contract_addr } => {
            reward_token_info_str = contract_addr.to_string();
        }
        TokenInfo::NativeToken { denom } => {
            reward_token_info_str = denom;
        }
    }

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
        ExecuteMsg::ClaimReward {} => execute_claim_reward(deps, env, info),
        ExecuteMsg::UpdateCampaign {
            campaign_name,
            campaign_image,
            campaign_description,
            start_time,
            end_time,
            limit_per_staker,
            allowed_collection,
            reward_token_info,
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
            allowed_collection,
            reward_token_info,
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

    if campaign_info.start_time <= env.block.time.seconds()
    {
        return Err(ContractError::NotAvailableForAddReward {});
    }

    // TODO: update campaign info if necessary

    // we need determine the reward token is native token or cw20 token
    match campaign_info.reward_token_info.info.clone() {
        TokenInfo::Token { contract_addr } => {
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

            // update amount token
            campaign_info.reward_token_info.amount += amount;
            campaign_info.reward_per_second = campaign_info.reward_token_info.amount.clone()
                / Uint128::from((campaign_info.end_time - campaign_info.start_time) as u128);
            campaign_info.total_reward += amount;

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
    if campaign_info.start_time.clone() > env.block.time.seconds()
        || campaign_info.end_time.clone() < env.block.time.seconds()
    {
        return Err(ContractError::NotAvailableForStaking {});
    }

    // load staker_info or default
    let mut staker_info = STAKERS_INFO
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or(StakerRewardAssetInfo {
            nft_keys: vec![],
            reward_debt: Uint128::zero(),
            reward_claimed: Uint128::zero(),
        });

    // if limit per staker > 0 then check amount nft staked
    if campaign_info.limit_per_staker.clone() > 0 {
        // the length of token_ids + length nft staked should be smaller than limit per staker
        if nfts.len() + staker_info.nft_keys.len() > campaign_info.limit_per_staker.clone() as usize
        {
            return Err(ContractError::TooManyTokenIds {});
        }
    }

    // prepare response
    let mut res = Response::new();

    // check the owner of token_ids, all token_ids should be owned by info.sender
    for nft in &nfts {
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

        let nft_key = NUMBER_OF_NFTS.load(deps.storage)? + 1;

        // get lockup term
        let find_term = campaign_info
            .lockup_term
            .iter()
            .find(|term| term.value == nft.lockup_term.clone());

        let lockup_term = match find_term {
            Some(find_term) => find_term.clone(),
            None => return Err(ContractError::InvalidLockupTerm {}),
        };

        let nft_info = NftInfo {
            key: nft_key,
            token_id: nft.token_id.clone(),
            owner_nft: info.sender.clone(),
            pending_reward: Uint128::zero(),
            lockup_term: lockup_term.clone(),
            time_calc: env.block.time.seconds(),
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
        staker_info.nft_keys.push(nft_key.clone());
        STAKERS_INFO.save(deps.storage, info.sender.clone(), &staker_info)?;

        // update pending reward for all nft

//         let lockup_term = campaign_info.lockup_term.clone();

//         for term in lockup_term.iter() {
//             // danh sach nft theo term chua end reward
//             let mut nft_list = (0..nft_key.clone())
//             .map(|key| NFTS.load(deps.storage, key + 1))
//             .collect::<StdResult<Vec<_>>>()?
//             .into_iter()
//             .filter(|nft| !nft.is_end_reward && nft.lockup_term.value == term.value)
//             .collect::<Vec<_>>();
//             nft_list.sort_by(|a, b| a.end_time.clone().cmp(&b.end_time.clone()));

//             // dem so luong
//  let mut count_staked: u128 = 0;
//                 for n in nft_list.clone().iter() {
//                     if n.end_time.clone() >= nft.end_time
//                         && n.lockup_term.value.clone() == nft.lockup_term.value.clone()
//                     {
//                         count_staked += 1;
//                     }
//                 }
//         }

        let mut nft_list = (0..nft_key.clone())
            .map(|key| NFTS.load(deps.storage, key + 1))
            .collect::<StdResult<Vec<_>>>()?
            .into_iter()
            .filter(|nft| !nft.is_end_reward)
            .collect::<Vec<_>>();
        nft_list.sort_by(|a, b| a.end_time.clone().cmp(&b.end_time.clone()));

        let reward_per_second = campaign_info.reward_per_second.clone();
        let mut time_calc = 0;

        for nft in nft_list.clone().iter_mut() {
            if nft.end_time.clone() <= env.block.time.seconds() {
                // count staked
                let mut count_staked: u128 = 0;
                for n in nft_list.clone().iter() {
                    if n.end_time.clone() >= nft.end_time
                        && n.lockup_term.value.clone() == nft.lockup_term.value.clone()
                    {
                        count_staked += 1;
                    }
                }
                // reward in time_calc -> nft end_time
                nft.pending_reward += reward_per_second
                    * Uint128::from((nft.end_time - nft.time_calc) as u128)
                    * nft.lockup_term.percent
                    / Uint128::new(100)
                    / Uint128::from(count_staked);
                time_calc = nft.end_time; // update time_calc
                nft.is_end_reward = true; // nft stake timeout
            } else {
                let mut count_staked: u128 = 0;
                for n in nft_list.clone().iter() {
                    if n.end_time >= env.block.time.seconds()
                        && n.lockup_term.value.clone() == nft.lockup_term.value.clone()
                    {
                        count_staked += 1;
                    }
                }

                if time_calc != 0 {
                    nft.time_calc = time_calc;
                }
                // reward in time_calc -> nft time now
                nft.pending_reward += reward_per_second
                    * Uint128::from((env.block.time.seconds() - nft.time_calc) as u128)
                    * nft.lockup_term.percent
                    / Uint128::new(100)
                    / Uint128::from(count_staked);
                nft.time_calc = env.block.time.seconds(); // update time_calc
            }
            // save nfts
            NFTS.save(deps.storage, nft.key.clone(), &nft)?;
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
    nfts: Vec<UnStakeNft>,
) -> Result<Response, ContractError> {
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    // prepare response
    let mut res = Response::new();

    // update pending reward for all nft
    let nft_key = NUMBER_OF_NFTS.load(deps.storage)?;
    let mut nft_list = (0..nft_key.clone())
        .map(|key| NFTS.load(deps.storage, key + 1))
        .collect::<StdResult<Vec<_>>>()?
        .into_iter()
        .filter(|nft| !nft.is_end_reward)
        .collect::<Vec<_>>();
    nft_list.sort_by(|a, b| a.end_time.clone().cmp(&b.end_time.clone()));

    let reward_per_second = campaign_info.reward_per_second.clone();
    let mut time_calc = 0;

    for nft in nft_list.clone().iter_mut() {
        if nft.end_time.clone() <= env.block.time.seconds() {
            // count staked
            let mut count_staked: u128 = 0;
            for n in nft_list.clone().iter() {
                if n.end_time.clone() >= nft.end_time
                    && n.lockup_term.value.clone() == nft.lockup_term.value.clone()
                {
                    count_staked += 1;
                }
            }
            // reward in time_calc -> nft end_time
            nft.pending_reward += reward_per_second
                * Uint128::from((nft.end_time - nft.time_calc) as u128)
                * nft.lockup_term.percent
                / Uint128::new(100)
                / Uint128::from(count_staked);
            time_calc = nft.end_time; // update time_calc
            nft.is_end_reward = true; // nft stake timeout
        } else {
            let mut count_staked: u128 = 0;
            for n in nft_list.clone().iter() {
                if n.end_time >= env.block.time.seconds()
                    && n.lockup_term.value.clone() == nft.lockup_term.value.clone()
                {
                    count_staked += 1;
                }
            }

            if time_calc != 0 {
                nft.time_calc = time_calc;
            }
            // reward in time_calc -> nft time now
            nft.pending_reward += reward_per_second
                * Uint128::from((env.block.time.seconds() - nft.time_calc) as u128)
                * nft.lockup_term.percent
                / Uint128::new(100)
                / Uint128::from(count_staked);
            nft.time_calc = env.block.time.seconds(); // update time_calc
        }
        // save nfts
        NFTS.save(deps.storage, nft.key.clone(), &nft)?;
    }

    // check the owner of token_ids, all token_ids should be owned by the contract
    for nft in nfts.iter() {
        let nft_info = NFTS.may_load(deps.storage, nft.nft_key.clone())?;

        // check time unstake and owner nft
        match nft_info.clone() {
            Some(info) => {
                if info.end_time > env.block.time.seconds() {
                    return Err(ContractError::NotAvailableForUnStake {});
                }
            }
            None => return Err(ContractError::NotAvailableForUnStake {}),
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
        staker.reward_debt += nft_info.clone().unwrap().pending_reward;
        staker.nft_keys.retain(|&key| key != nft.nft_key); // remove nft for staker
        STAKERS_INFO.save(deps.storage, info.sender.clone(), &staker)?;

        // update nft
        let mut update_nft = nft_info.clone().unwrap();
        update_nft.is_unstake = true;
        NFTS.save(deps.storage, nft.nft_key.clone(), &update_nft)?;

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
        ("nfts", &format!("{:?}", &nfts)),
    ]))
}

pub fn execute_claim_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    // load campaign info
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    let mut staker_info = STAKERS_INFO
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or(StakerRewardAssetInfo {
            nft_keys: vec![],
            reward_debt: Uint128::zero(),
            reward_claimed: Uint128::zero(),
        });
    // if staker_info.reward_debt.clone() == Uint128::zero(){
    //     return  Err(ContractError::Unauthorized {  });
    // }

    // update pending reward
    let nft_key = NUMBER_OF_NFTS.load(deps.storage)?;

    let mut nft_list = (0..nft_key.clone())
        .map(|key| NFTS.load(deps.storage, key + 1))
        .collect::<StdResult<Vec<_>>>()?
        .into_iter()
        .filter(|nft| !nft.is_end_reward)
        .collect::<Vec<_>>();
    nft_list.sort_by(|a, b| a.end_time.clone().cmp(&b.end_time.clone()));

    let reward_per_second = campaign_info.reward_per_second.clone();
    let mut time_calc = 0;

    for nft in nft_list.clone().iter_mut() {
        if nft.end_time.clone() <= env.block.time.seconds() {
            // count staked
            let mut count_staked: u128 = 0;
            for n in nft_list.clone().iter() {
                if n.end_time.clone() >= nft.end_time
                    && n.lockup_term.value.clone() == nft.lockup_term.value.clone()
                {
                    count_staked += 1;
                }
            }
            // reward in time_calc -> nft end_time
            nft.pending_reward += reward_per_second
                * Uint128::from((nft.end_time - nft.time_calc) as u128)
                * nft.lockup_term.percent
                / Uint128::new(100)
                / Uint128::from(count_staked);
            time_calc = nft.end_time; // update time_calc
            nft.is_end_reward = true; // nft stake timeout
        } else {
            let mut count_staked: u128 = 0;
            for n in nft_list.clone().iter() {
                if n.end_time >= env.block.time.seconds()
                    && n.lockup_term.value.clone() == nft.lockup_term.value.clone()
                {
                    count_staked += 1;
                }
            }

            if time_calc != 0 {
                nft.time_calc = time_calc;
            }
            // reward in time_calc -> nft time now
            nft.pending_reward += reward_per_second
                * Uint128::from((env.block.time.seconds() - nft.time_calc) as u128)
                * nft.lockup_term.percent
                / Uint128::new(100)
                / Uint128::from(count_staked);
            nft.time_calc = env.block.time.seconds(); // update time_calc
        }
        // save nfts
        NFTS.save(deps.storage, nft.key.clone(), &nft)?;
    }

    // calc pending reward for staker
    for key in staker_info.nft_keys.iter() {
        let mut nft = NFTS.load(deps.storage, key.clone())?;
        if !nft.clone().is_unstake {
            staker_info.reward_debt += nft.pending_reward;
        }
        //update pending reward for nft = 0
        nft.pending_reward = Uint128::zero();
        NFTS.save(deps.storage, key.clone(), &nft)?;
    }

    match campaign_info.reward_token_info.info.clone() {
        TokenInfo::Token { contract_addr } => {
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
                    if balance.balance < staker_info.reward_debt.clone() {
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
                    amount: staker_info.reward_debt.clone(),
                })?,
                funds: vec![],
            });

            // update staker info
            staker_info.reward_claimed += staker_info.reward_debt.clone();
            staker_info.reward_debt = Uint128::zero();
            STAKERS_INFO.save(deps.storage, info.sender.clone(), &staker_info)?;

            // update reward total and reward claimed for campaign
            let mut update_campaign_info = campaign_info.clone();
            update_campaign_info.reward_token_info.amount -= staker_info.reward_debt.clone();
            update_campaign_info.total_reward_claimed += staker_info.reward_debt.clone();
            CAMPAIGN_INFO.save(deps.storage, &update_campaign_info)?;

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
        TokenInfo::NativeToken { denom } => {
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
    allowed_collection: String, // staking collection nft
    reward_token_info: AssetTokenInfo,
    lockup_term: Vec<LockupTerm>, // flexible, 15days, 30days, 60days
) -> Result<Response, ContractError> {
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
    // permission check
    if info.sender != campaign_info.owner {
        return Err(ContractError::Unauthorized {});
    }

    // time check
    if campaign_info.start_time <= env.block.time.seconds() {
        return Err(ContractError::NotAvailableForUpdate {});
    }

    //get total nft in collection
    let mut total_nft_collection = campaign_info.total_nft_collection;
    if allowed_collection != campaign_info.allowed_collection {
        let num_tokens_response: StdResult<cw721::NumTokensResponse> =
            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: allowed_collection.clone(),
                msg: to_binary(&Cw721QueryMsg::NumTokens {})?,
            }));
        total_nft_collection = num_tokens_response?.count;
    }

    let campaign_info = CampaignInfo {
        owner: campaign_info.owner.clone(),
        campaign_name: campaign_name.clone(),
        campaign_image: campaign_image.clone(),
        campaign_description: campaign_description.clone(),
        start_time: start_time.clone(),
        end_time: end_time.clone(),
        total_reward_claimed: campaign_info.total_reward_claimed.clone(),
        total_reward: campaign_info.total_reward.clone(),
        limit_per_staker: limit_per_staker.clone(),
        reward_token_info: reward_token_info.clone(),
        allowed_collection: deps.api.addr_validate(&allowed_collection).unwrap(),
        total_nft_collection,
        lockup_term: lockup_term.clone(),
        reward_per_second: campaign_info.reward_per_second.clone(),
    };

    // store campaign info
    CAMPAIGN_INFO.save(deps.storage, &campaign_info)?;

    Ok(Response::new().add_attributes([
        ("action", "update_campaign"),
        ("owner", &campaign_info.owner.to_string()),
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
            &campaign_info.allowed_collection.to_string(),
        ),
        ("lockup_term", &format!("{:?}", &campaign_info.lockup_term)),
        ("start_time", &campaign_info.start_time.to_string()),
        ("end_time", &campaign_info.end_time.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::CampaignInfo {} => Ok(to_binary(&query_campaign_info(deps)?)?),
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
        QueryMsg::TotalStaked {} => Ok(to_binary(&query_total_nft_staked(deps)?)?),
        QueryMsg::RewardPerSecond {} => Ok(to_binary(&query_reward_per_second(deps)?)?),
    }
}

fn query_campaign_info(deps: Deps) -> Result<CampaignInfoResult, ContractError> {
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;
    let nft_staked: u64 = NUMBER_OF_NFTS.load(deps.storage)?;

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
        total_nft_collection: campaign_info.total_nft_collection.clone(),
    };
    Ok(campaign_result)
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
    let nft_key = NUMBER_OF_NFTS.load(deps.storage)?;

    let mut nft_list = (0..nft_key.clone())
        .map(|key| NFTS.load(deps.storage, key + 1))
        .collect::<StdResult<Vec<_>>>()?;

    nft_list.sort_by(|a, b| a.end_time.clone().cmp(&b.end_time.clone()));

    let reward_per_second = campaign_info.reward_per_second.clone();
    let mut time_calc = 0;
    for nft in nft_list.clone().iter_mut() {
        if !nft.is_end_reward{
            if nft.end_time.clone() <= env.block.time.seconds() {
                // count staked
                let mut count_staked: u128 = 0;
                for n in nft_list.clone().iter() {
                    if !n.is_end_reward{
                        if n.end_time.clone() >= nft.end_time
                        && n.lockup_term.value.clone() == nft.lockup_term.value.clone()
                    {
                        count_staked += 1;
                    }
                    }
                }
                // reward in time_calc -> nft end_time
                nft.pending_reward += reward_per_second
                    * Uint128::from((nft.end_time - nft.time_calc) as u128)
                    * nft.lockup_term.percent
                    / Uint128::new(100)
                    / Uint128::from(count_staked);
                time_calc = nft.end_time; // update time_calc
                nft.is_end_reward = true; // nft stake timeout
            } else {
                let mut count_staked: u128 = 0;
                for n in nft_list.clone().iter() {
                    if !n.is_end_reward{
                        if n.end_time >= env.block.time.seconds()
                        && n.lockup_term.value.clone() == nft.lockup_term.value.clone()
                    {
                        count_staked += 1;
                    }
                    }
                }

                if time_calc != 0 {
                    nft.time_calc = time_calc;
                }
                // reward in time_calc -> nft time now
                nft.pending_reward += reward_per_second
                    * Uint128::from((env.block.time.seconds() - nft.time_calc) as u128)
                    * nft.lockup_term.percent
                    / Uint128::new(100)
                    / Uint128::from(count_staked);
                nft.time_calc = env.block.time.seconds(); // update time_calc
            }
        }

        if staker_asset.nft_keys.contains(&nft.key) {
            staked_info.nfts.push(nft.clone());
            staked_info.reward_debt += nft.pending_reward.clone();
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
        nfts = nfts
            .iter()
            .filter(|x| x.owner_nft == owner)
            .cloned()
            .collect();
    }
    Ok(nfts)
}

fn query_total_nft_staked(deps: Deps) -> Result<u64, ContractError> {
    let total_nfts = NUMBER_OF_NFTS.load(deps.storage)?;
    Ok(total_nfts)
}

fn query_reward_per_second(deps: Deps) -> Result<Uint128, ContractError> {
    let campaign_info = CAMPAIGN_INFO.load(deps.storage)?;
    Ok(campaign_info.reward_per_second)
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
