use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Timestamp};

use crate::state::{ConfigResponse, FactoryCampaignInfo};
use campaign::state::{RewardTokenInfo, LockupTerm};

#[cw_serde]
pub struct InstantiateMsg {
    /// Campaign code ID
    pub campaign_code_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// UpdateConfig update relevant code IDs
    UpdateConfig {
        owner: Option<String>,
        campaign_code_id: Option<u64>,
    },
    /// CreateCampaign instantiates pair contract
    CreateCampaign {
        // info detail
        campaign_name:String,
        campaign_image:String,
        campaign_description:String,
        start_time: u64, // start time must be from T + 1
        end_time: u64, // max 3 years

        limit_per_staker:u64,
        // status: String, // pending | upcoming | active | ended
        reward_token_info: RewardTokenInfo,  // reward token
        allowed_collection: String, // staking collection nft
        lockup_term: Vec<LockupTerm>, // flexible, 15days, 30days, 60days
        reward_per_second: Uint128,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},

    #[returns(Vec<FactoryCampaignInfo>)]
    Campaign { campaign_owner: Addr },
    // #[returns(Vec<FactoryCampaignInfo>)]
    // Campaigns {
    //     start_after: Option<u64>,
    //     limit: Option<u32>,
    // },
}
