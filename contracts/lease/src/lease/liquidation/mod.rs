use std::marker::PhantomData;

use serde::Serialize;

use finance::{coin::Coin, currency::Currency, percent::Percent};
use lpp::stub::lender::LppLender as LppLenderTrait;
use oracle::stub::Oracle as OracleTrait;
use platform::{batch::Batch, generate_ids};
use profit::stub::Profit as ProfitTrait;
use sdk::cosmwasm_std::{Addr, Timestamp};
use timealarms::stub::TimeAlarms as TimeAlarmsTrait;

use crate::{
    error::ContractResult,
    lease::Lease,
    loan::{LiabilityStatus, RepayReceipt},
};

use super::LeaseDTO;

mod alarm;

impl<Lpn, Asset, Lpp, Profit, TimeAlarms, Oracle> Lease<Lpn, Asset, Lpp, Profit, TimeAlarms, Oracle>
where
    Lpn: Currency + Serialize,
    Lpp: LppLenderTrait<Lpn>,
    TimeAlarms: TimeAlarmsTrait,
    Oracle: OracleTrait<Lpn>,
    Profit: ProfitTrait,
    Asset: Currency + Serialize,
{
    fn act_on_overdue(
        &mut self,
        lease_lpn: Coin<Lpn>,
        now: Timestamp,
        ltv: Percent,
        _: Coin<Lpn>,
    ) -> ContractResult<Status<Lpn, Asset>> {
        if self.loan.grace_period_end() <= now {
            self.liquidate_on_interest_overdue(now, lease_lpn)
        } else {
            Ok(self.handle_warnings(ltv))
        }
    }

    fn act_on_liability(
        &mut self,
        lease_lpn: Coin<Lpn>,
        now: Timestamp,
        ltv: Percent,
        liability_lpn: Coin<Lpn>,
    ) -> ContractResult<Status<Lpn, Asset>> {
        if self.liability.max_percent() <= ltv {
            self.liquidate_on_liability(lease_lpn, liability_lpn, now)
        } else {
            Ok(self.handle_warnings(ltv))
        }
    }

    fn handle_warnings(&self, liability: Percent) -> Status<Lpn, Asset> {
        debug_assert!(liability < self.liability.max_percent());

        if liability < self.liability.first_liq_warn_percent() {
            return Status::None;
        }

        let (ltv, level) = if self.liability.third_liq_warn_percent() <= liability {
            (self.liability.third_liq_warn_percent(), WarningLevel::Third)
        } else if self.liability.second_liq_warn_percent() <= liability {
            (
                self.liability.second_liq_warn_percent(),
                WarningLevel::Second,
            )
        } else {
            debug_assert!(self.liability.first_liq_warn_percent() <= liability);
            (self.liability.first_liq_warn_percent(), WarningLevel::First)
        };

        Status::Warning(
            LeaseInfo::new(self.customer.clone(), self.addr.clone(), ltv),
            level,
        )
    }

    fn liquidate_on_liability(
        &mut self,
        lease_lpn: Coin<Lpn>,
        liability_lpn: Coin<Lpn>,
        now: Timestamp,
    ) -> ContractResult<Status<Lpn, Asset>> {
        let liquidation_lpn = self.liability.amount_to_liquidate(lease_lpn, liability_lpn);

        self.liquidate(
            Cause::Liability,
            lease_lpn,
            liquidation_lpn,
            now,
            self.liability.max_percent(),
        )
    }

    fn liquidate_on_interest_overdue(
        &mut self,
        now: Timestamp,
        lease_lpn: Coin<Lpn>,
    ) -> ContractResult<Status<Lpn, Asset>> {
        let LiabilityStatus {
            ltv, overdue_lpn, ..
        } = self
            .loan
            .liability_status(now, self.addr.clone(), lease_lpn)?;

        self.liquidate(Cause::Overdue, lease_lpn, overdue_lpn, now, ltv)
    }

    fn liquidate(
        &mut self,
        cause: Cause,
        lease_lpn: Coin<Lpn>,
        mut liquidation_lpn: Coin<Lpn>,
        now: Timestamp,
        ltv: Percent,
    ) -> ContractResult<Status<Lpn, Asset>> {
        liquidation_lpn = lease_lpn.min(liquidation_lpn);

        let receipt = self.no_reschedule_repay(liquidation_lpn, now)?;

        let info = LeaseInfo::new(self.customer.clone(), self.addr.clone(), ltv);

        let liquidation_info = LiquidationInfo {
            cause,
            lease: self.addr.clone(),
            receipt,
        };

        // TODO liquidate fully if the remaining value, lease_lpn - liquidation_lpn < 100
        Ok(if liquidation_lpn == lease_lpn {
            Status::FullLiquidation {
                info,
                liquidation_info,
            }
        } else {
            Status::PartialLiquidation {
                info,
                liquidation_info,
                healthy_ltv: self.liability.healthy_percent(),
            }
        })
    }
}

pub(crate) struct OnAlarmResult<Lpn, Asset>
where
    Lpn: Currency,
    Asset: Currency,
{
    pub batch: Batch,
    pub lease_dto: LeaseDTO,
    pub liquidation_status: Status<Lpn, Asset>,
}

#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub(crate) enum Status<Lpn, Asset>
where
    Lpn: Currency,
    Asset: Currency,
{
    None,
    Warning(LeaseInfo<Asset>, WarningLevel),
    PartialLiquidation {
        info: LeaseInfo<Asset>,
        liquidation_info: LiquidationInfo<Lpn>,
        healthy_ltv: Percent,
    },
    FullLiquidation {
        info: LeaseInfo<Asset>,
        liquidation_info: LiquidationInfo<Lpn>,
    },
}

#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub(crate) struct LeaseInfo<Asset>
where
    Asset: Currency,
{
    pub customer: Addr,
    pub lease: Addr,
    pub ltv: Percent,
    _asset: PhantomData<Asset>,
}

impl<Asset> LeaseInfo<Asset>
where
    Asset: Currency,
{
    pub fn new(customer: Addr, lease: Addr, ltv: Percent) -> Self {
        Self {
            customer,
            lease,
            ltv,
            _asset: PhantomData,
        }
    }
}

#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub(crate) struct LiquidationInfo<Lpn>
where
    Lpn: Currency,
{
    pub cause: Cause,
    pub lease: Addr,
    pub receipt: RepayReceipt<Lpn>,
}

generate_ids! {
    pub(crate) Cause as u8 {
        Overdue = 1,
        Liability = 2,
    }
}

impl Cause {
    pub fn to_uint(self) -> u8 {
        self.into()
    }
}

generate_ids! {
    pub(crate) WarningLevel as u8 {
        First = 1,
        Second = 2,
        Third = 3,
    }
}

impl WarningLevel {
    pub fn to_uint(self) -> u8 {
        self.into()
    }
}

#[cfg(test)]
mod tests {
    use finance::{duration::Duration, interest::InterestPeriod, percent::Percent};
    use lpp::msg::LoanResponse;
    use sdk::cosmwasm_std::{Addr, Timestamp};

    use crate::{
        lease::{
            tests::{coin, loan, lpn_coin, open_lease, LEASE_START, MARGIN_INTEREST_RATE},
            LeaseInfo, LiquidationInfo, Status, WarningLevel,
        },
        loan::{LiabilityStatus, RepayReceipt},
    };

    use super::Cause;

    #[test]
    fn warnings_none() {
        let interest_rate = Percent::from_permille(50);
        // LPP loan
        let loan = LoanResponse {
            principal_due: lpn_coin(500),
            annual_interest_rate: interest_rate,
            interest_paid: Timestamp::from_nanos(0),
        };

        let lease_addr = Addr::unchecked("lease");
        let lease = open_lease(
            lease_addr,
            10.into(),
            Some(loan),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
        );

        assert_eq!(
            lease.handle_warnings(Percent::from_percent(60)),
            Status::None,
        );
    }

    #[test]
    fn warnings_first() {
        let interest_rate = Percent::from_permille(50);
        // LPP loan
        let loan = LoanResponse {
            principal_due: lpn_coin(500),
            annual_interest_rate: interest_rate,
            interest_paid: Timestamp::from_nanos(0),
        };

        let lease_addr = Addr::unchecked("lease");
        let lease = open_lease(
            lease_addr,
            10.into(),
            Some(loan),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
        );

        assert_eq!(
            lease.handle_warnings(lease.liability.first_liq_warn_percent()),
            Status::Warning(
                LeaseInfo::new(
                    lease.customer,
                    lease.addr,
                    lease.liability.first_liq_warn_percent(),
                ),
                WarningLevel::First,
            )
        );
    }

    #[test]
    fn warnings_second() {
        let interest_rate = Percent::from_permille(50);
        // LPP loan
        let loan = LoanResponse {
            principal_due: lpn_coin(500),
            annual_interest_rate: interest_rate,
            interest_paid: Timestamp::from_nanos(0),
        };

        let lease_addr = Addr::unchecked("lease");
        let lease = open_lease(
            lease_addr,
            10.into(),
            Some(loan),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
        );

        assert_eq!(
            lease.handle_warnings(lease.liability.second_liq_warn_percent()),
            Status::Warning(
                LeaseInfo::new(
                    lease.customer,
                    lease.addr,
                    lease.liability.second_liq_warn_percent(),
                ),
                WarningLevel::Second,
            )
        );
    }

    #[test]
    fn warnings_third() {
        let lease_addr = Addr::unchecked("lease");
        let lease = open_lease(
            lease_addr,
            10.into(),
            Some(loan()),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
        );

        assert_eq!(
            lease.handle_warnings(lease.liability.third_liq_warn_percent()),
            Status::Warning(
                LeaseInfo::new(
                    lease.customer,
                    lease.addr,
                    lease.liability.third_liq_warn_percent(),
                ),
                WarningLevel::Third,
            )
        );
    }

    #[test]
    fn liability() {
        // LPP loan
        let loan = LoanResponse {
            principal_due: lpn_coin(500),
            annual_interest_rate: Percent::from_permille(50),
            interest_paid: LEASE_START,
        };

        let lease_addr = Addr::unchecked("lease");
        let lease = open_lease(
            lease_addr,
            10.into(),
            Some(loan),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
        );
        // lease.repay();
        // 100 days period
        // 100 interest due
        // let interest_due = 100.into();

        assert_eq!(
            lease
                .loan
                .liability_status(LEASE_START, Addr::unchecked(String::new()), lpn_coin(1000))
                .unwrap(),
            LiabilityStatus {
                ltv: Percent::from_percent(50),
                total_lpn: lpn_coin(500),
                overdue_lpn: lpn_coin(0),
            }
        );
    }

    #[test]
    fn liquidate_partial() {
        let lease_amount = coin(100);
        let lease_amount_lpn = lpn_coin(800);
        let loan_amount_lpn = lpn_coin(500);
        let past_open = Duration::from_days(90);
        let now = LEASE_START + past_open;
        let interest_rate = Percent::from_percent(114);
        // LPP loan
        let loan = LoanResponse {
            principal_due: loan_amount_lpn,
            annual_interest_rate: interest_rate,
            interest_paid: LEASE_START,
        };

        let lease_addr = Addr::unchecked("lease");
        let mut lease = open_lease(
            lease_addr,
            lease_amount,
            Some(loan.clone()),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
        );

        let interest_due = loan.interest_due(now);
        let exp_liquidation_lpn = lpn_coin(273);
        let margin_due = InterestPeriod::with_interest(MARGIN_INTEREST_RATE)
            .from(LEASE_START)
            .spanning(past_open)
            .interest(loan_amount_lpn);

        assert_eq!(
            lease
                .liquidate_on_liability(
                    lease_amount_lpn,
                    loan_amount_lpn + interest_due + margin_due,
                    now
                )
                .unwrap(),
            Status::PartialLiquidation {
                info: LeaseInfo::new(
                    Addr::unchecked("customer"),
                    lease.addr.clone(),
                    lease.liability.max_percent()
                ),
                liquidation_info: LiquidationInfo {
                    cause: Cause::Liability,
                    lease: lease.addr,
                    receipt: RepayReceipt::new(
                        lpn_coin(0),
                        margin_due,
                        lpn_coin(0),
                        interest_due,
                        exp_liquidation_lpn - interest_due - margin_due,
                        lpn_coin(0),
                        false
                    ),
                },
                healthy_ltv: lease.liability.healthy_percent(),
            }
        );
    }

    #[test]
    fn liquidate_full() {
        let lease_amount = coin(100);
        let lease_amount_lpn = lpn_coin(800);
        let loan_amount_lpn = lpn_coin(500);
        let past_open = Duration::from_days(90);
        let now = LEASE_START + past_open;
        let interest_rate = Percent::from_percent(242);
        // LPP loan
        let loan = LoanResponse {
            principal_due: loan_amount_lpn,
            annual_interest_rate: interest_rate,
            interest_paid: LEASE_START,
        };
        let interest_due = loan.interest_due(now);
        let exp_liquidation_lpn = lease_amount_lpn;
        let margin_due = InterestPeriod::with_interest(MARGIN_INTEREST_RATE)
            .from(LEASE_START)
            .spanning(past_open)
            .interest(loan_amount_lpn);

        let lease_addr = Addr::unchecked("lease");
        let mut lease = open_lease(
            lease_addr,
            lease_amount,
            Some(loan),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
        );

        assert_eq!(
            lease
                .liquidate_on_liability(
                    lease_amount_lpn,
                    loan_amount_lpn + interest_due + margin_due,
                    now
                )
                .unwrap(),
            Status::FullLiquidation {
                info: LeaseInfo::new(
                    Addr::unchecked("customer"),
                    lease.addr.clone(),
                    lease.liability.max_percent()
                ),
                liquidation_info: LiquidationInfo {
                    cause: Cause::Liability,
                    lease: lease.addr,
                    receipt: RepayReceipt::new(
                        lpn_coin(0),
                        margin_due,
                        lpn_coin(0),
                        interest_due,
                        exp_liquidation_lpn - margin_due - interest_due,
                        lpn_coin(0),
                        true,
                    ),
                },
            }
        );
    }
}
