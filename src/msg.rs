//! Defines *InstantiateMsg*, *ExecuteMsg* and *QueryMsg*.

use cosmwasm_std::{Binary, Coin};
use cw721::{Approval, Expiration};
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
        token_id: String,
        msg: Binary,
    },
    /// Allows operator to transfer / send the token from the owner's account.
    /// If expiration is set, then this allowance has a time/height limit
    Approve {
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    },
    /// Remove previously granted Approval
    Revoke { spender: String, token_id: String },
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintMsg {
    /// Unique ID of the NFT
    pub token_id: u32,
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
        token_id: String,
        include_expired: Option<bool>,
    },

    // Return the operator, who has approval for all tokens of the given owner
    Approved {
        owner: String,
        operator: String,
    },

    // Return all operators with access to all of the given owner's tokens
    ApprovedForAll {
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
    NftInfo {},

    // Return NFT info and OwnerOf response.
    // Part of Metadata Extension
    AllNftInfo {
        token_id: String,
        include_expired: Option<bool>,
    },

    // Return tokens owned by given owner.
    // Part of Enumerable Extension
    Tokens {
        owner: String,
        start_after: Option<bool>,
        limit: Option<u32>,
    },

    // Return all tokens controlled by contract.
    // Part of Enumerable Extension
    AllTokens {
        start_after: Option<bool>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AskingPriceResponse {
    pub price: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnerOfResponse {
    pub owner: String,
    pub approvals: Vec<Approval>,
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
    pub tokens: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContractInfoResponse {
    pub name: String,
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NftInfoResponse<T> {
    pub token_uri: String,
    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllNftInfoResponse<T> {
    pub access: OwnerOfResponse,
    pub info: NftInfoResponse<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokensResponse {
    pub tokens: Vec<String>,
}
