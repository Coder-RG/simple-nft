#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

use crate::state::{State, CONFIG};
use crate::{
    msg::{ExecuteMsg, InstantiateMsg, QueryMsg},
    ContractError,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:simple-nft";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    if msg.name.is_empty() {
        return Err(ContractError::CustomError {
            val: String::from("length of `name` should be greater than 1"),
        });
    }

    if msg.symbol.is_empty() {
        return Err(ContractError::CustomError {
            val: String::from("lenght of `symbol` should be greater than 1"),
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
        ExecuteMsg::TransferNft { .. } => handle_transfer_nft(deps, env, info, msg),
        _ => panic!("not Implemented"),
        // ExecuteMsg::SendNft { .. } => handle_send_nft(deps, env, info, msg),
        // ExecuteMsg::Approve { .. } => handle_approve(deps, env, info, msg),
        // ExecuteMsg::ApproveAll { .. } => handle_approve_all(deps, env, info, msg),
        // ExecuteMsg::Revoke { .. } => handle_revoke(deps, env, info, msg),
        // ExecuteMsg::RevokeAll { .. } => handle_revoke_all(deps, env, info, msg),
        // ExecuteMsg::Mint(..) => handle_mint(deps, env, info, msg),
    }
}

pub fn handle_transfer_nft(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    Err(ContractError::CustomError {
        val: String::from("Not implemented"),
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    panic!("Not Implemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    const DENOM: &str = "ubit";

    fn init_msg(name: String, symbol: String) -> InstantiateMsg {
        InstantiateMsg { name, symbol }
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let env = mock_env();

        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let info = mock_info("creator", &coins(0, &DENOM.to_string()));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = init_msg(String::new(), "NFT".to_string());
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap_err();

        match res {
            ContractError::CustomError { val } => {}
            e => panic!(e),
        }

        let msg = init_msg(String::from("TestNFT"), String::new());
        let res = instantiate(deps.as_mut(), env, info, msg).unwrap_err();

        match res {
            ContractError::CustomError { val } => {}
            e => panic!(e),
        }
    }
}
