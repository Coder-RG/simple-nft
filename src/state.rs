use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub name: String,
    pub symbol: String,
    pub minter: Addr,
    pub num_tokens: u64,
}

pub const CONFIG: Item<State> = Item::new("config");
