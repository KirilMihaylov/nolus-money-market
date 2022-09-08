use cosmwasm_std::{Addr, Timestamp};
use serde::Serialize;

use finance::{
    coin::Coin,
    currency::{Currency, SymbolOwned},
    fraction::Fraction,
    percent::{Percent, Units},
    price::{total, Price},
    ratio::Rational,
};
use lpp::stub::Lpp as LppTrait;
use market_price_oracle::stub::Oracle as OracleTrait;
use platform::{batch::Batch, generate_ids};
use time_alarms::stub::TimeAlarms as TimeAlarmsTrait;

use crate::{error::ContractResult, lease::Lease, loan::LiabilityStatus};

use super::LeaseDTO;

mod alarm;

impl<Lpn, Lpp, TimeAlarms, Oracle> Lease<Lpn, Lpp, TimeAlarms, Oracle>
    where
        Lpn: Currency + Serialize,
        Lpp: LppTrait<Lpn>,
        TimeAlarms: TimeAlarmsTrait,
        Oracle: OracleTrait<Lpn>,
{
    fn liquidate_on_interest_overdue(
        &self,
        now: Timestamp,
        lease: Addr,
        lease_amount: Coin<Lpn>,
    ) -> ContractResult<Status<Lpn>> {
        let lease_lpn = total(lease_amount, self.price_of_lease_currency()?);

        let LiabilityStatus { ltv, overdue, .. } =
            self.loan.liability_status(now, lease, lease_lpn)?;

        let liquidation_lpn = lease_lpn.min(overdue);

        self.liquidate(
            self.customer.clone(),
            self.currency.clone(),
            lease_lpn,
            liquidation_lpn,
        );

        let liquidation_amount = total(liquidation_lpn, self.price_of_lease_currency()?.inv());

        let info = LeaseInfo {
            customer: self.customer.clone(),
            ltv,
            lease_asset: self.currency.clone(),
        };

        Ok(if liquidation_amount == lease_amount {
            Status::FullLiquidation(info)
        } else {
            Status::PartialLiquidation {
                _info: info,
                _healthy_ltv: self.liability.healthy_percent(),
                liquidation_amount,
            }
        })
    }

    fn act_on_liability(
        &self,
        now: Timestamp,
        lease: Addr,
        lease_amount: Coin<Lpn>,
        price: Price<Lpn, Lpn>,
    ) -> ContractResult<Status<Lpn>> {
        let lease_lpn = total(lease_amount, price);

        let LiabilityStatus { ltv, total, .. } =
            self.loan.liability_status(now, lease, lease_lpn)?;

        Ok(if self.liability.max_percent() <= ltv {
            self.liquidate(
                self.customer.clone(),
                self.currency.clone(),
                lease_lpn,
                total,
            )
        } else {
            self.handle_warnings(ltv)
        })
    }

    fn handle_warnings(&self, liability: Percent) -> Status<Lpn> {
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
            LeaseInfo {
                customer: self.customer.clone(),
                ltv,
                lease_asset: self.currency.clone(),
            },
            level,
        )
    }

    fn liquidate(
        &self,
        customer: Addr,
        lease_asset: SymbolOwned,
        lease_lpn: Coin<Lpn>,
        liability_lpn: Coin<Lpn>,
    ) -> Status<Lpn> {
        // from 'liability - liquidation = healthy% of (lease - liquidation)' follows
        // 'liquidation = 100% / (100% - healthy%) of (liability - healthy% of lease)'
        let multiplier = Rational::new(
            Percent::HUNDRED,
            Percent::HUNDRED - self.liability.healthy_percent(),
        );
        let extra_liability =
            liability_lpn - liability_lpn.min(self.liability.healthy_percent().of(lease_lpn));
        let liquidation_amount =
            <Rational<Percent> as Fraction<Units>>::of(&multiplier, extra_liability);
        let liquidation_amount = lease_lpn.min(liquidation_amount);
        // TODO perform actual liquidation

        let info = LeaseInfo {
            customer,
            ltv: self.liability.max_percent(),
            lease_asset,
        };

        if liquidation_amount == lease_lpn {
            Status::FullLiquidation(info)
        } else {
            Status::PartialLiquidation {
                _info: info,
                _healthy_ltv: self.liability.healthy_percent(),
                liquidation_amount,
            }
        }
    }
}

pub(crate) struct OnAlarmResult<Lpn>
    where
        Lpn: Currency,
{
    pub batch: Batch,
    pub lease_dto: LeaseDTO,
    pub liquidation_status: Status<Lpn>,
}

#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub(crate) enum Status<Lpn>
    where
        Lpn: Currency,
{
    None,
    Warning(LeaseInfo, WarningLevel),
    PartialLiquidation {
        _info: LeaseInfo,
        _healthy_ltv: Percent,
        liquidation_amount: Coin<Lpn>,
    },
    FullLiquidation(LeaseInfo),
}

#[cfg_attr(test, derive(Debug, Eq, PartialEq))]
pub(crate) struct LeaseInfo {
    pub customer: Addr,
    pub ltv: Percent,
    pub lease_asset: SymbolOwned,
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
    use cosmwasm_std::{Addr, Timestamp};

    use finance::{currency::Currency, percent::Percent};
    use lpp::msg::LoanResponse;

    use crate::{
        lease::{
            tests::TestCurrency,
            tests::{coin, lease_setup, LEASE_START},
            LeaseInfo, Status, WarningLevel,
        },
        loan::LiabilityStatus,
    };

    #[test]
    fn warnings_none() {
        let _lease_amount = 1000;
        let interest_rate = Percent::from_permille(50);
        // LPP loan
        let loan = LoanResponse {
            principal_due: coin(500),
            interest_due: coin(100),
            annual_interest_rate: interest_rate,
            interest_paid: Timestamp::from_nanos(0),
        };

        let lease = lease_setup(
            Some(loan),
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
        let _lease_amount = 1000;
        let interest_rate = Percent::from_permille(50);
        // LPP loan
        let loan = LoanResponse {
            principal_due: coin(500),
            interest_due: coin(100),
            annual_interest_rate: interest_rate,
            interest_paid: Timestamp::from_nanos(0),
        };

        let lease = lease_setup(
            Some(loan),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
        );

        assert_eq!(
            lease.handle_warnings(lease.liability.first_liq_warn_percent()),
            Status::Warning(
                LeaseInfo {
                    customer: lease.customer.clone(),
                    ltv: lease.liability.first_liq_warn_percent(),
                    lease_asset: TestCurrency::SYMBOL.into(),
                },
                WarningLevel::First,
            )
        );
    }

    #[test]
    fn warnings_second() {
        let _lease_amount = 1000;
        let interest_rate = Percent::from_permille(50);
        // LPP loan
        let loan = LoanResponse {
            principal_due: coin(500),
            interest_due: coin(100),
            annual_interest_rate: interest_rate,
            interest_paid: Timestamp::from_nanos(0),
        };

        let lease = lease_setup(
            Some(loan),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
        );

        assert_eq!(
            lease.handle_warnings(lease.liability.second_liq_warn_percent()),
            Status::Warning(
                LeaseInfo {
                    customer: lease.customer.clone(),
                    ltv: lease.liability.second_liq_warn_percent(),
                    lease_asset: TestCurrency::SYMBOL.into(),
                },
                WarningLevel::Second,
            )
        );
    }

    #[test]
    fn warnings_third() {
        let _lease_amount = 1000;
        let interest_rate = Percent::from_permille(50);
        // LPP loan
        let loan = LoanResponse {
            principal_due: coin(500),
            interest_due: coin(100),
            annual_interest_rate: interest_rate,
            interest_paid: Timestamp::from_nanos(0),
        };

        let lease = lease_setup(
            Some(loan),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
        );

        assert_eq!(
            lease.handle_warnings(lease.liability.third_liq_warn_percent()),
            Status::Warning(
                LeaseInfo {
                    customer: lease.customer.clone(),
                    ltv: lease.liability.third_liq_warn_percent(),
                    lease_asset: TestCurrency::SYMBOL.into(),
                },
                WarningLevel::Third,
            )
        );
    }

    #[test]
    fn liability() {
        let interest_rate = Percent::from_permille(50);
        // LPP loan
        let loan = LoanResponse {
            principal_due: coin(500),
            interest_due: coin(100),
            annual_interest_rate: interest_rate,
            interest_paid: Timestamp::from_nanos(0),
        };

        let lease = lease_setup(
            Some(loan),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
        );

        assert_eq!(
            lease
                .loan
                .liability_status(LEASE_START, Addr::unchecked(String::new()), coin(1000))
                .unwrap(),
            LiabilityStatus {
                ltv: Percent::from_percent(60),
                total: coin(100 + 500),
                overdue: coin(0),
            }
        );
    }

    #[test]
    fn liquidate_partial() {
        let lease_amount = 1000;
        let interest_rate = Percent::from_permille(50);
        // LPP loan
        let loan = LoanResponse {
            principal_due: coin(500),
            interest_due: coin(100),
            annual_interest_rate: interest_rate,
            interest_paid: Timestamp::from_nanos(0),
        };

        let lease = lease_setup(
            Some(loan),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
        );

        assert_eq!(
            lease.liquidate(
                Addr::unchecked(String::new()),
                String::new(),
                coin(lease_amount),
                coin(800),
            ),
            Status::PartialLiquidation {
                _info: LeaseInfo {
                    customer: Addr::unchecked(String::new()),
                    ltv: lease.liability.max_percent(),
                    lease_asset: "".into(),
                },
                _healthy_ltv: lease.liability.healthy_percent(),
                liquidation_amount: coin(333),
            }
        );
    }

    #[test]
    fn liquidate_full() {
        let lease_amount = 1000;
        let interest_rate = Percent::from_permille(50);
        // LPP loan
        let loan = LoanResponse {
            principal_due: coin(500),
            interest_due: coin(100),
            annual_interest_rate: interest_rate,
            interest_paid: Timestamp::from_nanos(0),
        };

        let lease = lease_setup(
            Some(loan),
            Addr::unchecked(String::new()),
            Addr::unchecked(String::new()),
        );

        assert_eq!(
            lease.liquidate(
                Addr::unchecked(String::new()),
                String::new(),
                coin(lease_amount),
                coin(5000),
            ),
            Status::FullLiquidation(LeaseInfo {
                customer: Addr::unchecked(String::new()),
                ltv: lease.liability.max_percent(),
                lease_asset: "".into(),
            },)
        );
    }
}
