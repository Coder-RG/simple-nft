//! This module implements `instantiate`, `execute` and `query`.
//! These actions are performed using *wasmd*.

// #[cfg(not(feature = "library"))]
use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

use cw2::set_contract_version;
use cw721::Expiration;

use crate::msg::{AskingPriceResponse, NumTokensResponse, OwnerOfResponse};
// use crate::msg::{ApprovedResponse, AskingPriceResponse};
use crate::state::{State, TokenInfo, CONFIG, TOKENS};
use crate::{
    msg::{Approval, ExecuteMsg, InstantiateMsg, MintMsg, QueryMsg},
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
        ExecuteMsg::Approve {
            operator,
            token_id,
            expires,
        } => handle_approve(deps, env, info, operator.as_str(), token_id, expires),
        // ExecuteMsg::ApproveAll { .. } => handle_approve_all(deps, env, info, msg),
        ExecuteMsg::Revoke { operator, token_id } => {
            handle_revoke(deps, env, info, operator, token_id)
        }
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

    // if sender is not the onwer or an approved operator, return Err
    if requested_token.owner != info.sender {
        if let Some(val) = requested_token.approvals {
            if info.sender != val.operator {
                return Err(ContractError::Unauthorized);
            }
        } else {
            return Err(ContractError::Unauthorized);
        }
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

pub fn handle_approve(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: &str,
    token_id: u64,
    expires: Option<Expiration>,
) -> Result<Response, ContractError> {
    // Load the token with given token id
    let mut token = query_tokens(deps.as_ref(), token_id);
    if info.sender != token.owner {
        return Err(ContractError::Unauthorized);
    };
    let appr = Approval {
        operator: deps.api.addr_validate(operator)?,
        expires: match expires {
            Some(val) => val,
            None => Expiration::Never {},
        },
    };
    // Apply approval to the token
    token.approvals = Some(appr);
    TOKENS.save(deps.storage, token_id, &token)?;

    Ok(Response::new()
        .add_attribute("action", "approve")
        .add_attribute("from", info.sender)
        .add_attribute("approved", operator)
        .add_attribute("token_id", token_id.to_string()))
}

fn handle_revoke(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: String,
    token_id: u64,
) -> Result<Response, ContractError> {
    let mut token = query_tokens(deps.as_ref(), token_id);
    if info.sender != token.owner {
        return Err(ContractError::Unauthorized);
    } else if token.approvals.is_none() {
        return Err(ContractError::ApprovalNotFound { operator });
    }
    let revoked = token.approvals.unwrap();
    token.approvals = None;
    TOKENS.save(deps.storage, token_id, &token)?;
    Ok(Response::new()
        .add_attribute("action", "revoke")
        .add_attribute("from", info.sender)
        .add_attribute("revoked", revoked.operator)
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
    TOKENS.save(deps.storage, num_tokens, &token)?;

    // Increase the number of tokens issued in state
    config.num_tokens = num_tokens;
    CONFIG.save(deps.storage, &config)?;

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

        QueryMsg::OwnerOf {
            token_id,
            include_expired,
        } => query_owner_of(deps, env, token_id, include_expired),

        QueryMsg::NumTokens {} => query_num_tokens(deps, env),
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

fn query_owner_of(
    deps: Deps,
    env: Env,
    token_id: u64,
    include_expired: Option<bool>,
) -> StdResult<Binary> {
    let token = query_tokens(deps, token_id);

    let result = OwnerOfResponse {
        owner: token.owner.into_string(),
        approvals: match token.approvals {
            Some(val) => {
                if !val.expires.is_expired(&env.block) || include_expired.unwrap_or(false) {
                    Some(val)
                } else {
                    None
                }
            }
            None => None,
        },
    };

    to_binary(&result)
}

fn query_num_tokens(deps: Deps, _env: Env) -> StdResult<Binary> {
    let config = query_config(deps);
    to_binary(&NumTokensResponse {
        tokens: config.num_tokens,
    })
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
    use cosmwasm_std::{coins, from_binary, Addr, Coin, Uint128};

    const DENOM: &str = "ubit";

    fn init_msg(name: String, symbol: String) -> InstantiateMsg {
        InstantiateMsg { name, symbol }
    }

    fn mint_msg(owner: String) -> MintMsg {
        MintMsg {
            owner,
            token_uri: None,
            price: coins(1000, &DENOM.to_string()),
        }
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
    fn approval() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(0, &DENOM.to_string()));

        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let mint_msg = mint_msg("owner1".to_string());
        let _res = handle_mint(deps.as_mut(), env.clone(), info.clone(), mint_msg).unwrap();

        let approve_msg = ExecuteMsg::Approve {
            operator: "operator".to_string(),
            token_id: 1u64,
            expires: None,
        };

        let info = mock_info("owner1", &coins(0, &DENOM.to_string()));
        let res = execute(deps.as_mut(), env.clone(), info.clone(), approve_msg).unwrap();

        assert_eq!(res.messages.len(), 0);
        assert_eq!(res.attributes.len(), 4);

        let token = query_tokens(deps.as_ref(), 1u64);

        assert_eq!(
            token.approvals.unwrap().operator,
            Addr::unchecked("operator")
        );
    }

    #[test]
    fn revoke() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(0, &DENOM.to_string()));

        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Mint a new token
        let mint_msg = mint_msg("owner1".to_string());
        let _res = handle_mint(deps.as_mut(), env.clone(), info.clone(), mint_msg).unwrap();
        let approve_msg = ExecuteMsg::Approve {
            operator: "operator".to_string(),
            token_id: 1u64,
            expires: None,
        };
        let info = mock_info("owner1", &coins(0, &DENOM.to_string()));
        execute(deps.as_mut(), env.clone(), info.clone(), approve_msg).unwrap();

        let token = query_tokens(deps.as_ref(), 1u64);
        assert_eq!(
            token.approvals.unwrap().operator,
            Addr::unchecked("operator")
        );

        // Revoke approval
        let revoke_msg = ExecuteMsg::Revoke {
            operator: "operator".to_string(),
            token_id: 1u64,
        };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), revoke_msg).unwrap();
        assert_eq!(res.messages.len(), 0);
        assert_eq!(res.attributes.len(), 4);

        let token = query_tokens(deps.as_ref(), 1u64);
        assert_eq!(token.approvals, None);
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

        let mint_msg = mint_msg("creator".to_string());

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
    fn transfer_nft_by_owner() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let info = mock_info("minter", &coins(0u128, &DENOM.to_string()));
        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let mint_msg = mint_msg("creator".to_string());

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

    #[test]
    fn transfer_nft_by_operator() {
        // Setup the necessary environment
        let mut deps = mock_dependencies();
        let env = mock_env();

        let info = mock_info("minter", &coins(0u128, &DENOM.to_string()));
        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let _res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let mint_msg = mint_msg("creator".to_string());

        let msg = ExecuteMsg::Mint(mint_msg);
        let res: Response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(4, res.attributes.len());

        // Add approval to the token
        let mut token = query_tokens(deps.as_ref(), 1u64);
        token.approvals = Some(Approval {
            operator: Addr::unchecked("operator"),
            expires: Expiration::Never {},
        });
        TOKENS.save(&mut deps.storage, 1u64, &token).unwrap();

        // *operator* should not be capable of transferring the token
        let info = mock_info("operator", &coins(0u128, &DENOM.to_string()));
        let msg = ExecuteMsg::TransferNft {
            recipient: String::from("recipient"),
            token_id: 1,
        };
        let res: Response = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(4, res.attributes.len());
    }

    #[test]
    fn asking_price() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let info = mock_info("minter", &coins(0u128, &DENOM.to_string()));
        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        let mint_msg = mint_msg("creator".to_string());

        let msg = ExecuteMsg::Mint(mint_msg);
        let res: Response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(4, res.attributes.len());

        let query_msg = QueryMsg::AskingPrice { token_id: 1 };
        let res = query(deps.as_ref(), env, query_msg).unwrap();
        let res: AskingPriceResponse = from_binary(&res).unwrap();
        assert_eq!(
            res.price,
            Coin {
                amount: Uint128::from(1000u64),
                denom: DENOM.to_string()
            }
        );
    }

    #[test]
    fn owner_of() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let info = mock_info("minter", &coins(0u128, &DENOM.to_string()));
        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        let mint_msg = mint_msg("creator".to_string());
        let msg = ExecuteMsg::Mint(mint_msg);
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query_owner_of(deps.as_ref(), env, 1u64, None).unwrap();
        let res: OwnerOfResponse = from_binary(&res).unwrap();
        assert_eq!(res.owner, "creator");
        assert_eq!(res.approvals, None);
    }
}
