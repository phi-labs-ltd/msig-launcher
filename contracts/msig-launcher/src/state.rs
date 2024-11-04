use crate::ContractError;
use cosmwasm_std::{Addr, Attribute};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialOrd, Eq, Clone, Debug, PartialEq, JsonSchema, Default)]

pub struct MSigCodeIds {
    pub main: u64,
    pub voting: u64,
    pub proposal: u64,
    pub pre_proposal: u64,
    pub cw4: u64,
}

/// Easy helper for building the multisig wallet data
#[derive(Default)]
pub struct MSigBuilder {
    pub dao_dao_contract: Option<String>,
    pub voting_contract: Option<String>,
    pub proposal_contract: Option<String>,
    pub pre_propose_contract: Option<String>,
    pub cw4_contract: Option<String>,
}

impl MSigBuilder {
    pub fn set_contract(
        &mut self,
        code_ids: &MSigCodeIds,
        code_id: u64,
        address: String,
    ) -> Result<(), ContractError> {
        if code_id == code_ids.main {
            self.dao_dao_contract = Some(address);
        } else if code_id == code_ids.voting {
            self.voting_contract = Some(address);
        } else if code_id == code_ids.proposal {
            self.proposal_contract = Some(address);
        } else if code_id == code_ids.pre_proposal {
            self.pre_propose_contract = Some(address);
        } else if code_id == code_ids.cw4 {
            self.cw4_contract = Some(address);
        } else {
            return Err(ContractError::UnknownContract { code_id, address });
        }

        Ok(())
    }

    pub fn build(self) -> Result<MSig, ContractError> {
        Ok(MSig {
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
    pub dao_dao_contract: String,
    pub voting_contract: String,
    pub proposal_contract: String,
    pub pre_propose_contract: String,
    pub cw4_contract: String,
}

impl MSig {
    pub fn append_attrs(&self, creator: &[Addr], events: &mut Vec<Attribute>) {
        let mut iter = creator.iter();

        events.push(("creator", iter.next().unwrap()).into());
        for addr in iter {
            events.push(("member", addr).into());
        }
        events.push(("dao_dao_address", self.dao_dao_contract.to_string()).into());
        events.push(("voting_address", self.voting_contract.to_string()).into());
        events.push(("proposal_address", self.proposal_contract.to_string()).into());
        events.push(("pre_propose_address", self.pre_propose_contract.to_string()).into());
        events.push(("cw4_address", self.cw4_contract.to_string()).into());
    }
}

pub static MSIG_CODE_IDS: Item<MSigCodeIds> = Item::new("msig_code_ids");
pub static PENDING_MSIG: Item<(Vec<Addr>, u64)> = Item::new("pending_msig");
pub static MSIG: Map<(Addr, u64), MSig> = Map::new("msig");
