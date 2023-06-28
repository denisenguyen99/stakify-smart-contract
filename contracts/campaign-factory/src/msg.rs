use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128, Timestamp};

use crate::state::{ConfigResponse, FactoryCampaignInfo};
use campaign::state::RewardTokenInfo;

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
        owner: String,
        campaign_name:String,
        campaign_image:String,
        campaign_description:String,
        start_time: Timestamp,
        end_time: Timestamp,
        limit_per_staker:u64,
        reward_token_info: RewardTokenInfo,
        allowed_collection: Addr,
        lockup_term: Uint128,
        reward_per_second:Uint128,
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
