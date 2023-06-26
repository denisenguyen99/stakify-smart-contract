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
    pub owner: Addr,
    pub allowed_collection: Addr,
    pub reward_token_info: RewardTokenInfo,
    pub reward_per_second: Uint128,
    pub staking_duration: u64,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}

pub const CAMPAIGN_INFO: Item<CampaignInfo> = Item::new("campaign_info");

// #[cw_serde]
// pub struct StakerRewardAssetInfo {
//     pub token_ids: Vec<String>, // Current staker NFTs
//     pub reward_debt: Uint128,   // Reward debt.
// }

#[cw_serde]
pub struct NftInfo {
    pub token_id: String,
    pub reward_time: Timestamp,
    pub stake_time: Timestamp,
}

#[cw_serde]
pub struct StakedNFT {
    pub owner: Addr,
    pub status:String,
    pub token_ids: Vec<NftInfo>,
    pub reward_debt:Uint128
}

/// Mappping from staker address to staked balance.
// pub const STAKERS_INFO: Map<Addr, StakerRewardAssetInfo> = Map::new("stakers_info");

pub const STAKED_NFTS: Map<Addr, StakedNFT> = Map::new("staked_nft");
