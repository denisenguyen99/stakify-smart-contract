use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Timestamp};
use cw_storage_plus::{Item, Map};

use campaign::state::{RewardTokenInfo, LockupTerm};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub campaign_code_id: u64,
}

// We define a custom struct for each query response
#[cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub campaign_code_id: u64,
}


#[cw_serde]
pub struct FactoryInfo {    
    pub owner: Addr,

    // info detail
    pub campaign_name: String,
    pub campaign_image: String,
    pub campaign_description: String,
    pub start_time: Timestamp, // start time must be from T + 1
    pub end_time: Timestamp,   // max 3 years

    pub limit_per_staker: u64,              // max nft can stake
    pub reward_token_info: RewardTokenInfo, // reward token
    pub allowed_collection: Addr,           // staking collection nft
    pub lockup_term: Vec<LockupTerm>,            // 15days, 30days, 60days

}

#[cw_serde]
pub struct FactoryCampaignInfo {    
    pub campaign_addr: Addr,
    pub campaign_info: FactoryInfo,
}


pub const CONFIG: Item<Config> = Item::new("config");
pub const CAMPAIGNS: Map<Addr, Vec<FactoryCampaignInfo>> = Map::new("campaigns");
pub const NUMBER_OF_CAMPAIGNS: Item<u64> = Item::new("number_of_campaign");
