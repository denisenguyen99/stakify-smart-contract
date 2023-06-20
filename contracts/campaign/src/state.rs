use cosmwasm_schema::cw_serde; // attribute macro to (de)serialize and make schemas
use cosmwasm_std::{Addr, Uint128, Timestamp}; // address type
use cw_storage_plus::{Item, Map}; // analog of Singletons for storage

#[cw_serde]
pub enum RewardTokenInfo {
    Token {
        contract_addr: Addr,
        amount: Uint128,
    },
    NativeToken {
        denom: String,
        amount: Uint128,
    },
}

#[cw_serde]
pub struct CampaignInfo {
    pub owner: Addr,
    pub allowed_collection: Addr,
    pub reward_token_info: RewardTokenInfo,
    pub reward_per_second: Uint128,
    pub start_time: Uint128,
    pub end_time: Uint128,
}

pub const CAMPAIGN_INFO: Item<CampaignInfo> = Item::new("campaign_info");

#[cw_serde]
pub struct StakerRewardAssetInfo {
    pub token_ids: Vec<String>, // Current staker NFTs
    pub reward_debt: Uint128,   // Reward debt.
}
#[cw_serde]
pub struct StakedNFT {
    pub owner: Addr,
    pub token_ids: Vec<String>,
    pub staked_time: Timestamp,   // Reward debt.
}

/// Mappping from staker address to staked balance.
pub const STAKERS_INFO: Map<Addr, StakerRewardAssetInfo> = Map::new("stakers_info");

pub const STAKED_NFTS: Map<Addr, StakedNFT> = Map::new("staked_nft");
