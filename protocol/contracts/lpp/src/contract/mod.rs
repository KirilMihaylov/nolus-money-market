use std::ops::DerefMut as _;

use finance::coin::CoinDTO;
use serde::Serialize;

use access_control::SingleUserAccess;
use currencies::{Lpn as LpnCurrency, Lpns as LpnCurrencies};

use platform::{contract::Code, message::Response as PlatformResponse, response};
use sdk::{
    cosmwasm_ext::Response as CwResponse,
    cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo},
};
use versioning::{package_version, version, FullUpdateOutput, SemVer, Version, VersionSegment};

use crate::{
    error::{ContractError, Result},
    lpp::{LiquidityPool, LppBalances},
    msg::{ExecuteMsg, InstantiateMsg, LppBalanceResponse, MigrateMsg, QueryMsg, SudoMsg},
    state::{self, Config},
};

mod borrow;
mod lender;
mod rewards;

const CONTRACT_STORAGE_VERSION_FROM: VersionSegment = 1;
const CONTRACT_STORAGE_VERSION: VersionSegment = 2;
const PACKAGE_VERSION: SemVer = package_version!();
const CONTRACT_VERSION: Version = version!(CONTRACT_STORAGE_VERSION, PACKAGE_VERSION);

#[entry_point]
pub fn instantiate(
    mut deps: DepsMut<'_>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<CwResponse> {
    // TODO move these checks on deserialization
    currency::validate::<LpnCurrencies>(&msg.lpn_ticker)?;
    deps.api.addr_validate(msg.lease_code_admin.as_str())?;

    versioning::initialize(deps.storage, CONTRACT_VERSION)?;

    SingleUserAccess::new(
        deps.storage.deref_mut(),
        crate::access_control::LEASE_CODE_ADMIN_KEY,
    )
    .grant_to(&msg.lease_code_admin)?;

    Code::try_new(msg.lease_code.into(), &deps.querier)
        .map_err(Into::into)
        .and_then(|lease_code| Config::try_new::<LpnCurrency>(msg, lease_code))
        .and_then(|cfg| LiquidityPool::<LpnCurrency>::store(deps.storage, cfg))
        .map(|()| response::empty_response())
}

#[entry_point]
pub fn migrate(deps: DepsMut<'_>, _env: Env, MigrateMsg {}: MigrateMsg) -> Result<CwResponse> {
    versioning::update_software_and_storage::<CONTRACT_STORAGE_VERSION_FROM, _, _, _, _>(
        deps.storage,
        CONTRACT_VERSION,
        state::migrate::<LpnCurrency>,
        Into::into,
    )
    .and_then(
        |FullUpdateOutput {
             release_label,
             storage_migration_output: (),
         }| response::response(release_label),
    )
}

#[entry_point]
pub fn execute(
    mut deps: DepsMut<'_>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<LpnCurrencies>,
) -> Result<CwResponse> {
    match msg {
        ExecuteMsg::NewLeaseCode {
            lease_code: new_lease_code,
        } => {
            SingleUserAccess::new(
                deps.storage.deref_mut(),
                crate::access_control::LEASE_CODE_ADMIN_KEY,
            )
            .check(&info.sender)?;

            Config::update_lease_code(deps.storage, new_lease_code)
                .map(|()| PlatformResponse::default())
                .map(response::response_only_messages)
        }
        ExecuteMsg::DistributeRewards() => {
            rewards::try_distribute_rewards(deps, info).map(response::response_only_messages)
        }
        ExecuteMsg::ClaimRewards { other_recipient } => {
            rewards::try_claim_rewards(deps, env, info, other_recipient)
                .map(response::response_only_messages)
        }
        ExecuteMsg::OpenLoan { amount } => amount
            .try_into()
            .map_err(Into::into)
            .and_then(|amount_lpn| {
                borrow::try_open_loan::<LpnCurrency>(deps, env, info, amount_lpn)
            })
            .and_then(|(loan_resp, message_response)| {
                response::response_with_messages::<_, _, ContractError>(loan_resp, message_response)
            }),
        ExecuteMsg::RepayLoan() => borrow::try_repay_loan::<LpnCurrency>(deps, env, info).and_then(
            |(excess_amount, message_response)| {
                response::response_with_messages::<_, _, ContractError>(
                    excess_amount,
                    message_response,
                )
            },
        ),
        ExecuteMsg::Deposit() => lender::try_deposit::<LpnCurrency>(deps, env, info)
            .map(response::response_only_messages),
        ExecuteMsg::Burn { amount } => lender::try_withdraw::<LpnCurrency>(deps, env, info, amount)
            .map(response::response_only_messages),
    }
}

#[entry_point]
pub fn sudo(deps: DepsMut<'_>, _env: Env, msg: SudoMsg) -> Result<CwResponse> {
    // no currency context variants
    match msg {
        SudoMsg::NewBorrowRate { borrow_rate } => {
            Config::update_borrow_rate(deps.storage, borrow_rate)
        }
        SudoMsg::MinUtilization { min_utilization } => {
            Config::update_min_utilization(deps.storage, min_utilization)
        }
    }
    .map(|()| PlatformResponse::default())
    .map(response::response_only_messages)
}

#[entry_point]
pub fn query(deps: Deps<'_>, env: Env, msg: QueryMsg<LpnCurrencies>) -> Result<Binary> {
    match msg {
        QueryMsg::Config() => Config::load(deps.storage).and_then(|ref resp| to_json_binary(resp)),
        QueryMsg::Lpn() => to_json_binary(&Config::lpn_ticker::<LpnCurrency>()),
        QueryMsg::Balance { address } => {
            lender::query_balance(deps.storage, address).and_then(|ref resp| to_json_binary(resp))
        }
        QueryMsg::Rewards { address } => {
            rewards::query_rewards(deps.storage, address).and_then(|ref resp| to_json_binary(resp))
        }
        QueryMsg::Quote { amount } => amount
            .try_into()
            .map_err(Into::into)
            .and_then(|quote| borrow::query_quote::<LpnCurrency>(&deps, &env, quote))
            .and_then(|ref resp| to_json_binary(resp)),
        QueryMsg::Loan { lease_addr } => {
            borrow::query_loan::<LpnCurrency>(deps.storage, lease_addr)
                .and_then(|ref resp| to_json_binary(resp))
        }
        QueryMsg::LppBalance() => rewards::query_lpp_balance::<LpnCurrency>(deps, env)
            .map(LppBalanceResponse::from)
            .and_then(|ref resp| to_json_binary(resp)),
        QueryMsg::StableBalance {} => rewards::query_lpp_balance::<LpnCurrency>(deps, env)
            .map(|balance_lpn| {
                balance_lpn.balance
                    + balance_lpn.total_principal_due
                    + balance_lpn.total_interest_due
            })
            .and_then(|total| to_json_binary(&CoinDTO::<LpnCurrencies>::from(total))),
        QueryMsg::Price() => lender::query_ntoken_price::<LpnCurrency>(deps, env)
            .and_then(|ref resp| to_json_binary(resp)),
        QueryMsg::DepositCapacity() => {
            to_json_binary(&lender::deposit_capacity::<LpnCurrency>(deps, env)?)
        }
    }
}

impl From<LppBalances<LpnCurrency>> for LppBalanceResponse<LpnCurrencies> {
    fn from(value: LppBalances<LpnCurrency>) -> Self {
        Self {
            balance: value.balance.into(),
            total_principal_due: value.total_principal_due.into(),
            total_interest_due: value.total_interest_due.into(),
        }
    }
}

fn to_json_binary<T>(data: &T) -> Result<Binary>
where
    T: Serialize + ?Sized,
{
    cosmwasm_std::to_json_binary(data).map_err(ContractError::ConvertToBinary)
}
