use crate::ContractError;
use crate::{CW4_CODE_ID, MAIN_CODE_ID, PRE_PROPOSE_CODE_ID, PROPOSAL_CODE_ID, VOTING_CODE_ID};
use cosmwasm_std::{Addr, Attribute};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Easy helper for building the multisig wallet data
pub struct MSigBuilder {
    pub creator: Addr,
    pub dao_dao_contract: Option<String>,
    pub voting_contract: Option<String>,
    pub proposal_contract: Option<String>,
    pub pre_propose_contract: Option<String>,
    pub cw4_contract: Option<String>,
}

impl MSigBuilder {
    pub fn new(creator: Addr) -> Self {
        Self {
            creator,
            dao_dao_contract: None,
            voting_contract: None,
            proposal_contract: None,
            pre_propose_contract: None,
            cw4_contract: None,
        }
    }

    pub fn set_contract(&mut self, code_id: u64, address: String) -> Result<(), ContractError> {
        match code_id {
            MAIN_CODE_ID => self.dao_dao_contract = Some(address),
            VOTING_CODE_ID => self.voting_contract = Some(address),
            PROPOSAL_CODE_ID => self.proposal_contract = Some(address),
            PRE_PROPOSE_CODE_ID => self.pre_propose_contract = Some(address),
            CW4_CODE_ID => self.cw4_contract = Some(address),
            _ => return Err(ContractError::UnknownContract { code_id, address }),
        };

        Ok(())
    }

    pub fn build(self) -> Result<MSig, ContractError> {
        Ok(MSig {
            creator: self.creator,
            dao_dao_contract: self
                .dao_dao_contract
                .ok_or(ContractError::MissingContract("Dao Dao".to_string()))?,
            voting_contract: self
                .voting_contract
                .ok_or(ContractError::MissingContract("Voting".to_string()))?,
            proposal_contract: self
                .proposal_contract
                .ok_or(ContractError::MissingContract("Proposal".to_string()))?,
            pre_propose_contract: self
                .pre_propose_contract
                .ok_or(ContractError::MissingContract("Pre proposal".to_string()))?,
            cw4_contract: self
                .cw4_contract
                .ok_or(ContractError::MissingContract("Cw4".to_string()))?,
        })
    }
}

#[derive(Serialize, Deserialize, PartialOrd, Eq, Clone, Debug, PartialEq, JsonSchema)]
pub struct MSig {
    /// Multisig creator
    pub creator: Addr,
    pub dao_dao_contract: String,
    pub voting_contract: String,
    pub proposal_contract: String,
    pub pre_propose_contract: String,
    pub cw4_contract: String,
}

impl MSig {
    pub fn append_attrs(&self, events: &mut Vec<Attribute>) {
        events.push(("creator", self.creator.to_string()).into());
        events.push(("dao_dao_address", self.dao_dao_contract.to_string()).into());
        events.push(("voting_address", self.voting_contract.to_string()).into());
        events.push(("proposal_address", self.proposal_contract.to_string()).into());
        events.push(("pre_propose_address", self.pre_propose_contract.to_string()).into());
        events.push(("cw4_address", self.cw4_contract.to_string()).into());
    }
}

pub static PENDING_MSIG: Item<(String, Addr)> = Item::new("pending_msig");
pub static MSIG: Map<String, MSig> = Map::new("msig");
