#[cfg(not(feature = "library"))]
use cosmwasm_std::{ to_binary, entry_point };
use cosmwasm_std::{
    Storage,
    Order,
    Uint128,
    Binary,
    Deps,
    DepsMut,
    Env,
    MessageInfo,
    Response,
    StdError,
    StdResult,
};
use cw_storage_plus::Bound;

use cw2::set_contract_version;
use cw20::{
    BalanceResponse,
    Cw20Coin,
    Cw20ReceiveMsg,
    DownloadLogoResponse,
    EmbeddedLogo,
    Logo,
    LogoInfo,
    MarketingInfoResponse,
    MinterResponse,
    TokenInfoResponse,
};
use crate::error::ContractError;
use crate::msg::{ ExecuteMsg, QueryMsg };

use crate::allowances::{
    execute_burn_from,
    execute_decrease_allowance,
    execute_increase_allowance,
    execute_send_from,
    execute_transfer_from,
    query_allowance,
};

use crate::enumerable::{ query_all_accounts, query_owner_allowances, query_spender_allowances };

use crate::state::{
    BALANCES,
    MARKETING_INFO,
    LOGO,
    TOKEN_INFO,
    TOTAL_SUPPLY_HISTORY,
    MinterData,
    TokenInfo,
};

use classic_terraport::token::InstantiateMsg;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:terraport-token";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const LOGO_SIZE_CAP: usize = 5 * 1024;

fn verify_xml_preamble(data: &[u8]) -> Result<(), ContractError> {
    // The easiest way to perform this check would be just match on regex, however regex
    // compilation is heavy and probably not worth it.

    let preamble = data
        .split_inclusive(|c| *c == b'>')
        .next()
        .ok_or(ContractError::InvalidXmlPreamble {})?;

    const PREFIX: &[u8] = b"<?xml ";
    const POSTFIX: &[u8] = b"?>";

    if !(preamble.starts_with(PREFIX) && preamble.ends_with(POSTFIX)) {
        Err(ContractError::InvalidXmlPreamble {})
    } else {
        Ok(())
    }

    // Additionally attributes format could be validated as they are well defined, as well as
    // comments presence inside of preable, but it is probably not worth it.
}

fn verify_xml_logo(logo: &[u8]) -> Result<(), ContractError> {
    verify_xml_preamble(logo)?;

    if logo.len() > LOGO_SIZE_CAP {
        Err(ContractError::LogoTooBig {})
    } else {
        Ok(())
    }
}

fn verify_png_logo(logo: &[u8]) -> Result<(), ContractError> {
    // PNG header format:
    // 0x89 - magic byte, out of ASCII table to fail on 7-bit systems
    // "PNG" ascii representation
    // [0x0d, 0x0a] - dos style line ending
    // 0x1a - dos control character, stop displaying rest of the file
    // 0x0a - unix style line ending
    const HEADER: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
    if logo.len() > LOGO_SIZE_CAP {
        Err(ContractError::LogoTooBig {})
    } else if !logo.starts_with(&HEADER) {
        Err(ContractError::InvalidPngHeader {})
    } else {
        Ok(())
    }
}

fn verify_logo(logo: &Logo) -> Result<(), ContractError> {
    match logo {
        Logo::Embedded(EmbeddedLogo::Svg(logo)) => verify_xml_logo(logo),
        Logo::Embedded(EmbeddedLogo::Png(logo)) => verify_png_logo(logo),
        Logo::Url(_) => Ok(()), // Any reasonable url validation would be regex based, probably not worth it
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // check valid token info
    msg.validate()?;

    // create initial accounts
    let total_supply = create_accounts(&mut deps, &msg.initial_balances, env.block.height)?;

    if !total_supply.is_zero() {
        capture_total_supply_history(deps.storage, &env, total_supply)?;
    }

    if let Some(limit) = msg.get_cap() {
        if total_supply > limit {
            return Err(
                ContractError::Std(StdError::generic_err("Initial supply greater than cap"))
            );
        }
    }

    let mint = match msg.mint {
        Some(m) =>
            Some(MinterData {
                minter: deps.api.addr_validate(&m.minter)?,
                cap: m.cap,
            }),
        None => None,
    };

    // store token info
    let data = TokenInfo {
        name: msg.name,
        symbol: msg.symbol,
        decimals: msg.decimals,
        total_supply,
        mint,
    };

    TOKEN_INFO.save(deps.storage, &data)?;
    if let Some(marketing) = msg.marketing {
        let logo = if let Some(logo) = marketing.logo {
            verify_logo(&logo)?;
            LOGO.save(deps.storage, &logo)?;

            match logo {
                Logo::Url(url) => Some(LogoInfo::Url(url)),
                Logo::Embedded(_) => Some(LogoInfo::Embedded),
            }
        } else {
            None
        };

        let data = MarketingInfoResponse {
            project: marketing.project,
            description: marketing.description,
            marketing: marketing.marketing.map(|addr| deps.api.addr_validate(&addr)).transpose()?,
            logo,
        };
        MARKETING_INFO.save(deps.storage, &data)?;
    }
    Ok(Response::default())
}

pub fn create_accounts(
    deps: &mut DepsMut,
    accounts: &[Cw20Coin],
    height: u64
) -> Result<Uint128, ContractError> {
    validate_accounts(accounts)?;

    let mut total_supply = Uint128::zero();
    for row in accounts {
        let address = deps.api.addr_validate(&row.address)?;
        BALANCES.save(deps.storage, &address, &row.amount, height)?;
        total_supply += row.amount;
    }

    Ok(total_supply)
}

pub fn validate_accounts(accounts: &[Cw20Coin]) -> Result<(), ContractError> {
    let mut addresses = accounts
        .iter()
        .map(|c| &c.address)
        .collect::<Vec<_>>();
    addresses.sort();
    addresses.dedup();

    if addresses.len() != accounts.len() {
        Err(ContractError::DuplicateInitialBalanceAddresses {})
    } else {
        Ok(())
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Transfer { recipient, amount } => {
            execute_transfer(deps, env, info, recipient, amount)
        }
        ExecuteMsg::Burn { amount } => execute_burn(deps, env, info, amount),
        ExecuteMsg::Send { contract, amount, msg } =>
            execute_send(deps, env, info, contract, amount, msg),
        ExecuteMsg::Mint { recipient, amount } => execute_mint(deps, env, info, recipient, amount),
        ExecuteMsg::IncreaseAllowance { spender, amount, expires } =>
            execute_increase_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::DecreaseAllowance { spender, amount, expires } =>
            execute_decrease_allowance(deps, env, info, spender, amount, expires),
        ExecuteMsg::TransferFrom { owner, recipient, amount } =>
            execute_transfer_from(deps, env, info, owner, recipient, amount),
        ExecuteMsg::BurnFrom { owner, amount } => execute_burn_from(deps, env, info, owner, amount),
        ExecuteMsg::SendFrom { owner, contract, amount, msg } =>
            execute_send_from(deps, env, info, owner, contract, amount, msg),
        ExecuteMsg::UpdateMarketing { project, description, marketing } =>
            execute_update_marketing(deps, env, info, project, description, marketing),
        ExecuteMsg::UploadLogo(logo) => execute_upload_logo(deps, env, info, logo),
        ExecuteMsg::UpdateMinter { new_minter } => {
            execute_update_minter(deps, env, info, new_minter)
        }
    }
}

pub fn execute_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128
) -> Result<Response, ContractError> {
    let rcpt_addr = deps.api.addr_validate(&recipient)?;

    BALANCES.update(
        deps.storage,
        &info.sender,
        env.block.height,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        }
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        env.block.height,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) }
    )?;

    let res = Response::new()
        .add_attribute("action", "transfer")
        .add_attribute("from", info.sender)
        .add_attribute("to", recipient)
        .add_attribute("amount", amount);
    Ok(res)
}

pub fn execute_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128
) -> Result<Response, ContractError> {
    // lower balance
    BALANCES.update(
        deps.storage,
        &info.sender,
        env.block.height,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        }
    )?;
    // reduce total_supply
    let token_info = TOKEN_INFO.update(
        deps.storage,
        |mut info| -> StdResult<_> {
            info.total_supply = info.total_supply.checked_sub(amount)?;
            Ok(info)
        }
    )?;

    capture_total_supply_history(deps.storage, &env, token_info.total_supply)?;

    let res = Response::new()
        .add_attribute("action", "burn")
        .add_attribute("from", info.sender)
        .add_attribute("amount", amount);
    Ok(res)
}

pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: String,
    amount: Uint128
) -> Result<Response, ContractError> {
    let mut config = TOKEN_INFO.may_load(deps.storage)?.ok_or(ContractError::Unauthorized {})?;

    if config.mint.as_ref().ok_or(ContractError::Unauthorized {})?.minter != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    // update supply and enforce cap
    config.total_supply += amount;
    if let Some(limit) = config.get_cap() {
        if config.total_supply > limit {
            return Err(ContractError::CannotExceedCap {});
        }
    }
    TOKEN_INFO.save(deps.storage, &config)?;

    capture_total_supply_history(deps.storage, &env, config.total_supply)?;

    // add amount to recipient balance
    let rcpt_addr = deps.api.addr_validate(&recipient)?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        env.block.height,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) }
    )?;

    let res = Response::new()
        .add_attribute("action", "mint")
        .add_attribute("to", recipient)
        .add_attribute("amount", amount);
    Ok(res)
}

pub fn execute_send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contract: String,
    amount: Uint128,
    msg: Binary
) -> Result<Response, ContractError> {
    let rcpt_addr = deps.api.addr_validate(&contract)?;

    // move the tokens to the contract
    BALANCES.update(
        deps.storage,
        &info.sender,
        env.block.height,
        |balance: Option<Uint128>| -> StdResult<_> {
            Ok(balance.unwrap_or_default().checked_sub(amount)?)
        }
    )?;
    BALANCES.update(
        deps.storage,
        &rcpt_addr,
        env.block.height,
        |balance: Option<Uint128>| -> StdResult<_> { Ok(balance.unwrap_or_default() + amount) }
    )?;

    let res = Response::new()
        .add_attribute("action", "send")
        .add_attribute("from", &info.sender)
        .add_attribute("to", &contract)
        .add_attribute("amount", amount)
        .add_message(
            (Cw20ReceiveMsg {
                sender: info.sender.into(),
                amount,
                msg,
            }).into_cosmos_msg(contract)?
        );
    Ok(res)
}

pub fn execute_update_minter(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_minter: Option<String>
) -> Result<Response, ContractError> {
    let mut config = TOKEN_INFO.may_load(deps.storage)?.ok_or(ContractError::Unauthorized {})?;

    let mint = config.mint.as_ref().ok_or(ContractError::Unauthorized {})?;
    if mint.minter != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let minter_data = new_minter
        .map(|new_minter| deps.api.addr_validate(&new_minter))
        .transpose()?
        .map(|minter| MinterData {
            minter,
            cap: mint.cap,
        });

    config.mint = minter_data;

    TOKEN_INFO.save(deps.storage, &config)?;

    Ok(
        Response::default()
            .add_attribute("action", "update_minter")
            .add_attribute(
                "new_minter",
                config.mint.map(|m| m.minter.into_string()).unwrap_or_else(|| "None".to_string())
            )
    )
}

pub fn execute_update_marketing(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    project: Option<String>,
    description: Option<String>,
    marketing: Option<String>
) -> Result<Response, ContractError> {
    let mut marketing_info = MARKETING_INFO.may_load(deps.storage)?.ok_or(
        ContractError::Unauthorized {}
    )?;

    if marketing_info.marketing.as_ref().ok_or(ContractError::Unauthorized {})? != &info.sender {
        return Err(ContractError::Unauthorized {});
    }

    match project {
        Some(empty) if empty.trim().is_empty() => {
            marketing_info.project = None;
        }
        Some(project) => {
            marketing_info.project = Some(project);
        }
        None => (),
    }

    match description {
        Some(empty) if empty.trim().is_empty() => {
            marketing_info.description = None;
        }
        Some(description) => {
            marketing_info.description = Some(description);
        }
        None => (),
    }

    match marketing {
        Some(empty) if empty.trim().is_empty() => {
            marketing_info.marketing = None;
        }
        Some(marketing) => {
            marketing_info.marketing = Some(deps.api.addr_validate(&marketing)?);
        }
        None => (),
    }

    if
        marketing_info.project.is_none() &&
        marketing_info.description.is_none() &&
        marketing_info.marketing.is_none() &&
        marketing_info.logo.is_none()
    {
        MARKETING_INFO.remove(deps.storage);
    } else {
        MARKETING_INFO.save(deps.storage, &marketing_info)?;
    }

    let res = Response::new().add_attribute("action", "update_marketing");
    Ok(res)
}

pub fn execute_upload_logo(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    logo: Logo
) -> Result<Response, ContractError> {
    let mut marketing_info = MARKETING_INFO.may_load(deps.storage)?.ok_or(
        ContractError::Unauthorized {}
    )?;

    verify_logo(&logo)?;

    if marketing_info.marketing.as_ref().ok_or(ContractError::Unauthorized {})? != &info.sender {
        return Err(ContractError::Unauthorized {});
    }

    LOGO.save(deps.storage, &logo)?;

    let logo_info = match logo {
        Logo::Url(url) => LogoInfo::Url(url),
        Logo::Embedded(_) => LogoInfo::Embedded,
    };

    marketing_info.logo = Some(logo_info);
    MARKETING_INFO.save(deps.storage, &marketing_info)?;

    let res = Response::new().add_attribute("action", "upload_logo");
    Ok(res)
}

/// Snapshots the total token supply at current block.
///
/// * **total_supply** current token total supply.
pub fn capture_total_supply_history(
    storage: &mut dyn Storage,
    env: &Env,
    total_supply: Uint128
) -> StdResult<()> {
    TOTAL_SUPPLY_HISTORY.save(storage, env.block.height, &total_supply)
}

/// Returns the total token supply at the given block.
pub fn get_total_supply_at(storage: &dyn Storage, block: u64) -> StdResult<Uint128> {
    // Look for the last value recorded before the current block (if none then value is zero)
    let end = Bound::inclusive(block);
    let last_value_up_to_block = TOTAL_SUPPLY_HISTORY.range(
        storage,
        None,
        Some(end),
        Order::Descending
    ).next();

    if let Some(value) = last_value_up_to_block {
        let (_, v) = value?;
        return Ok(v);
    }

    Ok(Uint128::zero())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Balance { address } => to_binary(&query_balance(deps, address)?),
        QueryMsg::BalanceAt { address, height } =>
            to_binary(&query_balance_at(deps, address, height)?),
        QueryMsg::TokenInfo {} => to_binary(&query_token_info(deps)?),
        QueryMsg::TotalSupplyAt { block } => to_binary(&get_total_supply_at(deps.storage, block)?),
        QueryMsg::Minter {} => to_binary(&query_minter(deps)?),
        QueryMsg::Allowance { owner, spender } => {
            to_binary(&query_allowance(deps, owner, spender)?)
        }
        QueryMsg::AllAllowances { owner, start_after, limit } =>
            to_binary(&query_owner_allowances(deps, owner, start_after, limit)?),
        QueryMsg::AllSpenderAllowances { spender, start_after, limit } =>
            to_binary(&query_spender_allowances(deps, spender, start_after, limit)?),
        QueryMsg::AllAccounts { start_after, limit } => {
            to_binary(&query_all_accounts(deps, start_after, limit)?)
        }
        QueryMsg::MarketingInfo {} => to_binary(&query_marketing_info(deps)?),
        QueryMsg::DownloadLogo {} => to_binary(&query_download_logo(deps)?),
    }
}

pub fn query_balance(deps: Deps, address: String) -> StdResult<BalanceResponse> {
    let address = deps.api.addr_validate(&address)?;
    let balance = BALANCES.may_load(deps.storage, &address)?.unwrap_or_default();
    Ok(BalanceResponse { balance })
}

pub fn query_balance_at(deps: Deps, address: String, block: u64) -> StdResult<BalanceResponse> {
    let address = deps.api.addr_validate(&address)?;
    let balance = BALANCES.may_load_at_height(deps.storage, &address, block)?.unwrap_or_default();
    Ok(BalanceResponse { balance })
}

pub fn query_token_info(deps: Deps) -> StdResult<TokenInfoResponse> {
    let info = TOKEN_INFO.load(deps.storage)?;
    let res = TokenInfoResponse {
        name: info.name,
        symbol: info.symbol,
        decimals: info.decimals,
        total_supply: info.total_supply,
    };
    Ok(res)
}

pub fn query_minter(deps: Deps) -> StdResult<Option<MinterResponse>> {
    let meta = TOKEN_INFO.load(deps.storage)?;
    let minter = match meta.mint {
        Some(m) =>
            Some(MinterResponse {
                minter: m.minter.into(),
                cap: m.cap,
            }),
        None => None,
    };
    Ok(minter)
}

pub fn query_marketing_info(deps: Deps) -> StdResult<MarketingInfoResponse> {
    Ok(MARKETING_INFO.may_load(deps.storage)?.unwrap_or_default())
}

pub fn query_download_logo(deps: Deps) -> StdResult<DownloadLogoResponse> {
    let logo = LOGO.load(deps.storage)?;
    match logo {
        Logo::Embedded(EmbeddedLogo::Svg(logo)) =>
            Ok(DownloadLogoResponse {
                mime_type: "image/svg+xml".to_owned(),
                data: logo,
            }),
        Logo::Embedded(EmbeddedLogo::Png(logo)) =>
            Ok(DownloadLogoResponse {
                mime_type: "image/png".to_owned(),
                data: logo,
            }),
        Logo::Url(_) => Err(StdError::not_found("logo")),
    }
}
