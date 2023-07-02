use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Timestamp, Addr};

use crate::state::{CampaignInfo, RewardTokenInfo, StakerRewardAssetInfo, LockupTerm, NftStake};

#[cw_serde]
pub struct InstantiateMsg {
    // owner of campaign
    pub owner: String,

    // info detail
    pub campaign_name:String,
    pub campaign_image:String,
    pub campaign_description:String,
    pub start_time: u64, // start time must be from T + 1
    pub end_time: u64, // max 3 years

    pub limit_per_staker: u64,
    // pub status: String, // pending | upcoming | active | ended
    pub reward_token_info: RewardTokenInfo,  // reward token
    pub allowed_collection: String, // staking collection nft
    pub lockup_term: Vec<LockupTerm>, // flexible, 15days, 30days, 60days
    pub reward_per_second: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddRewardToken {
        amount:Uint128
    },
    // user can stake 1 or many nfts to this campaign
    StakeNfts {
        nfts: Vec<NftStake>,
     },
    // user can claim reward
    ClaimReward {},
    // user can unstake 1 or many nfts from this campaign
    UnstakeNfts { token_ids: Vec<String> },

    // update campaign
    UpdateCampaign {
        campaign_name:String,
        campaign_image:String,
        campaign_description:String,
        start_time: u64, // start time must be from T + 1
        end_time: u64, // max 3 years
        limit_per_staker:u64,
        reward_token_info: RewardTokenInfo,  // reward token
        allowed_collection: String, // staking collection nft
        lockup_term:Vec<LockupTerm>, // flexible, 15days, 30days, 60days
     },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CampaignInfo)]
    Campaign {},

    #[returns(StakerRewardAssetInfo)]
    RewardInfo {
        owner:Addr
    },
}
