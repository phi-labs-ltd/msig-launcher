pub mod contract;
mod error;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;

#[cfg(feature = "mainnet")]
pub const CODE_IDS: [u64; 5] = [7, 5, 4, 6, 3];

#[cfg(feature = "testing")]
pub const CODE_IDS: [u64; 5] = [1, 2, 3, 4, 5];

#[cfg(feature = "testnet")]
pub const CODE_IDS: [u64; 5] = [1, 2, 3, 4, 5];

#[cfg(feature = "devnet")]
pub const CODE_IDS: [u64; 5] = [2, 5, 4, 3, 1];

pub const MAIN_CODE_ID: u64 = CODE_IDS[0];
pub const VOTING_CODE_ID: u64 = CODE_IDS[1];
pub const PROPOSAL_CODE_ID: u64 = CODE_IDS[2];
pub const PRE_PROPOSE_CODE_ID: u64 = CODE_IDS[3];
pub const CW4_CODE_ID: u64 = CODE_IDS[4];
