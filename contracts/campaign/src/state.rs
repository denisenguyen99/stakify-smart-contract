use std::fmt;

use cosmwasm_schema::cw_serde; // attribute macro to (de)serialize and make schemas
use cosmwasm_std::{Addr, Timestamp, Uint128}; // address type
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
pub struct  LockupTerm {
    pub name: String,
    pub value: u64,
    // pub percent: u64
}



// impl fmt::Display for LockupTerm {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             LockupTerm::Fifteen { value } => write!(f, "{}", value),
//             LockupTerm::Thirty { value } => write!(f, "{}", value),
//             LockupTerm::Sixty { value } => write!(f, "{}", value),
//         }
//     }
// }

#[cw_serde]
pub struct CampaignInfo {
    // owner of campaign
    pub owner: Addr,

    // info detail
    pub campaign_name: String,
    pub campaign_image: String,
    pub campaign_description: String,
    pub start_time: Timestamp, // start time must be from T + 1
    pub end_time: Timestamp,   // max 3 years

    pub total_reward: Uint128,              // default 0
    pub total_reward_claimed: Uint128,              // default 0
    pub total_daily_reward: Uint128,              // default 0
    pub limit_per_staker: u64,              // max nft can stake
    // pub status: String,                     // pending | upcoming | active | ended
    pub reward_token_info: RewardTokenInfo, // reward token
    pub allowed_collection: Addr,           // staking collection nft
    pub lockup_term: Vec<LockupTerm>,            // 15days, 30days, 60days
    pub reward_per_second: Uint128,
}

#[cw_serde]
pub struct StakerRewardAssetInfo {
    pub nft_list: Vec<NftInfo>, // Current staker NFTs
    pub reward_debt: Uint128,       // can claim reward.
    pub reward_claimed: Uint128,
}

#[cw_serde]
pub struct NftInfo {
    pub token_id: String,
    pub owner_nft: Addr,
    pub pending_reward: Uint128,
    pub lockup_term: LockupTerm, // value = seconds
    pub time_calc: Timestamp,
    pub is_end_stake: bool,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}

// impl fmt::Display for NftInfo {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "{} , {} , {}", self.token_id, self.lockup_term, self.start_time)
//     }
// }

#[cw_serde]
pub struct NftStake {
    pub token_id: String,
    pub lockup_term: LockupTerm,
}

// campaign info
pub const CAMPAIGN_INFO: Item<CampaignInfo> = Item::new("campaign_info");

// Mapping from staker address to staked balance.
pub const STAKERS_INFO: Map<Addr, StakerRewardAssetInfo> = Map::new("stakers_info");

// list staker
pub const STAKERS: Item<Vec<Addr>> = Item::new("stakers");

// list nft staked
pub const NFT_STAKED: Item<Vec<String>> = Item::new("nft_stakers");
