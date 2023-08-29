use crate::state::{ConfigResponse, CreateCampaign, FactoryCampaign};
use cosmwasm_schema::{cw_serde, QueryResponses};

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
    CreateCampaign { create_campaign: CreateCampaign },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},

    #[returns(FactoryCampaign)]
    Campaign { campaign_id: u64 },

    #[returns(Vec<FactoryCampaign>)]
    Campaigns {
        start_after: Option<u64>,
        limit: Option<u32>,
    },

    #[returns(Vec<String>)]
    CampaignAddrs {},
}
