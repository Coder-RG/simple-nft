//! This module implements `instantiate`, `execute` and `query`.
//! These actions are performed using *wasmd*.

// #[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

use cw2::set_contract_version;

use crate::msg::AskingPriceResponse;
// use crate::msg::{ApprovedResponse, AskingPriceResponse};
use crate::state::{State, TokenInfo, CONFIG, TOKENS};
use crate::{
    msg::{ExecuteMsg, InstantiateMsg, MintMsg, QueryMsg},
    ContractError,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:simple-nft";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialise a new instance of this contract.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    if msg.name.is_empty() {
        return Err(ContractError::CustomError {
            val: String::from("length of `name` should be greater than 1"),
        });
    }

    if msg.symbol.is_empty() {
        return Err(ContractError::CustomError {
            val: String::from("length of `symbol` should be greater than 1"),
        });
    }
    // sender will the minter for the time being
    let minter = deps.api.addr_validate(info.sender.as_str())?;

    // Configure the state for storing
    let config = State {
        name: msg.name,
        symbol: msg.symbol,
        minter,
        num_tokens: 0u64,
    };
    // Store the configured state
    CONFIG.save(deps.storage, &config)?;
    // Return an Ok() response as everything went well
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::TransferNft {
            recipient,
            token_id,
        } => handle_transfer_nft(deps, env, info, recipient, token_id),
        // ExecuteMsg::SendNft { .. } => handle_send_nft(deps, env, info, msg),
        // ExecuteMsg::Approve { .. } => handle_approve(deps, env, info, msg),
        // ExecuteMsg::ApproveAll { .. } => handle_approve_all(deps, env, info, msg),
        // ExecuteMsg::Revoke { .. } => handle_revoke(deps, env, info, msg),
        // ExecuteMsg::RevokeAll { .. } => handle_revoke_all(deps, env, info, msg),
        ExecuteMsg::Mint(msg) => handle_mint(deps, env, info, msg),
        _ => Err(ContractError::CustomError {
            val: String::from("Not implemented"),
        }),
    }
}

pub fn handle_transfer_nft(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: u64,
) -> Result<Response, ContractError> {
    let mut requested_token = TOKENS.load(deps.storage, token_id)?;

    if requested_token.owner != info.sender {
        return Err(ContractError::Unauthorized);
    }

    requested_token.owner = deps.api.addr_validate(&recipient)?;
    requested_token.approvals = None;

    TOKENS.save(deps.storage, token_id, &requested_token)?;

    Ok(Response::new()
        .add_attribute("action", "TransferNFT")
        .add_attribute("from", info.sender)
        .add_attribute("to", recipient)
        .add_attribute("token_id", token_id.to_string()))
}

pub fn handle_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MintMsg,
) -> Result<Response, ContractError> {
    // Load current contract state
    let mut config = query_config(deps.as_ref());

    // sender and minter address should be same
    if info.sender != config.minter {
        return Err(ContractError::Unauthorized);
    }

    // price of the new NFT cannot be zero
    if msg.price[0].amount.is_zero() {
        return Err(ContractError::CustomError {
            val: String::from("Token price cannot be zero"),
        });
    }

    // Increase the current amount of tokens issued
    let num_tokens = config.num_tokens + 1;
    // Create a new token
    let token = TokenInfo {
        owner: deps.api.addr_validate(&msg.owner)?,
        approvals: None,
        token_uri: None,
        base_price: msg.price,
        token_id: num_tokens,
    };
    // Save the new token to storage
    TOKENS.save(deps.storage, num_tokens, &token).unwrap();

    // Increase the number of tokens issues in state
    config.num_tokens = num_tokens;
    CONFIG.save(deps.storage, &config).unwrap();

    Ok(Response::new()
        .add_attribute("action", "mint")
        .add_attribute("from", info.sender)
        .add_attribute("owner", msg.owner)
        .add_attribute("token_id", num_tokens.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AskingPrice { token_id } => query_asking_price(deps, env, token_id),
        _ => Err(StdError::NotFound {
            kind: String::from("Not Implemented"),
        }),
    }
}

pub fn query_asking_price(deps: Deps, _env: Env, token_id: u64) -> StdResult<Binary> {
    let token_info = query_tokens(deps, token_id);
    let response = AskingPriceResponse {
        price: token_info.base_price[0].clone(),
    };
    to_binary(&response)
}

pub fn query_config(deps: Deps) -> State {
    CONFIG.load(deps.storage).unwrap()
}

pub fn query_tokens(deps: Deps, token_id: u64) -> TokenInfo {
    TOKENS.load(deps.storage, token_id).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, Addr};

    const DENOM: &str = "ubit";

    fn init_msg(name: String, symbol: String) -> InstantiateMsg {
        InstantiateMsg { name, symbol }
    }

    #[test]
    fn proper_initialization() {
        // Create mock dependencies and environment
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(0, &DENOM.to_string()));

        // sample InstantiateMsg
        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());

        // Upon currect instantiation, there will be no response; as expected.
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Check if the correct values are stored.
        let stored_state = query_config(deps.as_ref());
        assert_eq!(stored_state.name, "TestNFT");
        assert_eq!(stored_state.symbol, "NFT");
        assert_eq!(stored_state.num_tokens, 0u64);
        assert_eq!(stored_state.minter, Addr::unchecked("creator"));

        // Following tests are to check correct error when no value is given
        // to either of the fields in InstantiateMsg.
        let msg = init_msg(String::new(), "NFT".to_string());
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();

        match res {
            ContractError::CustomError { .. } => {}
            e => panic!("{:?}", e),
        }

        let msg = init_msg(String::from("TestNFT"), String::new());
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap_err();

        match res {
            ContractError::CustomError { .. } => {}
            e => panic!("{:?}", e),
        }
    }

    #[test]
    fn mint() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let info = mock_info("minter", &coins(0u128, &DENOM.to_string()));
        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let stored_state = query_config(deps.as_ref());
        assert_eq!(stored_state.name, "TestNFT");
        assert_eq!(stored_state.symbol, "NFT");
        assert_eq!(stored_state.minter, Addr::unchecked("minter"));
        assert_eq!(stored_state.num_tokens, 0);

        let mint_msg = MintMsg {
            token_id: 0,
            owner: String::from("creator"),
            token_uri: None,
            price: coins(1000, DENOM.to_string()),
        };

        let msg = ExecuteMsg::Mint(mint_msg);
        let res: Response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(4, res.attributes.len());

        let stored_state = query_config(deps.as_ref());
        assert_eq!(stored_state.name, "TestNFT");
        assert_eq!(stored_state.symbol, "NFT");
        assert_eq!(stored_state.minter, Addr::unchecked("minter"));
        assert_eq!(stored_state.num_tokens, 1);

        let stored_tokens = query_tokens(deps.as_ref(), 1);
        assert_eq!(stored_tokens.owner, "creator");
        assert_eq!(stored_tokens.token_id, 1);
        assert_eq!(stored_tokens.base_price, coins(1000, DENOM.to_string()));
        assert_eq!(stored_tokens.approvals, None);
        assert_eq!(stored_tokens.token_uri, None);
    }

    #[test]
    fn transfer_nft() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let info = mock_info("minter", &coins(0u128, &DENOM.to_string()));
        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let mint_msg = MintMsg {
            token_id: 0,
            owner: String::from("creator"),
            token_uri: None,
            price: coins(1000, DENOM.to_string()),
        };

        let msg = ExecuteMsg::Mint(mint_msg);
        let res: Response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(4, res.attributes.len());

        let info = mock_info("creator", &coins(0u128, &DENOM.to_string()));
        let msg = ExecuteMsg::TransferNft {
            recipient: String::from("recipient"),
            token_id: 1,
        };
        let res: Response = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(4, res.attributes.len());
    }
}
