#[cfg(feature = "cosmwasm-bindings")]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, Timestamp,
    BankMsg, Storage,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, QueryQuoteResponse, QueryLoanResponse, QueryLoanOutstandingInterestResponse, OutstandingInterest};
use crate::state::LPP;

// version info for migration info
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(feature = "cosmwasm-bindings", entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    LPP.init(deps, msg.denom, msg.lease_code_id)?;

    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(feature = "cosmwasm-bindings", entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {

    let lease_unchecked = info.sender;
    let funds = info.funds;

    match msg {
        ExecuteMsg::OpenLoan { amount } => try_open_loan(deps, env, lease_unchecked, amount),
        ExecuteMsg::RepayLoan => try_repay_loan(deps, env, lease_unchecked, funds),
    }
}

#[cfg_attr(feature = "cosmwasm-bindings", entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let res = match msg {
        QueryMsg::Quote { amount } => to_binary(&query_quote(&deps, &env, amount)?),
        QueryMsg::Loan { lease_addr } => to_binary(&query_loan(deps.storage, lease_addr)?),
        QueryMsg::LoanOutstandingInterest {
            lease_addr,
            outstanding_time,
        } => to_binary(&query_loan_outstanding_interest(deps.storage, lease_addr, outstanding_time)?),
    }?;

    Ok(res)
}

fn try_open_loan(deps: DepsMut, env: Env, lease_addr: Addr, amount: Coin) -> Result<Response, ContractError> {

    LPP.validate_lease_addr(&deps.as_ref(), &lease_addr)?;
    LPP.try_open_loan(deps, env, lease_addr.clone(), amount.clone())?;

    let transfer_msg = BankMsg::Send {
        to_address: lease_addr.to_string(),
        amount: vec![amount],
    };

    let response = Response::new()
        .add_attribute("method", "try_open_loan")
        .add_message(transfer_msg);

    Ok(response)
}

fn try_repay_loan(deps: DepsMut, env: Env, lease_addr: Addr, funds: Vec<Coin>) -> Result<Response, ContractError> {
    LPP.validate_lease_addr(&deps.as_ref(), &lease_addr)?;
    let return_coin = LPP.try_repay_loan(deps, env, lease_addr.clone(), funds)?;

    let mut response = Response::new()
        .add_attribute("method", "try_repay_loan");

    if !return_coin.amount.is_zero() {

        let transfer_msg = BankMsg::Send {
            to_address: lease_addr.to_string(),
            amount: vec![return_coin],
        };

        response = response.add_message(transfer_msg);
    }

    Ok(response)
}

fn query_quote(deps: &Deps, env: &Env, quote: Coin) -> Result<QueryQuoteResponse, ContractError> {
    match LPP.query_quote(deps, env, quote)? {
        Some(quote) => Ok(QueryQuoteResponse::QuoteInterestRate(quote)),
        None => Ok(QueryQuoteResponse::NoLiquidity),
    }
}

fn query_loan(storage: &dyn Storage, lease_addr: Addr) -> Result<QueryLoanResponse, ContractError> {
    LPP.query_loan(storage, lease_addr)
}

fn query_loan_outstanding_interest(
    storage: &dyn Storage,
    loan: Addr,
    outstanding_time: Timestamp,
) -> Result<QueryLoanOutstandingInterestResponse, ContractError> {
    let res = LPP.query_loan_outstanding_interest(storage, loan, outstanding_time)?
        .map(OutstandingInterest);
    Ok(res)
}


#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{coins, Uint64};
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "ust"));

        let msg = InstantiateMsg {
            denom: "ust".into(),
            lease_code_id: Uint64::new(1000),
        };
        let info = mock_info("creator", &coins(1000, "ust"));

        instantiate(deps.as_mut(), mock_env(), info, msg).expect("can't instantiate");
    }
}
