use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    /// collection
    pub collection: String,
    /// Token denom
    pub reward_token_address: String,
    /// Token denom
    pub reward_token_amount: Uint128,
    /// Start time
    pub reward_token_available: Uint128,
    /// Start time
    pub start_time: u64,
    /// End time
    pub end_time: u64,
    /// owner
    pub owner: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    AddRewardToken {},
    // DepositNft {
    //     owner: String,
    //     nft: String,
    //     start: Uint128
    // },
    // ClaimReward {
    //     owner: String
    // },
    // WithdrawNft {
    //     owner: String,
    //     nft: String
    // }
}

#[cw_serde]
pub enum QueryMsg {
    // #[returns(CampaignInfo)]
    Campaign {}
}
