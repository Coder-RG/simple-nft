//! This is a simple implementation of CosmWasm based Non-Fungible Token(NFT).
//! This mod does implement the CW721 spec. However, it is different than the
//! base implementation shown in [cw721-base]. All the functions
//! are implemented as indivisual functions in contract.rs, rather than
//! implementing it as a Cw721 trait.
//!
//! [cw721-base]: https://github.com/CosmWasm/cw-nfts/tree/main/contracts/cw721-base

pub mod contract;
mod error;
pub mod msg;
pub mod query;
pub mod state;

pub use crate::error::ContractError;
