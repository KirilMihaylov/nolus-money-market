use serde::{Deserialize, Serialize};

use currency::{CurrencyDef, MemberOf};
use finance::liability::Zone;
use lpp::stub::loan::LppLoan as LppLoanTrait;
use oracle_platform::Oracle as OracleTrait;
use platform::batch::Batch;
use sdk::cosmwasm_std::Timestamp;
use timealarms::stub::TimeAlarmsRef;

use crate::{
    api::{LeaseAssetCurrencies, LeaseCoin, LeasePaymentCurrencies},
    error::{ContractError, ContractResult},
    finance::{LpnCurrencies, LpnCurrency, OracleRef},
    lease::{with_lease::WithLease, CloseStatus, Lease as LeaseDO},
    position::{Cause, Liquidation},
};

pub(crate) fn check_close<Asset, Lpp, Oracle>(
    lease: &LeaseDO<Asset, Lpp, Oracle>,
    when: &Timestamp,
    time_alarms: &TimeAlarmsRef,
    price_alarms: &OracleRef,
) -> ContractResult<CloseStatusDTO>
where
    Asset: CurrencyDef,
    Asset::Group: MemberOf<LeaseAssetCurrencies> + MemberOf<LeasePaymentCurrencies>,
    Lpp: LppLoanTrait<LpnCurrency, LpnCurrencies>,
    Oracle: OracleTrait<LeasePaymentCurrencies, QuoteC = LpnCurrency, QuoteG = LpnCurrencies>,
{
    lease
        .check_close(when, time_alarms, price_alarms)
        .map(Into::into)
}

pub(crate) struct Cmd<'a> {
    now: &'a Timestamp,
    time_alarms: &'a TimeAlarmsRef,
    price_alarms: &'a OracleRef,
}

pub(crate) enum CloseStatusDTO {
    NoDebt,
    NewAlarms {
        current_liability: Zone,
        alarms: Batch,
    },
    NeedLiquidation(LiquidationDTO),
}

#[derive(Serialize, Deserialize)]
pub(crate) enum LiquidationDTO {
    Partial(PartialLiquidationDTO),
    Full(FullLiquidationDTO),
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PartialLiquidationDTO {
    pub amount: LeaseCoin,
    pub cause: Cause,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct FullLiquidationDTO {
    pub cause: Cause,
}

impl<Asset> From<CloseStatus<Asset>> for CloseStatusDTO
where
    Asset: CurrencyDef,
    Asset::Group: MemberOf<LeaseAssetCurrencies>,
{
    fn from(value: CloseStatus<Asset>) -> Self {
        match value {
            CloseStatus::NoDebt => Self::NoDebt,
            CloseStatus::NewAlarms {
                current_liability,
                alarms,
            } => Self::NewAlarms {
                current_liability,
                alarms,
            },
            CloseStatus::NeedLiquidation(liquidation) => Self::NeedLiquidation(liquidation.into()),
        }
    }
}

impl<Asset> From<Liquidation<Asset>> for LiquidationDTO
where
    Asset: CurrencyDef,
    Asset::Group: MemberOf<LeaseAssetCurrencies>,
{
    fn from(value: Liquidation<Asset>) -> Self {
        match value {
            Liquidation::Partial { amount, cause } => Self::Partial(PartialLiquidationDTO {
                amount: amount.into(),
                cause,
            }),
            Liquidation::Full(cause) => Self::Full(FullLiquidationDTO { cause }),
        }
    }
}

impl<'a> Cmd<'a> {
    pub fn new(
        now: &'a Timestamp,
        time_alarms: &'a TimeAlarmsRef,
        price_alarms: &'a OracleRef,
    ) -> Self {
        Self {
            now,
            time_alarms,
            price_alarms,
        }
    }
}

impl<'a> WithLease for Cmd<'a> {
    type Output = CloseStatusDTO;

    type Error = ContractError;

    fn exec<Asset, Loan, Oracle>(
        self,
        lease: LeaseDO<Asset, Loan, Oracle>,
    ) -> Result<Self::Output, Self::Error>
    where
        Asset: CurrencyDef,
        Asset::Group: MemberOf<LeaseAssetCurrencies> + MemberOf<LeasePaymentCurrencies>,
        Loan: LppLoanTrait<LpnCurrency, LpnCurrencies>,
        Oracle: OracleTrait<LeasePaymentCurrencies, QuoteC = LpnCurrency, QuoteG = LpnCurrencies>,
    {
        check_close(&lease, self.now, self.time_alarms, self.price_alarms)
    }
}
