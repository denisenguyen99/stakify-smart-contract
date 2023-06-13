
use cosmwasm_std::{Addr, Uint128}; // address type
use cosmwasm_schema::cw_serde;  // attribute macro to (de)serialize and make schemas
use cw_storage_plus::{Item, Map}; // analog of Singletons for storage

#[cw_serde]
pub enum RewardTokenInfo {
    Token { contract_addr: String },
    NativeToken { denom: String },
}

#[cw_serde]
pub struct CampaignInfo {
    pub owner: String,
    pub collection: String,
    pub reward_token_address: String,
    pub reward_token_amount: Uint128,
    pub reward_token_available: Uint128,
    pub reward_per_second: Uint128,
    pub start_time: Uint128,
    pub end_time: Uint128,
}

pub const CAMPAIGN_INFO: Item<CampaignInfo> = Item::new("campaign_info");

#[cw_serde]
pub struct StakerRewardAssetInfo {
    pub nft: Uint128,      // Current staker NFT
    pub reward_debt: Uint128, // Reward debt.
}

/// Mappping from staker address to staker balance.
pub const STAKERS_INFO: Map<Addr, StakerRewardAssetInfo> = Map::new("stakers_info");
