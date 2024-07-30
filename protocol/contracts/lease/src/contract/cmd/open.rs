use currency::{Currency, MemberOf};
use lpp::stub::loan::LppLoan as LppLoanTrait;
use oracle_platform::Oracle as OracleTrait;
use profit::stub::ProfitRef;
use sdk::cosmwasm_std::{Addr, QuerierWrapper, Timestamp};
use timealarms::stub::TimeAlarmsRef;

use crate::{
    api::{open::NewLeaseForm, LeaseAssetCurrencies, LeaseCoin, LeasePaymentCurrencies},
    error::{ContractError, ContractResult},
    finance::{LpnCurrencies, LpnCurrency, LppRef, OracleRef, ReserveRef},
    lease::{
        with_lease_deps::{self, WithLeaseDeps},
        IntoDTOResult, Lease,
    },
    loan::Loan,
    position::{Position, PositionDTO},
};

use super::{check_debt, LiquidationStatus};

pub(crate) fn open_lease(
    form: NewLeaseForm,
    lease_addr: Addr,
    start_at: Timestamp,
    now: &Timestamp,
    asset: LeaseCoin,
    querier: QuerierWrapper<'_>,
    deps: (LppRef, OracleRef, TimeAlarmsRef),
) -> ContractResult<IntoDTOResult> {
    debug_assert_eq!(asset.ticker(), &form.currency);
    debug_assert!(asset.amount() > 0);

    let position = PositionDTO::new(asset, form.position_spec.clone());
    let profit = ProfitRef::new(form.loan.profit.clone(), &querier)?;
    let reserve = ReserveRef::try_new(form.reserve.clone(), &querier)?;
    let cmd = LeaseFactory {
        form,
        lease_addr: lease_addr.clone(),
        profit,
        reserve,
        time_alarms: deps.2,
        price_alarms: deps.1.clone(),
        start_at,
        now,
    };
    with_lease_deps::execute(cmd, lease_addr, position, deps.0, deps.1, querier)
}

struct LeaseFactory<'a> {
    form: NewLeaseForm,
    lease_addr: Addr,
    profit: ProfitRef,
    reserve: ReserveRef,
    time_alarms: TimeAlarmsRef,
    price_alarms: OracleRef,
    start_at: Timestamp,
    now: &'a Timestamp,
}

impl<'a> WithLeaseDeps for LeaseFactory<'a> {
    type Output = IntoDTOResult;
    type Error = ContractError;

    fn exec<Lpn, Asset, LppLoan, Oracle>(
        self,
        position: Position<Asset>,
        lpp_loan: LppLoan,
        oracle: Oracle,
    ) -> Result<Self::Output, Self::Error>
    where
        Asset: Currency + MemberOf<LeaseAssetCurrencies> + MemberOf<LeasePaymentCurrencies>,
        LppLoan: LppLoanTrait<LpnCurrency, LpnCurrencies>,
        Oracle: OracleTrait<LeasePaymentCurrencies, QuoteC = LpnCurrency, QuoteG = LpnCurrencies>
            + Into<OracleRef>,
    {
        let lease = {
            let loan = Loan::new(
                lpp_loan,
                self.start_at,
                self.form.loan.annual_margin_interest,
                self.form.loan.due_period,
            );
            Lease::new(self.lease_addr, self.form.customer, position, loan, oracle)
        };

        let alarms = match check_debt::check_debt(
            &lease,
            self.now,
            &self.time_alarms,
            &self.price_alarms,
        )? {
            LiquidationStatus::NoDebt => unreachable!(),
            LiquidationStatus::NewAlarms {
                current_liability: _,
                alarms,
            } => alarms,
            LiquidationStatus::NeedLiquidation(_) => unreachable!(),
        };

        lease
            .try_into_dto(self.profit, self.time_alarms, self.reserve)
            .map(|mut dto| {
                dto.batch = dto.batch.merge(alarms);
                dto
            })
    }
}
