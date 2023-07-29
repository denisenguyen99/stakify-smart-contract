use cosmwasm_schema::cw_serde;
use cosmwasm_std:: Addr;
use cw_storage_plus::{Item, Map};


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
pub struct StakedCampaign {
    pub owner: Addr,
    pub campaign_addr: Addr,
    // pub campaign_name: String,
    // pub campaign_image: String,
    // pub campaign_description: String,
    // pub num_tokens: u64,
    // pub limit_per_staker: u64,
    // pub reward_token_info: AssetTokenInfo,
    // pub allowed_collection: Addr,
    // pub lockup_term: Vec<LockupTerm>,
    // pub start_time: u64,
    // pub end_time: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const CAMPAIGNS: Map<Addr, StakedCampaign> = Map::new("campaigns"); // Addr is Campaign address
pub const ADDR_CAMPAIGNS: Item<Vec<String>> = Item::new("addr_campaigns");

// #[cw_serde]
// pub struct CampaignInfoResponse {
//     pub owner: Addr,
//     pub campaign_addr: String,
//     pub campaign_name: String,
//     pub campaign_image: String,
//     pub campaign_description: String,
//     pub limit_per_staker: u64,
//     pub reward_token_info: AssetTokenInfo,
//     pub allowed_collection: Addr,
//     pub lockup_term: Vec<LockupTerm>,
//     pub start_time: u64,
//     pub end_time: u64,
// }
