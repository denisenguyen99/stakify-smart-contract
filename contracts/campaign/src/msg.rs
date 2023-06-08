use cosmwasm_schema::{cw_serde, QueryResponses};
use crate::state::RewardTokenInfo;

#[cw_serde]
pub struct InstantiateMsg {
    /// Reward Token address (CW20 or Native)
    pub collection: String,
    /// Reward Token address (CW20 or Native)
    pub reward_token: RewardTokenInfo,
    /// Start time
    pub start_time: u64,
    /// End time
    pub end_time: u64,
}

#[cw_serde]
pub enum ExecuteMsg {

}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CampaignInfo)]
    Campaign {}
}
