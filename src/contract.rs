//! This module implements `instantiate`, `execute` and `query`.
//! These actions are performed using *wasmd*.

// #[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};

use cw2::set_contract_version;
use cw721::{Cw721ReceiveMsg, Expiration};

use crate::query::{query_config, query_tokens};
use crate::state::{State, TokenInfo, CONFIG, OPERATORS, TOKENS};
use crate::{
    msg::{Approval, ExecuteMsg, InstantiateMsg, MintMsg},
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
    // sender will be the minter for the time being
    let minter = deps.api.addr_validate(info.sender.as_str())?;

    // Configure the state for storing
    let config = State {
        name: msg.name,
        symbol: msg.symbol,
        minter,
        num_tokens: 0u64,
    };
    // Store
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

        ExecuteMsg::SendNft {
            contract,
            token_id,
            msg,
        } => handle_send_nft(deps, env, info, contract, token_id, msg),

        ExecuteMsg::Approve {
            operator,
            token_id,
            expires,
        } => handle_approve(deps, env, info, operator.as_str(), token_id, expires),

        ExecuteMsg::ApproveAll { operator, expires } => {
            handle_approve_all(deps, env, info, operator, expires)
        }

        ExecuteMsg::Revoke { operator, token_id } => {
            handle_revoke(deps, env, info, operator, token_id)
        }

        ExecuteMsg::RevokeAll { operator } => handle_revoke_all(deps, env, info, operator),

        ExecuteMsg::Mint(msg) => handle_mint(deps, env, info, msg),
    }
}

pub fn check_is_authorized(
    deps: Deps,
    env: &Env,
    info: &MessageInfo,
    token_id: u64,
) -> Result<(), ContractError> {
    let token = query_tokens(deps, token_id)?;
    if token.owner == info.sender {
        return Ok(());
    };

    let token_appr = token.approvals;
    if let Some(val) = token_appr {
        if val.operator == info.sender && !val.expires.is_expired(&env.block) {
            return Ok(());
        }
    };
    let super_appr = OPERATORS.may_load(deps.storage, (&token.owner, &info.sender))?;
    if let Some(val) = super_appr {
        if !val.is_expired(&env.block) {
            return Ok(());
        }
    };
    Err(ContractError::Unauthorized)
}

pub fn handle_transfer_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    token_id: u64,
) -> Result<Response, ContractError> {
    let mut requested_token = TOKENS.load(deps.storage, token_id)?;

    check_is_authorized(deps.as_ref(), &env, &info, token_id)?;

    requested_token.owner = deps.api.addr_validate(&recipient)?;
    requested_token.approvals = None;

    TOKENS.save(deps.storage, token_id, &requested_token)?;

    Ok(Response::new()
        .add_attribute("action", "transfer_nft")
        .add_attribute("from", info.sender)
        .add_attribute("to", recipient)
        .add_attribute("token_id", token_id.to_string()))
}

fn handle_send_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    token_id: u64,
    msg: Binary,
) -> Result<Response, ContractError> {
    handle_transfer_nft(deps, env, info.clone(), contract.clone(), token_id)?;

    let msg = Cw721ReceiveMsg {
        sender: info.sender.to_string(),
        token_id: token_id.to_string(),
        msg,
    };

    Ok(Response::new()
        .add_message(msg.into_cosmos_msg(contract.clone())?)
        .add_attribute("action", "send_nft")
        .add_attribute("from", info.sender)
        .add_attribute("to", contract)
        .add_attribute("token_id", token_id.to_string()))
}

pub fn handle_approve(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: &str,
    token_id: u64,
    expires: Option<Expiration>,
) -> Result<Response, ContractError> {
    // Load the token with given token id
    let mut token = query_tokens(deps.as_ref(), token_id)?;
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

    if appr.expires.is_expired(&env.block) {
        return Err(ContractError::Expired);
    }
    // Apply approval to the token
    token.approvals = Some(appr);
    TOKENS.save(deps.storage, token_id, &token)?;

    Ok(Response::new()
        .add_attribute("action", "approve")
        .add_attribute("from", info.sender)
        .add_attribute("approved", operator)
        .add_attribute("token_id", token_id.to_string()))
}

fn handle_approve_all(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    operator: String,
    expires: Option<Expiration>,
) -> Result<Response, ContractError> {
    let operator_addr = deps.api.addr_validate(&operator[..])?;
    let expires = match expires {
        Some(val) => val,
        None => Expiration::Never {},
    };

    let appr = OPERATORS.may_load(deps.storage, (&info.sender, &operator_addr))?;
    if let Some(val) = appr {
        if val == expires {
            return Err(ContractError::OperatorApproved { operator });
        }
    }

    if expires.is_expired(&env.block) {
        return Err(ContractError::Expired);
    }

    // Save the new/updated details
    OPERATORS.save(deps.storage, (&info.sender, &operator_addr), &expires)?;

    Ok(Response::new()
        .add_attribute("action", "approve_all")
        .add_attribute("from", info.sender)
        .add_attribute("approved", operator))
}

pub fn handle_revoke(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: String,
    token_id: u64,
) -> Result<Response, ContractError> {
    let mut token = query_tokens(deps.as_ref(), token_id)?;
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

pub fn handle_revoke_all(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operator: String,
) -> Result<Response, ContractError> {
    let operator_addr = deps.api.addr_validate(operator.as_str())?;

    if OPERATORS.has(deps.storage, (&info.sender, &operator_addr)) {
        OPERATORS.remove(deps.storage, (&info.sender, &operator_addr));
    } else {
        return Err(ContractError::ApprovalNotFound { operator });
    }

    Ok(Response::new()
        .add_attribute("action", "revoke_all")
        .add_attribute("from", info.sender)
        .add_attribute("revoked", operator))
}

pub fn handle_mint(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MintMsg,
) -> Result<Response, ContractError> {
    // Load current contract state
    let mut config = query_config(deps.as_ref())?;

    // sender and minter address should be same
    if info.sender != config.minter {
        return Err(ContractError::Unauthorized);
    }

    // price of the new NFT cannot be zero
    if msg.price.is_empty() {
        return Err(ContractError::CustomError {
            val: String::from("Token price cannot be empty"),
        });
    } else {
        // let res: Vec<&Coin> = msg
        //     .price
        //     .iter()
        //     .filter(|val| val.amount.is_zero())
        //     .collect();
        for val in msg.price.iter() {
            if val.amount.is_zero() {
                return Err(ContractError::CustomError {
                    val: String::from("Token price cannot be zero"),
                });
            }
        }
    }

    // Increase the current amount of tokens issued
    let num_tokens = config.num_tokens + 1;
    // Create a new token
    let token = TokenInfo {
        owner: deps.api.addr_validate(&msg.owner)?,
        approvals: None,
        token_uri: msg.token_uri,
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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, Addr, StdError};
    use cw721::Expiration;

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

        // Correct instantiation
        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let stored_state = query_config(deps.as_ref()).unwrap();
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
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();

        match res {
            ContractError::CustomError { .. } => {}
            e => panic!("{:?}", e),
        }

        let msg = init_msg(String::from(""), String::from(""));
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
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Successful token minting
        let mint_msg = mint_msg("creator".to_string());
        let res = handle_mint(deps.as_mut(), env.clone(), info.clone(), mint_msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(4, res.attributes.len());

        let stored_token = query_tokens(deps.as_ref(), 1).unwrap();
        assert_eq!(stored_token.owner, "creator");
        assert_eq!(stored_token.token_id, 1);
        assert_eq!(stored_token.base_price, coins(1000, DENOM.to_string()));
        assert_eq!(stored_token.approvals, None);
        assert_eq!(stored_token.token_uri, None);

        // Unsuccessful token minting
        // * owner name is empty
        let msg = MintMsg {
            owner: String::new(),
            token_uri: None,
            price: coins(1000, DENOM.to_string()),
        };

        let res = handle_mint(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        match res {
            ContractError::Std(StdError::GenericErr { .. }) => {}
            e => panic!("{:?}", e),
        };

        // * amount is empty i.e., price has been provided as 0Denom
        let msg = MintMsg {
            owner: String::from("owner"),
            token_uri: None,
            price: coins(0, DENOM.to_string()),
        };

        let res = handle_mint(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        match res {
            ContractError::CustomError { .. } => {}
            e => panic! {"{:?}", e},
        };

        // * price is empty
        let msg = MintMsg {
            owner: String::from("owner"),
            token_uri: None,
            price: vec![],
        };

        let res = handle_mint(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();
        match res {
            ContractError::CustomError { .. } => {}
            e => panic! {"{:?}", e},
        };
    }

    #[test]
    fn approve() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.height = 50u64;
        let info = mock_info("creator", &coins(0, &DENOM.to_string()));

        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let mint_msg = mint_msg("owner1".to_string());
        handle_mint(deps.as_mut(), env.clone(), info, mint_msg).unwrap();

        // Successful approval request
        let approve_msg = ExecuteMsg::Approve {
            operator: "operator".to_string(),
            token_id: 1u64,
            expires: None,
        };

        let info = mock_info("owner1", &coins(0, &DENOM.to_string()));
        let res = execute(deps.as_mut(), env.clone(), info.clone(), approve_msg).unwrap();
        assert_eq!(res.messages.len(), 0);
        assert_eq!(res.attributes.len(), 4);

        let token = query_tokens(deps.as_ref(), 1u64).unwrap();
        assert_eq!(
            token.approvals.as_ref().unwrap().operator,
            Addr::unchecked("operator")
        );
        assert_eq!(token.approvals.unwrap().expires, Expiration::Never {});

        // Unsuccessful approval request
        // * empty operator field
        let approve_msg = ExecuteMsg::Approve {
            operator: String::new(),
            token_id: 1u64,
            expires: None,
        };
        let info = mock_info("owner1", &coins(0, &DENOM.to_string()));
        let res = execute(deps.as_mut(), env.clone(), info.clone(), approve_msg).unwrap_err();
        match res {
            ContractError::Std(StdError::GenericErr { .. }) => {}
            e => panic!("{:?}", e),
        };

        // * Invalid token
        let approve_msg = ExecuteMsg::Approve {
            operator: "operator".to_string(),
            token_id: 2u64,
            expires: None,
        };
        let info = mock_info("owner1", &coins(0, &DENOM.to_string()));
        execute(deps.as_mut(), env.clone(), info.clone(), approve_msg).unwrap_err();

        // * expired approval
        let approve_msg = ExecuteMsg::Approve {
            operator: "operator".to_string(),
            token_id: 1u64,
            expires: Some(Expiration::AtHeight(45u64)),
        };
        let info = mock_info("owner1", &coins(0, &DENOM.to_string()));
        let res = execute(deps.as_mut(), env.clone(), info.clone(), approve_msg).unwrap_err();
        match res {
            ContractError::Expired {} => {}
            e => panic!("{:?}", e),
        };
    }

    #[test]
    fn approve_all() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        env.block.height = 25u64;
        let info = mock_info("creator", &coins(0, &DENOM.to_string()));
        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Successful approval for operator
        // * new operator
        let operator = String::from("operator");
        let expires = None;

        let res = handle_approve_all(deps.as_mut(), env.clone(), info.clone(), operator, expires)
            .unwrap();
        assert_eq!(res.messages.len(), 0);
        assert_eq!(res.attributes.len(), 3);

        let res = OPERATORS
            .load(&deps.storage, (&info.sender, &Addr::unchecked("operator")))
            .unwrap();
        assert_eq!(res, Expiration::Never {});

        // * update expiration
        let operator = String::from("operator");
        let expires = Some(Expiration::AtHeight(130_000));

        let res = handle_approve_all(deps.as_mut(), env.clone(), info.clone(), operator, expires)
            .unwrap();
        assert_eq!(res.messages.len(), 0);
        assert_eq!(res.attributes.len(), 3);

        // Unsuccessful approval for operator
        // * Operator exists
        let operator = String::from("operator");
        let expires = Some(Expiration::AtHeight(130_000));
        let res = handle_approve_all(deps.as_mut(), env.clone(), info.clone(), operator, expires)
            .unwrap_err();

        match res {
            ContractError::OperatorApproved { .. } => {}
            e => panic!("{:?}", e),
        };

        // * Expired height
        let operator = String::from("operator");
        let expires = Some(Expiration::AtHeight(24u64));

        let res = handle_approve_all(deps.as_mut(), env.clone(), info.clone(), operator, expires)
            .unwrap_err();
        match res {
            ContractError::Expired => {}
            e => panic!("{:?}", e),
        };

        // * Expired time
        let operator = String::from("operator");
        let expires = Some(Expiration::AtTime(env.block.time.minus_seconds(64u64)));

        let res = handle_approve_all(deps.as_mut(), env.clone(), info.clone(), operator, expires)
            .unwrap_err();
        match res {
            ContractError::Expired => {}
            e => panic!("{:?}", e),
        };
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

        let token = query_tokens(deps.as_ref(), 1u64).unwrap();
        assert_eq!(
            token.approvals.unwrap().operator,
            Addr::unchecked("operator")
        );

        // Successful approval revoke
        let revoke_msg = ExecuteMsg::Revoke {
            operator: "operator".to_string(),
            token_id: 1u64,
        };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), revoke_msg).unwrap();
        assert_eq!(res.messages.len(), 0);
        assert_eq!(res.attributes.len(), 4);

        let token = query_tokens(deps.as_ref(), 1u64).unwrap();
        assert_eq!(token.approvals, None);

        // Unsuccessful approval revoke
        // * Invalid token id
        let revoke_msg = ExecuteMsg::Revoke {
            operator: "operator".to_string(),
            token_id: 2u64,
        };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), revoke_msg).unwrap_err();
        match res {
            ContractError::Std(StdError::NotFound { .. }) => {}
            e => panic!("{:?}", e),
        };

        // * Approval being revoked for an address that isn't approved.
        let revoke_msg = ExecuteMsg::Revoke {
            operator: "unknown".to_string(),
            token_id: 1u64,
        };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), revoke_msg).unwrap_err();
        match res {
            ContractError::ApprovalNotFound { .. } => {}
            e => panic!("{:?}", e),
        };

        // * Unauthorised sender
        let info = mock_info("owner2", &coins(0, &DENOM.to_string()));
        let revoke_msg = ExecuteMsg::Revoke {
            operator: "operator".to_string(),
            token_id: 1u64,
        };

        let res = execute(deps.as_mut(), env.clone(), info.clone(), revoke_msg).unwrap_err();
        match res {
            ContractError::Unauthorized => {}
            e => panic!("{:?}", e),
        };
    }

    #[test]
    fn revoke_all() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("owner", &coins(0u128, &DENOM.to_string()));

        // Approve an address for all tokens
        let res = handle_approve_all(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            String::from("operator"),
            None,
        )
        .unwrap();
        assert_eq!(res.messages.len(), 0);
        assert_eq!(res.attributes.len(), 3);

        // Check that the new approval exists in correct format
        let key = (&Addr::unchecked("owner"), &Addr::unchecked("operator"));
        assert!(OPERATORS.has(&deps.storage, key));
        assert_eq!(
            OPERATORS.load(&deps.storage, key).unwrap(),
            Expiration::Never {}
        );

        // Revoke previously provided approval
        let res = handle_revoke_all(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            String::from("operator"),
        )
        .unwrap();
        assert_eq!(res.messages.len(), 0);
        assert_eq!(res.attributes.len(), 3);
        assert!(!OPERATORS.has(&deps.storage, key));

        // Unsuccessful revoke
        let res = handle_revoke_all(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            String::from("operator"),
        )
        .unwrap_err();
        match res {
            ContractError::ApprovalNotFound { .. } => {}
            e => panic!("{:?}", e),
        };
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
        let mut token = query_tokens(deps.as_ref(), 1u64).unwrap();
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
}
