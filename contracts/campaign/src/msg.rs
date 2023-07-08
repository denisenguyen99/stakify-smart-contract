use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Addr};

use crate::state::{CampaignInfo, StakedInfoResult, LockupTerm, NftStake, AssetTokenInfo, UnStakeNft, NftInfo};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,    // owner of campaign

    // info detail
    pub campaign_name:String,
    pub campaign_image:String,
    pub campaign_description:String,

    pub limit_per_staker: u64,
    pub reward_token_info: AssetTokenInfo,  // reward token
    pub allowed_collection: String, // staking collection nft
    pub lockup_term: Vec<LockupTerm>, // flexible, 15days, 30days, 60days

    pub start_time: u64, // start time must be from T + 1
    pub end_time: u64, // max 3 years
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
    UnstakeNfts { nfts: Vec<UnStakeNft> },

    // update campaign
    UpdateCampaign {
        campaign_name:String,
        campaign_image:String,
        campaign_description:String,
        limit_per_staker:u64,
        // reward_token_info: AssetTokenInfo,  // reward token
        allowed_collection: String, // staking collection nft
        lockup_term:Vec<LockupTerm>, // flexible, 15days, 30days, 60days

        start_time: u64, // start time must be from T + 1
        end_time: u64, // max 3 years
     },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CampaignInfo)]
    CampaignInfo {},

    #[returns(StakedInfoResult)]
    NftStaked {
        owner:Addr
    },

    #[returns(Vec<NftInfo>)]
    Nfts {
        start_after: Option<u64>,
        limit: Option<u32>,
        owner: Option<Addr>
    },

    #[returns(u64)]
    TotalStaked {},

    #[returns(Uint128)]
    RewardPerSecond {},
}
