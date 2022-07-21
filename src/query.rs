use cosmwasm_std::{entry_point, from_binary, to_binary, Binary, Deps, Env, StdError, StdResult};

use crate::msg::{
    AllNftInfoResponse, AskingPriceResponse, ContractInfoResponse, NftInfoResponse,
    NumTokensResponse, OwnerOfResponse, QueryMsg,
};
use crate::state::{State, TokenInfo, CONFIG, TOKENS};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AskingPrice { token_id } => query_asking_price(deps, env, token_id),

        QueryMsg::OwnerOf {
            token_id,
            include_expired,
        } => query_owner_of(deps, env, token_id, include_expired),

        QueryMsg::NumTokens {} => query_num_tokens(deps, env),

        QueryMsg::NftInfo { token_id } => query_nft_info(deps, env, token_id),

        QueryMsg::AllNftInfo {
            token_id,
            include_expired,
        } => query_all_nft_info(deps, env, token_id, include_expired),

        QueryMsg::ContractInfo {} => query_contract_info(deps, env),

        // Returns not implemented msg for rest
        _ => Err(StdError::NotFound {
            kind: String::from("Not Implemented"),
        }),
    }
}

pub fn query_asking_price(deps: Deps, _env: Env, token_id: u64) -> StdResult<Binary> {
    let token_info = query_tokens(deps, token_id)?;
    let response = AskingPriceResponse {
        price: token_info.base_price,
    };
    to_binary(&response)
}

fn query_owner_of(
    deps: Deps,
    env: Env,
    token_id: u64,
    include_expired: Option<bool>,
) -> StdResult<Binary> {
    let token = query_tokens(deps, token_id)?;

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
    let config = query_config(deps)?;
    to_binary(&NumTokensResponse {
        tokens: config.num_tokens,
    })
}

fn query_contract_info(deps: Deps, _env: Env) -> StdResult<Binary> {
    let config = query_config(deps)?;
    to_binary(&ContractInfoResponse {
        name: config.name,
        symbol: config.symbol,
    })
}

fn query_nft_info(deps: Deps, _env: Env, token_id: u64) -> StdResult<Binary> {
    let token = query_tokens(deps, token_id)?;
    let res = NftInfoResponse {
        token_uri: token.token_uri.unwrap_or_else(|| "None".to_string()),
    };
    to_binary(&res)
}

fn query_all_nft_info(
    deps: Deps,
    env: Env,
    token_id: u64,
    include_expired: Option<bool>,
) -> StdResult<Binary> {
    let owner: OwnerOfResponse = from_binary(&query_owner_of(
        deps,
        env.clone(),
        token_id,
        include_expired,
    )?)?;
    let nft: NftInfoResponse = from_binary(&query_nft_info(deps, env, token_id)?)?;

    let res = AllNftInfoResponse { owner, info: nft };
    to_binary(&res)
}

pub fn query_config(deps: Deps) -> StdResult<State> {
    let res = CONFIG.may_load(deps.storage)?;
    match res {
        Some(val) => Ok(val),
        None => Err(StdError::GenericErr {
            msg: String::from("Unable to load internal state"),
        }),
    }
}

pub fn query_tokens(deps: Deps, token_id: u64) -> StdResult<TokenInfo> {
    let res = TOKENS.may_load(deps.storage, token_id)?;
    match res {
        Some(val) => Ok(val),
        None => Err(StdError::NotFound {
            kind: format!("Unable to load token with token_id: {}", token_id),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{execute, instantiate};
    use crate::msg::{ExecuteMsg, InstantiateMsg, MintMsg};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Coin, Uint128};

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
    fn asking_price() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("minter", &coins(0u128, &DENOM.to_string()));
        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        let mint_msg = mint_msg("creator".to_string());
        let msg = ExecuteMsg::Mint(mint_msg);
        let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(0, res.messages.len());
        assert_eq!(4, res.attributes.len());

        // Successful query
        let query_msg = QueryMsg::AskingPrice { token_id: 1 };
        let res = query(deps.as_ref(), env.clone(), query_msg).unwrap();
        let res: AskingPriceResponse = from_binary(&res).unwrap();
        assert_eq!(
            res.price,
            vec![Coin {
                amount: Uint128::from(1000u64),
                denom: DENOM.to_string()
            }]
        );

        // Unsuccessful query
        let query_msg = QueryMsg::AskingPrice { token_id: 2 };
        let res = query(deps.as_ref(), env, query_msg).unwrap_err();
        match res {
            StdError::NotFound { .. } => {}
            e => panic!("{:?}", e),
        };
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

        // Successful response
        let res = query_owner_of(deps.as_ref(), env.clone(), 1u64, None).unwrap();
        let res: OwnerOfResponse = from_binary(&res).unwrap();
        assert_eq!(res.owner, "creator");
        assert_eq!(res.approvals, None);

        // Unsuccessful response
        let res = query_owner_of(deps.as_ref(), env.clone(), 2u64, Some(true)).unwrap_err();
        match res {
            StdError::NotFound { .. } => {}
            e => panic!("{:?}", e),
        };
    }

    #[test]
    fn num_tokens() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("minter", &coins(0u128, &DENOM.to_string()));
        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        // Query # of tokens after initialization
        let res = query_num_tokens(deps.as_ref(), env.clone()).unwrap();
        let result: NumTokensResponse = from_binary(&res).unwrap();
        assert_eq!(result.tokens, 0);

        let mint_msg = mint_msg("creator".to_string());
        let msg = ExecuteMsg::Mint(mint_msg);
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Query # of tokens after minting
        let res = query_num_tokens(deps.as_ref(), env.clone()).unwrap();
        let result: NumTokensResponse = from_binary(&res).unwrap();
        assert_eq!(result.tokens, 1);
    }

    #[test]
    fn nft_info() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("minter", &coins(0u128, &DENOM.to_string()));
        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        let mint_msg = mint_msg("creator".to_string());
        let msg = ExecuteMsg::Mint(mint_msg);
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Successful query
        let msg = QueryMsg::NftInfo { token_id: 1u64 };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let result: NftInfoResponse = from_binary(&res).unwrap();
        assert_eq!(result.token_uri, String::from("None"));

        // Unsuccessful query
        let msg = QueryMsg::NftInfo { token_id: 2u64 };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap_err();
        match res {
            StdError::NotFound { .. } => {}
            e => panic!("{:?}", e),
        };
    }

    #[test]
    fn all_nft_info() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("minter", &coins(0u128, &DENOM.to_string()));
        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        let msg = ExecuteMsg::Mint(mint_msg(String::from("owner")));
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Successful query
        let msg = QueryMsg::AllNftInfo {
            token_id: 1u64,
            include_expired: Some(true),
        };
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let result: AllNftInfoResponse = from_binary(&res).unwrap();
        assert_eq!(
            result.owner,
            OwnerOfResponse {
                owner: String::from("owner"),
                approvals: None,
            }
        );
        assert_eq!(
            result.info,
            NftInfoResponse {
                token_uri: String::from("None")
            }
        );
    }

    #[test]
    fn contract_info() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("minter", &coins(0u128, &DENOM.to_string()));
        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        let msg = QueryMsg::ContractInfo {};
        let res = query(deps.as_ref(), env.clone(), msg).unwrap();
        let result: ContractInfoResponse = from_binary(&res).unwrap();
        assert_eq!(result.name, String::from("TestNFT"));
        assert_eq!(result.symbol, String::from("NFT"));
    }
}
