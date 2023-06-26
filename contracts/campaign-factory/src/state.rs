use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Timestamp};
use cw_storage_plus::{Item, Map};

use campaign::state::RewardTokenInfo;

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
pub struct Info {    
    pub owner: Addr,
    pub allowed_collection: Addr,
    pub reward_token_info: RewardTokenInfo,
    pub reward_per_second: Uint128,
    pub staking_duration: u64,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}

#[cw_serde]
pub struct CampaignItemInfo {    
    pub campaign_addr: Addr,
    pub campaign_info: Info,
}


// We define a custom struct for storing pools info
#[cw_serde]
pub struct FactoryCampaignInfo {
    pub campaigns:Vec<CampaignItemInfo>,
}

// impl From<CampaignInfo> for FactoryCampaignInfo {
//     fn from(value: CampaignInfo) -> Self {
//         Self {
//             owner: value.owner,
//             allowed_collection: value.allowed_collection,
//             reward_token_info: value.reward_token_info,
//             reward_per_second: value.reward_per_second,
//             staking_duration: value.staking_duration,
//             start_time: value.start_time,
//             end_time: value.end_time,
//         }
//     }
// }

pub const CONFIG: Item<Config> = Item::new("config");
pub const CAMPAIGNS: Map<Addr, FactoryCampaignInfo> = Map::new("campaigns");
pub const NUMBER_OF_CAMPAIGNS: Item<u64> = Item::new("number_of_pools");
