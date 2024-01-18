use classic_terraport::{
    mock_querier::mock_dependencies,
    token::{ InstantiateMsg, InstantiateMarketingInfo },
};
use cosmwasm_std::{
    Addr,
    Uint128,
    testing::{ mock_info, mock_env, MOCK_CONTRACT_ADDR },
    from_binary,
    OverflowError,
    StdError,
    Binary,
};
use cw20::{ MinterResponse, BalanceResponse };

use crate::{ contract::{ instantiate, query, execute }, msg::QueryMsg, error::ContractError };

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let governance = Addr::unchecked("governance");
    let token = Addr::unchecked("token");

    let msg = InstantiateMsg {
        name: "Test".to_string(),
        symbol: "TEST".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: governance.clone().into_string(),
            cap: Some(Uint128::from(1000000000u128)),
        }),
        marketing: Some(InstantiateMarketingInfo {
            project: Some("mark".to_string()),
            description: Some("mark".to_string()),
            marketing: Some("mark".to_string()),
            logo: Some(cw20::Logo::Url("".to_string())),
        }),
    };

    let info = mock_info(governance.as_str(), &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut().into_empty(), mock_env(), info.clone(), msg).unwrap();

    let _res = execute(deps.as_mut().into_empty(), mock_env(), info, crate::msg::ExecuteMsg::Mint {
        recipient: governance.clone().into_string(),
        amount: Uint128::from(1000000u128),
    }).unwrap();

    let query_res = query(deps.as_ref().into_empty(), mock_env(), QueryMsg::Balance {
        address: governance.into_string(),
    }).unwrap();
    let balance_res: BalanceResponse = from_binary(&query_res).unwrap();

    assert_eq!(BalanceResponse { balance: Uint128::from(1000000u128) }, balance_res);
}

#[test]
fn mint_with_different_minter() {
    let mut deps = mock_dependencies(&[]);

    let governance = Addr::unchecked("governance");
    let token = Addr::unchecked("token");

    let msg = InstantiateMsg {
        name: "Test".to_string(),
        symbol: "TEST".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: governance.clone().into_string(),
            cap: Some(Uint128::from(1000000000u128)),
        }),
        marketing: Some(InstantiateMarketingInfo {
            project: Some("mark".to_string()),
            description: Some("mark".to_string()),
            marketing: Some("mark".to_string()),
            logo: Some(cw20::Logo::Url("".to_string())),
        }),
    };

    let info = mock_info("user", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut().into_empty(), mock_env(), info.clone(), msg).unwrap();

    let err = execute(deps.as_mut().into_empty(), mock_env(), info, crate::msg::ExecuteMsg::Mint {
        recipient: governance.clone().into_string(),
        amount: Uint128::from(1000000u128),
    }).unwrap_err();

    assert_eq!(err, ContractError::Unauthorized {});
}

#[test]
fn transfer_with_insufficient_balance() {
    let mut deps = mock_dependencies(&[]);

    let governance = Addr::unchecked("governance");
    let token = Addr::unchecked("token");

    let msg = InstantiateMsg {
        name: "Test".to_string(),
        symbol: "TEST".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: governance.clone().into_string(),
            cap: Some(Uint128::from(1000000000u128)),
        }),
        marketing: Some(InstantiateMarketingInfo {
            project: Some("mark".to_string()),
            description: Some("mark".to_string()),
            marketing: Some("mark".to_string()),
            logo: Some(cw20::Logo::Url("".to_string())),
        }),
    };

    let info = mock_info(governance.as_str(), &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut().into_empty(), mock_env(), info.clone(), msg).unwrap();

    let _res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info.clone(),
        crate::msg::ExecuteMsg::Mint {
            recipient: governance.clone().into_string(),
            amount: Uint128::from(1000000u128),
        }
    ).unwrap();

    let err = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info,
        crate::msg::ExecuteMsg::Transfer {
            recipient: "user".to_string(),
            amount: Uint128::from(10000000u128),
        }
    ).unwrap_err();

    let query_res = query(deps.as_ref().into_empty(), mock_env(), QueryMsg::Balance {
        address: governance.into_string(),
    }).unwrap();
    let balance_res: BalanceResponse = from_binary(&query_res).unwrap();

    assert_eq!(BalanceResponse { balance: Uint128::from(1000000u128) }, balance_res);

    assert_eq!(
        err,
        ContractError::Std(StdError::Overflow {
            source: OverflowError {
                operation: cosmwasm_std::OverflowOperation::Sub,
                operand1: "1000000".to_string(),
                operand2: "10000000".to_string(),
            },
        })
    )
}

#[test]
fn transfer() {
    let mut deps = mock_dependencies(&[]);

    let governance = Addr::unchecked("governance");
    let token = Addr::unchecked("token");

    let msg = InstantiateMsg {
        name: "Test".to_string(),
        symbol: "TEST".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: governance.clone().into_string(),
            cap: Some(Uint128::from(1000000000u128)),
        }),
        marketing: Some(InstantiateMarketingInfo {
            project: Some("mark".to_string()),
            description: Some("mark".to_string()),
            marketing: Some("mark".to_string()),
            logo: Some(cw20::Logo::Url("".to_string())),
        }),
    };

    let info = mock_info(governance.as_str(), &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut().into_empty(), mock_env(), info.clone(), msg).unwrap();

    let _res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info.clone(),
        crate::msg::ExecuteMsg::Mint {
            recipient: governance.clone().into_string(),
            amount: Uint128::from(1000000u128),
        }
    ).unwrap();

    let _res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info,
        crate::msg::ExecuteMsg::Transfer {
            recipient: "user".to_string(),
            amount: Uint128::from(1000000u128),
        }
    ).unwrap();

    let query_res = query(deps.as_ref().into_empty(), mock_env(), QueryMsg::Balance {
        address: governance.into_string(),
    }).unwrap();
    let balance_res: BalanceResponse = from_binary(&query_res).unwrap();

    let query_res = query(deps.as_ref().into_empty(), mock_env(), QueryMsg::Balance {
        address: "user".to_string(),
    }).unwrap();
    let balance_res2: BalanceResponse = from_binary(&query_res).unwrap();

    assert_eq!(BalanceResponse { balance: Uint128::zero() }, balance_res);
    assert_eq!(BalanceResponse { balance: Uint128::from(1000000u128) }, balance_res2);
}

#[test]
fn send_with_insufficient_balance() {
    let mut deps = mock_dependencies(&[]);

    let governance = Addr::unchecked("governance");
    let token = Addr::unchecked("token");

    let msg = InstantiateMsg {
        name: "Test".to_string(),
        symbol: "TEST".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: governance.clone().into_string(),
            cap: Some(Uint128::from(1000000000u128)),
        }),
        marketing: Some(InstantiateMarketingInfo {
            project: Some("mark".to_string()),
            description: Some("mark".to_string()),
            marketing: Some("mark".to_string()),
            logo: Some(cw20::Logo::Url("".to_string())),
        }),
    };

    let info = mock_info(governance.as_str(), &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut().into_empty(), mock_env(), info.clone(), msg).unwrap();

    let _res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info.clone(),
        crate::msg::ExecuteMsg::Mint {
            recipient: governance.clone().into_string(),
            amount: Uint128::from(1000000u128),
        }
    ).unwrap();

    let err = execute(deps.as_mut().into_empty(), mock_env(), info, crate::msg::ExecuteMsg::Send {
        amount: Uint128::from(10000000u128),
        contract: MOCK_CONTRACT_ADDR.to_string(),
        msg: Binary(vec![]),
    }).unwrap_err();

    let query_res = query(deps.as_ref().into_empty(), mock_env(), QueryMsg::Balance {
        address: governance.into_string(),
    }).unwrap();
    let balance_res: BalanceResponse = from_binary(&query_res).unwrap();

    assert_eq!(BalanceResponse { balance: Uint128::from(1000000u128) }, balance_res);

    assert_eq!(
        err,
        ContractError::Std(StdError::Overflow {
            source: OverflowError {
                operation: cosmwasm_std::OverflowOperation::Sub,
                operand1: "1000000".to_string(),
                operand2: "10000000".to_string(),
            },
        })
    )
}

#[test]
fn send() {
    let mut deps = mock_dependencies(&[]);

    let governance = Addr::unchecked("governance");
    let token = Addr::unchecked("token");

    let msg = InstantiateMsg {
        name: "Test".to_string(),
        symbol: "TEST".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: governance.clone().into_string(),
            cap: Some(Uint128::from(1000000000u128)),
        }),
        marketing: Some(InstantiateMarketingInfo {
            project: Some("mark".to_string()),
            description: Some("mark".to_string()),
            marketing: Some("mark".to_string()),
            logo: Some(cw20::Logo::Url("".to_string())),
        }),
    };

    let info = mock_info(governance.as_str(), &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut().into_empty(), mock_env(), info.clone(), msg).unwrap();

    let _res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info.clone(),
        crate::msg::ExecuteMsg::Mint {
            recipient: governance.clone().into_string(),
            amount: Uint128::from(1000000u128),
        }
    ).unwrap();

    let _res = execute(deps.as_mut().into_empty(), mock_env(), info, crate::msg::ExecuteMsg::Send {
        amount: Uint128::from(1000000u128),
        contract: MOCK_CONTRACT_ADDR.to_string(),
        msg: Binary(vec![]),
    }).unwrap();

    let query_res = query(deps.as_ref().into_empty(), mock_env(), QueryMsg::Balance {
        address: governance.into_string(),
    }).unwrap();
    let balance_res: BalanceResponse = from_binary(&query_res).unwrap();

    let query_res = query(deps.as_ref().into_empty(), mock_env(), QueryMsg::Balance {
        address: MOCK_CONTRACT_ADDR.to_string(),
    }).unwrap();
    let balance_res2: BalanceResponse = from_binary(&query_res).unwrap();

    assert_eq!(BalanceResponse { balance: Uint128::zero() }, balance_res);
    assert_eq!(BalanceResponse { balance: Uint128::from(1000000u128) }, balance_res2);
}

#[test]
fn burn() {
    let mut deps = mock_dependencies(&[]);

    let governance = Addr::unchecked("governance");
    let token = Addr::unchecked("token");

    let msg = InstantiateMsg {
        name: "Test".to_string(),
        symbol: "TEST".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: governance.clone().into_string(),
            cap: Some(Uint128::from(1000000000u128)),
        }),
        marketing: Some(InstantiateMarketingInfo {
            project: Some("mark".to_string()),
            description: Some("mark".to_string()),
            marketing: Some("mark".to_string()),
            logo: Some(cw20::Logo::Url("".to_string())),
        }),
    };

    let info = mock_info(governance.as_str(), &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut().into_empty(), mock_env(), info.clone(), msg).unwrap();

    let _res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info.clone(),
        crate::msg::ExecuteMsg::Mint {
            recipient: governance.clone().into_string(),
            amount: Uint128::from(1000000u128),
        }
    ).unwrap();

    let _res = execute(deps.as_mut().into_empty(), mock_env(), info, crate::msg::ExecuteMsg::Burn {
        amount: Uint128::from(1000000u128),
    }).unwrap();

    let query_res = query(deps.as_ref().into_empty(), mock_env(), QueryMsg::Balance {
        address: governance.into_string(),
    }).unwrap();
    let balance_res: BalanceResponse = from_binary(&query_res).unwrap();

    assert_eq!(BalanceResponse { balance: Uint128::zero() }, balance_res);
}

#[test]
fn update_marketing_with_wrong_sender() {
    let mut deps = mock_dependencies(&[]);

    let governance = Addr::unchecked("governance");
    let token = Addr::unchecked("token");

    let msg = InstantiateMsg {
        name: "Test".to_string(),
        symbol: "TEST".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: governance.clone().into_string(),
            cap: Some(Uint128::from(1000000000u128)),
        }),
        marketing: Some(InstantiateMarketingInfo {
            project: Some("mark".to_string()),
            description: Some("mark".to_string()),
            marketing: Some(governance.clone().into_string()),
            logo: Some(cw20::Logo::Url("".to_string())),
        }),
    };

    let info = mock_info(governance.as_str(), &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut().into_empty(), mock_env(), info.clone(), msg).unwrap();

    let _res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info.clone(),
        crate::msg::ExecuteMsg::Mint {
            recipient: governance.clone().into_string(),
            amount: Uint128::from(1000000u128),
        }
    ).unwrap();

    let info = mock_info("user2", &[]);

    let err = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info,
        crate::msg::ExecuteMsg::UpdateMarketing {
            project: Some("".to_string()),
            description: Some("".to_string()),
            marketing: Some("user".to_string()),
        }
    ).unwrap_err();

    assert_eq!(err, ContractError::Unauthorized {})
}

#[test]
fn update_marketing() {
    let mut deps = mock_dependencies(&[]);

    let governance = Addr::unchecked("governance");
    let token = Addr::unchecked("token");

    let msg = InstantiateMsg {
        name: "Test".to_string(),
        symbol: "TEST".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: governance.clone().into_string(),
            cap: Some(Uint128::from(1000000000u128)),
        }),
        marketing: Some(InstantiateMarketingInfo {
            project: Some("mark".to_string()),
            description: Some("mark".to_string()),
            marketing: Some(governance.clone().into_string()),
            logo: Some(cw20::Logo::Url("".to_string())),
        }),
    };

    let info = mock_info(governance.as_str(), &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut().into_empty(), mock_env(), info.clone(), msg).unwrap();

    let _res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info.clone(),
        crate::msg::ExecuteMsg::Mint {
            recipient: governance.clone().into_string(),
            amount: Uint128::from(1000000u128),
        }
    ).unwrap();

    let info = mock_info(governance.as_str(), &[]);

    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info,
        crate::msg::ExecuteMsg::UpdateMarketing {
            project: Some("".to_string()),
            description: Some("".to_string()),
            marketing: Some("user".to_string()),
        }
    ).unwrap();
}

#[test]
fn update_logo_with_wrong_sender() {
    let mut deps = mock_dependencies(&[]);

    let governance = Addr::unchecked("governance");
    let token = Addr::unchecked("token");

    let msg = InstantiateMsg {
        name: "Test".to_string(),
        symbol: "TEST".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: governance.clone().into_string(),
            cap: Some(Uint128::from(1000000000u128)),
        }),
        marketing: Some(InstantiateMarketingInfo {
            project: Some("mark".to_string()),
            description: Some("mark".to_string()),
            marketing: Some(governance.clone().into_string()),
            logo: Some(cw20::Logo::Url("".to_string())),
        }),
    };

    let info = mock_info(governance.as_str(), &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut().into_empty(), mock_env(), info.clone(), msg).unwrap();

    let _res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info.clone(),
        crate::msg::ExecuteMsg::Mint {
            recipient: governance.clone().into_string(),
            amount: Uint128::from(1000000u128),
        }
    ).unwrap();

    let info = mock_info("user2", &[]);

    let err = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info,
        crate::msg::ExecuteMsg::UploadLogo(cw20::Logo::Url("".to_string()))
    ).unwrap_err();

    assert_eq!(err, ContractError::Unauthorized {})
}

#[test]
fn update_logo() {
    let mut deps = mock_dependencies(&[]);

    let governance = Addr::unchecked("governance");
    let token = Addr::unchecked("token");

    let msg = InstantiateMsg {
        name: "Test".to_string(),
        symbol: "TEST".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: governance.clone().into_string(),
            cap: Some(Uint128::from(1000000000u128)),
        }),
        marketing: Some(InstantiateMarketingInfo {
            project: Some("mark".to_string()),
            description: Some("mark".to_string()),
            marketing: Some(governance.clone().into_string()),
            logo: Some(cw20::Logo::Url("".to_string())),
        }),
    };

    let info = mock_info(governance.as_str(), &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut().into_empty(), mock_env(), info.clone(), msg).unwrap();

    let _res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info.clone(),
        crate::msg::ExecuteMsg::Mint {
            recipient: governance.clone().into_string(),
            amount: Uint128::from(1000000u128),
        }
    ).unwrap();

    let info = mock_info(governance.as_str(), &[]);

    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info,
        crate::msg::ExecuteMsg::UploadLogo(cw20::Logo::Url("".to_string()))
    ).unwrap();
}

#[test]
fn update_minter_with_wrong_sender() {
    let mut deps = mock_dependencies(&[]);

    let governance = Addr::unchecked("governance");
    let token = Addr::unchecked("token");

    let msg = InstantiateMsg {
        name: "Test".to_string(),
        symbol: "TEST".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: governance.clone().into_string(),
            cap: Some(Uint128::from(1000000000u128)),
        }),
        marketing: Some(InstantiateMarketingInfo {
            project: Some("mark".to_string()),
            description: Some("mark".to_string()),
            marketing: Some(governance.clone().into_string()),
            logo: Some(cw20::Logo::Url("".to_string())),
        }),
    };

    let info = mock_info(governance.as_str(), &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut().into_empty(), mock_env(), info.clone(), msg).unwrap();

    let _res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info.clone(),
        crate::msg::ExecuteMsg::Mint {
            recipient: governance.clone().into_string(),
            amount: Uint128::from(1000000u128),
        }
    ).unwrap();

    let info = mock_info("user2", &[]);

    let err = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info,
        crate::msg::ExecuteMsg::UpdateMinter { new_minter: Some("user".to_string()) }
    ).unwrap_err();

    assert_eq!(err, ContractError::Unauthorized {})
}

#[test]
fn update_minter() {
    let mut deps = mock_dependencies(&[]);

    let governance = Addr::unchecked("governance");
    let token = Addr::unchecked("token");

    let msg = InstantiateMsg {
        name: "Test".to_string(),
        symbol: "TEST".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: Some(MinterResponse {
            minter: governance.clone().into_string(),
            cap: Some(Uint128::from(1000000000u128)),
        }),
        marketing: Some(InstantiateMarketingInfo {
            project: Some("mark".to_string()),
            description: Some("mark".to_string()),
            marketing: Some(governance.clone().into_string()),
            logo: Some(cw20::Logo::Url("".to_string())),
        }),
    };

    let info = mock_info(governance.as_str(), &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut().into_empty(), mock_env(), info.clone(), msg).unwrap();

    let _res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info.clone(),
        crate::msg::ExecuteMsg::Mint {
            recipient: governance.clone().into_string(),
            amount: Uint128::from(1000000u128),
        }
    ).unwrap();

    let info = mock_info(governance.as_str(), &[]);

    let res = execute(
        deps.as_mut().into_empty(),
        mock_env(),
        info,
        crate::msg::ExecuteMsg::UpdateMinter { new_minter: Some("user".to_string()) }
    ).unwrap();
}
