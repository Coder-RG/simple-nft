use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, Env, Order, StdError, StdResult};
use cw721::{Expiration, OperatorsResponse};
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;

use crate::msg::{
    AllNftInfoResponse, Approval, ApprovalResponse, ApprovalsResponse, AskingPriceResponse,
    ContractInfoResponse, NftInfoResponse, NumTokensResponse, OwnerOfResponse, QueryMsg,
};
use crate::state::{State, TokenInfo, CONFIG, OPERATORS, TOKENS};

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AskingPrice { token_id } => to_binary(&query_asking_price(deps, env, token_id)?),

        QueryMsg::OwnerOf {
            token_id,
            include_expired,
        } => to_binary(&query_owner_of(deps, env, token_id, include_expired)?),

        QueryMsg::Approval {
            token_id,
            operator,
            include_expired,
        } => to_binary(&query_approval(
            deps,
            env,
            token_id,
            operator,
            include_expired,
        )?),

        QueryMsg::Approvals {
            token_id,
            include_expired,
        } => to_binary(&query_approvals(deps, env, token_id, include_expired)?),

        QueryMsg::AllOperators {
            owner,
            include_expired,
            start_after,
            limit,
        } => to_binary(&query_approved_for_all(
            deps,
            env,
            owner,
            include_expired,
            start_after,
            limit,
        )?),

        QueryMsg::NumTokens {} => to_binary(&query_num_tokens(deps, env)?),

        QueryMsg::ContractInfo {} => to_binary(&query_contract_info(deps, env)?),

        QueryMsg::NftInfo { token_id } => to_binary(&query_nft_info(deps, env, token_id)?),

        QueryMsg::AllNftInfo {
            token_id,
            include_expired,
        } => to_binary(&query_all_nft_info(deps, env, token_id, include_expired)?),
    }
}

pub fn query_asking_price(deps: Deps, _env: Env, token_id: u64) -> StdResult<AskingPriceResponse> {
    let token_info = query_tokens(deps, token_id)?;
    Ok(AskingPriceResponse {
        price: token_info.base_price,
    })
}

fn query_owner_of(
    deps: Deps,
    env: Env,
    token_id: u64,
    include_expired: Option<bool>,
) -> StdResult<OwnerOfResponse> {
    let token = query_tokens(deps, token_id)?;
    let include_expired = include_expired.unwrap_or(false);

    let approvals = token
        .approvals
        .into_iter()
        .filter(|appr| include_expired || !appr.expires.is_expired(&env.block))
        .collect();

    Ok(OwnerOfResponse {
        owner: token.owner.into_string(),
        approvals,
    })
}

fn query_approval(
    deps: Deps,
    env: Env,
    token_id: u64,
    operator: String,
    include_expired: Option<bool>,
) -> StdResult<ApprovalResponse> {
    let token = query_tokens(deps, token_id)?;
    let operator_addr = deps.api.addr_validate(operator.as_str())?;
    let include_expired = include_expired.unwrap_or(false);

    let appr: Vec<Approval> = token
        .approvals
        .into_iter()
        .filter(|val| val.operator == operator_addr)
        .collect();

    if !appr.is_empty() && (include_expired || !appr[0].expires.is_expired(&env.block)) {
        let res = ApprovalResponse {
            approval: Approval {
                operator: operator_addr,
                expires: appr[0].expires,
            },
        };

        return Ok(res);
    };
    Err(StdError::NotFound {
        kind: String::from("Approval not found for given address"),
    })
}

fn query_approvals(
    deps: Deps,
    env: Env,
    token_id: u64,
    include_expired: Option<bool>,
) -> StdResult<ApprovalsResponse> {
    let include_expired = include_expired.unwrap_or(false);
    let token = query_tokens(deps, token_id)?;

    let res = token
        .approvals
        .into_iter()
        .filter(|appr| include_expired || !appr.expires.is_expired(&env.block))
        .collect();

    Ok(ApprovalsResponse { approvals: res })
}

fn query_approved_for_all(
    deps: Deps,
    env: Env,
    owner: String,
    include_expired: Option<bool>,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<OperatorsResponse> {
    let include_expired = include_expired.unwrap_or(false);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start_addr = maybe_addr(deps.api, start_after)?;
    let start = start_addr.as_ref().map(Bound::exclusive);
    let owner_addr = deps.api.addr_validate(&owner)?;

    let res: StdResult<Vec<_>> = OPERATORS
        .prefix(&owner_addr)
        .range(deps.storage, start, None, Order::Ascending)
        .filter(|r| include_expired || r.is_err() || !r.as_ref().unwrap().1.is_expired(&env.block))
        .take(limit)
        .map(parse_approval)
        .collect();
    Ok(OperatorsResponse { operators: res? })
}

fn parse_approval(item: StdResult<(Addr, Expiration)>) -> StdResult<cw721::Approval> {
    item.map(|(spender, expires)| cw721::Approval {
        spender: spender.to_string(),
        expires,
    })
}

fn query_num_tokens(deps: Deps, _env: Env) -> StdResult<NumTokensResponse> {
    let config = query_config(deps)?;
    Ok(NumTokensResponse {
        tokens: config.num_tokens,
    })
}

fn query_contract_info(deps: Deps, _env: Env) -> StdResult<ContractInfoResponse> {
    let config = query_config(deps)?;
    Ok(ContractInfoResponse {
        name: config.name,
        symbol: config.symbol,
    })
}

fn query_nft_info(deps: Deps, _env: Env, token_id: u64) -> StdResult<NftInfoResponse> {
    let token = query_tokens(deps, token_id)?;
    let res = NftInfoResponse {
        token_uri: token.token_uri.unwrap_or_else(|| "None".to_string()),
    };
    Ok(res)
}

fn query_all_nft_info(
    deps: Deps,
    env: Env,
    token_id: u64,
    include_expired: Option<bool>,
) -> StdResult<AllNftInfoResponse> {
    let owner = query_owner_of(deps, env.clone(), token_id, include_expired)?;
    let nft = query_nft_info(deps, env, token_id)?;

    let res = AllNftInfoResponse { owner, info: nft };
    Ok(res)
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
        assert_eq!(res.owner, "creator");
        assert_eq!(res.approvals, vec![]);

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
        assert_eq!(res.tokens, 0);

        let mint_msg = mint_msg("creator".to_string());
        let msg = ExecuteMsg::Mint(mint_msg);
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Query # of tokens after minting
        let res = query_num_tokens(deps.as_ref(), env.clone()).unwrap();
        assert_eq!(res.tokens, 1);
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
                approvals: vec![],
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

    #[test]
    fn approval() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("minter", &coins(0u128, &DENOM.to_string()));
        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        // Mint a new token
        let mint_msg = mint_msg("creator".to_string());
        let msg = ExecuteMsg::Mint(mint_msg);
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Approve operator
        let msg = ExecuteMsg::Approve {
            operator: "operator".to_string(),
            token_id: 1,
            expires: None,
        };
        let info = mock_info("creator", &coins(0u128, &DENOM.to_string()));
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Successful query
        let res = query_approval(
            deps.as_ref(),
            env.clone(),
            1,
            String::from("operator"),
            Some(false),
        )
        .unwrap();
        assert_eq!(
            res,
            ApprovalResponse {
                approval: Approval {
                    operator: Addr::unchecked("operator"),
                    expires: Expiration::Never {}
                }
            }
        );

        // Unsuccessful query
        // * operator not approved
        let res = query_approval(
            deps.as_ref(),
            env.clone(),
            1u64,
            String::from("unknown"),
            None,
        )
        .unwrap_err();
        match res {
            StdError::NotFound { .. } => {}
            e => panic!("{:?}", e),
        };
    }

    #[test]
    fn approvals() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("minter", &coins(0u128, &DENOM.to_string()));
        let msg = init_msg("TestNFT".to_string(), "NFT".to_string());
        let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        // Mint a new token
        let mint_msg = mint_msg("creator".to_string());
        let msg = ExecuteMsg::Mint(mint_msg);
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        // Approve operator
        let msg = ExecuteMsg::Approve {
            operator: "operator".to_string(),
            token_id: 1,
            expires: None,
        };
        let info = mock_info("creator", &coins(0u128, &DENOM.to_string()));
        execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

        let res = query_approvals(deps.as_ref(), env.clone(), 1u64, None).unwrap();

        assert_eq!(
            res,
            ApprovalsResponse {
                approvals: vec![Approval {
                    operator: Addr::unchecked("operator"),
                    expires: Expiration::Never {}
                }]
            }
        )
    }
}
