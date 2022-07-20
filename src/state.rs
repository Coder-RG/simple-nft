//! Defines the state and tokeninfo structs

use crate::msg::Approval;
use cw721::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub name: String,
    pub symbol: String,
    pub minter: Addr,
    pub num_tokens: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo {
    /// Owner of the new token
    pub owner: Addr,
    /// Approvals to third-party to perform actions on this token
    pub approvals: Option<Approval>,
    /// Base price of this token
    pub base_price: Vec<Coin>,
    /// URI of NFT according to ERC 721 Metadata Schema
    pub token_uri: Option<String>,
    /// Unique token_id
    pub token_id: u64,
}

pub struct Storage<'a> {
    pub operator: Map<'a, (Addr, Addr), Expiration>,
    pub config: Item<'a, State>,
    pub tokens: Map<'a, u64, TokenInfo>,
}

pub const CONFIG: Item<State> = Item::new("config");
pub const TOKENS: Map<u64, TokenInfo> = Map::new("tokens");
pub const OPERATOR: Map<(Addr, Addr), Expiration> = Map::new("approvals");
