//! Defines *InstantiateMsg*, *ExecuteMsg* and *QueryMsg*.

use cosmwasm_std::{Addr, Binary, Coin};
use cw721::Expiration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    /// Name of the NFT
    pub name: String,
    /// Symbol of the NFT
    pub symbol: String,
    // /// Minter has the permission to mint new tokens
    // pub minter: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Transfer is a base message to move a token to another account without triggering actions
    TransferNft { recipient: String, token_id: u64 },
    /// Send is a base message to transfer a token to a contract and trigger an action
    /// on the receiving contract.
    SendNft {
        contract: String,
        token_id: u64,
        msg: Binary,
    },
    /// Allows operator to transfer / send the token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    Approve {
        operator: String,
        token_id: u64,
        expires: Option<Expiration>,
    },
    /// Remove previously granted Approval
    Revoke { operator: String, token_id: u64 },
    /// Allows operator to transfer / send any token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    ApproveAll {
        operator: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted ApproveAll permission
    RevokeAll { operator: String },
    /// Mint a new token with the details as in MintMsg.
    Mint(MintMsg),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Approval {
    /// Account that can transfer/send the token
    pub operator: Addr,
    /// When the Approval expires (maybe Expiration::never)
    pub expires: Expiration,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintMsg {
    // /// Unique ID of the NFT
    // pub token_id: u64,
    /// The owner of the newly minter NFT
    pub owner: String,
    /// Universal resource identifier for this NFT
    /// Should point to a JSON file that conforms to the ERC721
    /// Metadata JSON Schema
    pub token_uri: Option<String>,
    /// Price of the token
    pub price: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // The price of the token
    AskingPrice {
        token_id: u64,
    },
    // Owner of the token
    OwnerOf {
        token_id: u64,
        include_expired: Option<bool>,
    },
    // Return the operator, who has approval for given token
    Approval {
        token_id: u64,
        operator: String,
        include_expired: Option<bool>,
    },
    // Return all operators with access to all of the given owner's tokens
    AllOperators {
        owner: String,
        include_expired: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    // Number of tokens issued thus far
    NumTokens {},
    // Return the contract info.
    // Part of Metadata Extension
    ContractInfo {},
    // Return NFT info.
    // Part of Metadata Extension
    NftInfo {
        token_id: u64,
    },
    // Return NFT info and OwnerOf response.
    // Part of Metadata Extension
    AllNftInfo {
        token_id: u64,
        include_expired: Option<bool>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AskingPriceResponse {
    pub price: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnerOfResponse {
    pub owner: String,
    pub approvals: Option<Approval>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApprovedResponse {
    pub approval: Approval,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApprovedForAllResponse {
    pub approval: Vec<Approval>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NumTokensResponse {
    pub tokens: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractInfoResponse {
    pub name: String,
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NftInfoResponse {
    pub token_uri: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllNftInfoResponse {
    pub owner: OwnerOfResponse,
    pub info: NftInfoResponse,
}
