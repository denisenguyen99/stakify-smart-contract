use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub collection: String,
    pub reward_token_address: String,
    pub reward_token_amount: Uint128,
    pub start_time: Uint128,
    pub end_time: Uint128,
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
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(CampaignInfo)]
    Campaign {}
}
