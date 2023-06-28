use std::fmt;

use cosmwasm_schema::cw_serde; // attribute macro to (de)serialize and make schemas
use cosmwasm_std::{Addr, Uint128, Timestamp}; // address type
use cw_storage_plus::{Item, Map}; // analog of Singletons for storage

#[cw_serde]
pub enum RewardTokenInfo {
    Token { contract_addr: String },
    NativeToken { denom: String },
}

impl fmt::Display for RewardTokenInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RewardTokenInfo::NativeToken { denom } => write!(f, "{}", denom),
            RewardTokenInfo::Token { contract_addr } => write!(f, "{}", contract_addr),
        }
    }
}

#[cw_serde]
pub struct CampaignInfo {
    // owner of campaign
    pub owner: Addr,

    // info detail
    pub campaign_name:String,
    pub campaign_image:String,
    pub campaign_description:String,
    pub start_time: Timestamp, // start time must be from T + 1
    pub end_time: Timestamp, // max 3 years

    pub total_reward: Uint128, // default 0
    pub limit_per_staker:u64,

    pub reward_token_info: RewardTokenInfo,  // reward token
    pub allowed_collection: Addr, // staking collection nft
    pub lockup_term:Uint128, // flexible, 15days, 30days, 60days
    pub reward_per_second: Uint128,
}

#[cw_serde]
pub struct StakerRewardAssetInfo {
    pub nft_list: Vec<NftInfo>, // Current staker NFTs
    pub reward_debt: Uint128,   // Reward debt.
}
#[cw_serde]
pub struct StakersCampaign {
    pub staker_address: Vec<Addr>,  // Reward debt.
}


#[cw_serde]
pub struct NftInfo {
    pub token_id: String,
    pub reward_time: Timestamp,
    pub stake_time: Timestamp,
}


// campaign info
pub const CAMPAIGN_INFO: Item<CampaignInfo> = Item::new("campaign_info");

// Mapping from staker address to staked balance.
pub const STAKERS_INFO: Map<Addr, StakerRewardAssetInfo> = Map::new("stakers_info");

// list staker
pub const STAKERS: Item<StakersCampaign> = Item::new("stakers");

