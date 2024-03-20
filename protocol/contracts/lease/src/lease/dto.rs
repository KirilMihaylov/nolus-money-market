use serde::{Deserialize, Serialize};

use currency::Currency;
use lpp::stub::loan::LppLoan as LppLoanTrait;
use oracle_platform::{Oracle as OracleTrait, OracleRef};
use sdk::cosmwasm_std::{Addr, QuerierWrapper};
use timealarms::stub::TimeAlarmsRef;

use crate::{
    finance::{LpnCurrencies, LpnCurrency, ReserveRef},
    loan::LoanDTO,
    position::PositionDTO,
};

use super::{
    with_lease::WithLease,
    with_lease_deps::{self, WithLeaseDeps},
    Lease,
};

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(test, derive(Debug))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct LeaseDTO {
    pub(crate) addr: Addr,
    pub(crate) customer: Addr,
    pub(crate) position: PositionDTO,
    pub(crate) loan: LoanDTO,
    pub(crate) time_alarms: TimeAlarmsRef,
    pub(crate) oracle: OracleRef,
    pub(crate) reserve: ReserveRef,
}

impl LeaseDTO {
    pub(crate) fn new(
        addr: Addr,
        customer: Addr,
        position: PositionDTO,
        loan: LoanDTO,
        time_alarms: TimeAlarmsRef,
        oracle: OracleRef,
        reserve: ReserveRef,
    ) -> Self {
        Self {
            addr,
            customer,
            position,
            loan,
            time_alarms,
            oracle,
            reserve,
        }
    }

    pub(crate) fn execute<Cmd>(
        self,
        cmd: Cmd,
        querier: QuerierWrapper<'_>,
    ) -> Result<Cmd::Output, Cmd::Error>
    where
        Cmd: WithLease,
        Cmd::Error: From<lpp::error::ContractError>,
        currency::error::Error: Into<Cmd::Error>,
        timealarms::error::ContractError: Into<Cmd::Error>,
        oracle_platform::error::Error: Into<Cmd::Error>,
    {
        let lease = self.addr.clone();
        let asset = self.position.amount().ticker().clone();
        let lpp = self.loan.lpp().clone();
        let oracle = self.oracle.clone();

        with_lease_deps::execute(Factory::new(cmd, self), lease, &asset, lpp, oracle, querier)
    }
}

struct Factory<Cmd> {
    cmd: Cmd,
    lease_dto: LeaseDTO,
}
impl<Cmd> Factory<Cmd> {
    pub(super) fn new(cmd: Cmd, lease_dto: LeaseDTO) -> Self {
        Self { cmd, lease_dto }
    }
}

impl<Cmd> WithLeaseDeps for Factory<Cmd>
where
    Cmd: WithLease,
{
    type Output = Cmd::Output;
    type Error = Cmd::Error;

    fn exec<Lpn, Asset, LppLoan, Oracle>(
        self,
        lpp_loan: LppLoan,
        oracle: Oracle,
    ) -> Result<Self::Output, Self::Error>
    where
        Lpn: Currency,
        Asset: Currency,
        LppLoan: LppLoanTrait<LpnCurrency, LpnCurrencies>,
        Oracle: OracleTrait<LpnCurrency>,
    {
        self.cmd.exec(Lease::<Asset, _, _>::from_dto(
            self.lease_dto,
            lpp_loan,
            oracle,
        ))
    }
}
