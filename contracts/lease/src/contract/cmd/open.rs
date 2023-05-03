use serde::Serialize;

use finance::{coin::Coin, currency::Currency, liability::Zone};
use lpp::stub::lender::{LppLender as LppLenderTrait, LppLenderRef};
use oracle::stub::{Oracle as OracleTrait, OracleRef};
use profit::stub::{Profit as ProfitTrait, ProfitRef};
use sdk::cosmwasm_std::{Addr, QuerierWrapper, Timestamp};
use timealarms::stub::{TimeAlarms as TimeAlarmsTrait, TimeAlarmsRef};

use crate::{
    api::{LeaseCoin, NewLeaseForm},
    error::{ContractError, ContractResult},
    lease::{
        with_lease_deps::{self, WithLeaseDeps},
        IntoDTOResult, Lease, Reschedule,
    },
    loan::Loan,
};

pub(crate) fn open_lease(
    form: NewLeaseForm,
    lease_addr: Addr,
    start_at: Timestamp,
    amount: &LeaseCoin,
    querier: &QuerierWrapper<'_>,
    deps: (LppLenderRef, OracleRef, TimeAlarmsRef),
) -> ContractResult<IntoDTOResult> {
    debug_assert_eq!(amount.ticker(), &form.currency);
    debug_assert!(amount.amount() > 0);

    let time_alarms = TimeAlarmsRef::new(form.time_alarms.clone(), querier)?;
    let profit = ProfitRef::new(form.loan.profit.clone(), querier)?;

    let cmd = LeaseFactory {
        form,
        lease_addr,
        time_alarms,
        start_at,
        amount,
    };
    //TODO avoid cloning by extending the trait WithLeaseDeps to provide it
    let asset_currency = cmd.form.currency.clone();
    with_lease_deps::execute(
        cmd,
        &asset_currency,
        deps.0,
        profit,
        deps.2,
        deps.1,
        querier,
    )
}

struct LeaseFactory<'a> {
    form: NewLeaseForm,
    lease_addr: Addr,
    time_alarms: TimeAlarmsRef,
    start_at: Timestamp,
    amount: &'a LeaseCoin,
}

impl<'a> WithLeaseDeps for LeaseFactory<'a> {
    type Output = IntoDTOResult;
    type Error = ContractError;

    fn exec<Lpn, Asset, Lpp, Profit, TimeAlarms, Oracle>(
        self,
        lpp: Lpp,
        profit: Profit,
        _alarms: TimeAlarms,
        oracle: Oracle,
    ) -> Result<Self::Output, Self::Error>
    where
        Lpn: Currency + Serialize,
        Asset: Currency + Serialize,
        Lpp: LppLenderTrait<Lpn>,
        TimeAlarms: TimeAlarmsTrait,
        Oracle: OracleTrait<Lpn>,
        Profit: ProfitTrait,
    {
        let liability = self.form.liability;

        let loan = Loan::new(
            self.start_at,
            lpp,
            self.form.loan.annual_margin_interest,
            self.form.loan.interest_payment,
            profit,
        );
        let amount: Coin<Asset> = self.amount.try_into()?;

        let mut lease = Lease::<_, Asset, _, _, _>::new(
            self.lease_addr,
            self.form.customer,
            amount,
            liability,
            loan,
            oracle,
        );

        let alarms = self.time_alarms.execute(Reschedule(
            &mut lease,
            &self.start_at,
            &Zone::no_warnings(liability.first_liq_warn()),
        ))?;
        let mut dto = lease.into_dto(alarms.time_alarms_ref);
        dto.batch = dto.batch.merge(alarms.batch);
        Ok(dto)
    }
}
