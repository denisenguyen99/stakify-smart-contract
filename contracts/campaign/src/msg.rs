use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

use crate::state::{CampaignInfo, RewardTokenInfo};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub allowed_collection: String,
    pub reward_token_info: RewardTokenInfo,
    pub reward_per_second: Uint128,
    pub start_time: Uint128,
    pub end_time: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddRewardToken {},
    // user can stake 1 or many nfts to this campaign
    StakeNfts { token_ids: Vec<String> },
    // user can claim reward
    ClaimReward {},
    // user can unstake 1 or many nfts from this campaign
    UnstakeNfts { token_ids: Vec<String> },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CampaignInfo)]
    Campaign {},
}
