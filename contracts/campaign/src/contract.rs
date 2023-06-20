#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    has_coins, to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdResult, Uint128, WasmMsg, WasmQuery, CosmosMsg,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{CampaignInfo, RewardTokenInfo, CAMPAIGN_INFO, StakedNFT, STAKED_NFTS};
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
        allowed_collection: deps.api.addr_validate(&msg.allowed_collection).unwrap(),
        reward_token_info: msg.reward_token_info.clone(),
        reward_per_second: Uint128::zero(),
        start_time: msg.start_time,
        end_time: msg.end_time,
    };

    // store campaign info
    CAMPAIGN_INFO.save(deps.storage, &campaign)?;

    // we need emit the information of reward token to response
    let reward_token_info_str: String;
    let reward_token_amount_str: String;

    match msg.reward_token_info {
        RewardTokenInfo::Token {
            contract_addr,
            amount,
        } => {
            reward_token_info_str = contract_addr.to_string();
            reward_token_amount_str = amount.to_string();
        }
        RewardTokenInfo::NativeToken { denom, amount } => {
            reward_token_info_str = denom;
            reward_token_amount_str = amount.to_string();
        }
    }

    // emit the information of instantiated campaign
    Ok(Response::new().add_attributes([
        ("action", "instantiate"),
        ("owner", &msg.owner),
        ("allowed_collection", &msg.allowed_collection),
        ("reward_token_info", &reward_token_info_str),
        ("reward_token_amount", &reward_token_amount_str),
        ("reward_per_second", &msg.reward_per_second.to_string()),
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
        ExecuteMsg::AddRewardToken {} => execute_add_reward_token(deps, env, info),
        ExecuteMsg::StakeNfts { token_ids } => execute_stake_nft(deps, env, info, token_ids),
        ExecuteMsg::UnstakeNfts { token_ids } => execute_unstake_nft(deps, env, info, token_ids),
        ExecuteMsg::ClaimReward {} => execute_claim_reward(deps, env, info),
    }
}

pub fn execute_add_reward_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
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
        RewardTokenInfo::Token {
            contract_addr,
            amount,
        } => {
            // execute cw20 transfer msg from info.sender to contract
            let transfer_reward = CosmosMsg::Wasm(WasmMsg::Execute {
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
                    (
                        "allowed_collection",
                        campaign_info.allowed_collection.as_ref(),
                    ),
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
        RewardTokenInfo::NativeToken { denom, amount } => {
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
                (
                    "allowed_collection",
                    campaign_info.allowed_collection.as_ref(),
                ),
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
    // the length of token_ids should be smaller than 5 because of gas limit
    if token_ids.len() > 5 {
        return Err(ContractError::TooManyTokenIds {});
    }

    // TODO: check more condition of staking nft

    // load campaign info
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

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
        let transfer_nft_msg = WasmMsg::Execute {
            contract_addr: campaign_info.allowed_collection.to_string(),
            msg: to_binary(&Cw721ExecuteMsg::TransferNft {
                recipient: env.contract.address.to_string(),
                token_id: token_id.clone(),
            })?,
            funds: vec![],
        };

        if STAKED_NFTS.may_load(deps.storage, info.sender.clone())?.is_none(){
            let mut ids: Vec<String> = vec![];
            ids.push(token_id.clone());

            let staked = StakedNFT{
                owner:info.sender.clone(),
                staked_time:env.block.time,
                token_ids:ids
            };
            STAKED_NFTS.save(deps.storage, info.sender.clone(), &staked)?;
        }else{
            let mut staked: StakedNFT = STAKED_NFTS.load(deps.storage,info.sender.clone())?;
            if staked.token_ids.contains(token_id){
                return Err(ContractError::AlreadyExist {});
            }

            staked.token_ids.push(token_id.clone());
            STAKED_NFTS.save(deps.storage, info.sender.clone(), &staked)?;
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
    // Ok(Response::new())
    // the length of token_ids should be smaller than 5 because of gas limit
    if token_ids.len() > 5 {
        return Err(ContractError::TooManyTokenIds {});
    }

    // TODO: check more conditions for unstaking NFT
    if STAKED_NFTS.may_load(deps.storage, info.sender.clone())?.is_none(){
        return Err(ContractError::Unauthorized {});
    }

    // load campaign info
    let campaign_info: CampaignInfo = CAMPAIGN_INFO.load(deps.storage)?;

    // prepare response
    let mut res = Response::new();

    // check the owner of token_ids, all token_ids should be owned by the contract
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
        let mut staked_nft = STAKED_NFTS.load(deps.storage, info.sender.clone())?;
        staked_nft.token_ids.retain(|item|item != token_id);
        STAKED_NFTS.save(deps.storage, info.sender.clone(),&staked_nft)?;

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
