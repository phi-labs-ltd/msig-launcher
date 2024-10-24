use crate::state::MSig;
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Instantiate {
        label: String,
        name: String,
        description: String,
        image_url: Option<String>,
        /// Time in seconds
        max_voting_period: u64,
        members: Vec<cw4::Member>,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(MSig)]
    MSig { label: String },
}
