use crate::state::{MSig, MSigCodeIds};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

pub const PAGINATION_DEFAULT: u8 = 25;
pub const PAGINATION_LIMIT: u8 = 100;

#[cw_serde]
pub struct InstantiateMsg {
    pub code_ids: MSigCodeIds,
}

#[cw_serde]
pub enum ExecuteMsg {
    Instantiate {
        name: String,
        description: String,
        image_url: Option<String>,
        /// Time in seconds
        max_voting_period: u64,
        members: Vec<cw4::Member>,
    },
}

#[cw_serde]
pub struct Pagination {
    pub user: Addr,
    pub limit: Option<u8>,
    // Page key
    pub start_at: Option<u64>,
}

#[cw_serde]
pub struct PageResult {
    pub data: Vec<MSig>,
    pub next: Option<u64>,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(MSigCodeIds)]
    CodeIds {},
    #[returns(PageResult)]
    MSigs { pagination: Pagination },
}
