use cosmwasm_std::{entry_point, Env};
use cosmwasm_std::{to_binary, Binary, Deps, StdError, StdResult};

use crate::msg::{
    AskingPriceResponse, NftInfoResponse, NumTokensResponse, OwnerOfResponse, QueryMsg,
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

        // Returns not implemented msg for rest
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

fn query_nft_info(deps: Deps, _env: Env, token_id: u64) -> StdResult<Binary> {
    let token = query_tokens(deps, token_id);
    let res = NftInfoResponse {
        token_uri: token.token_uri.unwrap_or_else(|| "None".to_string()),
    };
    to_binary(&res)
}

pub fn query_config(deps: Deps) -> State {
    CONFIG
        .load(deps.storage)
        .expect("Unable to load internal state")
    // match res {
    //     Ok(state) => state,
    //     Err(e) => panic!("{:?}", e),
    // }
}

pub fn query_tokens(deps: Deps, token_id: u64) -> TokenInfo {
    TOKENS
        .load(deps.storage, token_id)
        .unwrap_or_else(|_| panic!("Unable to load token with token_id: {}", token_id))
    // match res {
    //     Ok(token) => token,
    //     Err(e) => panic!("{:?}", e),
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{execute, instantiate};
    use crate::msg::{ExecuteMsg, InstantiateMsg, MintMsg};
    use cosmwasm_std::coins;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, Coin, Uint128};

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
