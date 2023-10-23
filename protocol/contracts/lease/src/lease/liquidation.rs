use currency::Currency;
use finance::{coin::Coin, price::Price, zero::Zero};
use lpp::stub::loan::LppLoan as LppLoanTrait;
use oracle_platform::Oracle as OracleTrait;
use sdk::cosmwasm_std::Timestamp;

use crate::{error::ContractResult, loan::LiabilityStatus, position::Status};

use super::Lease;

impl<Lpn, Asset, Lpp, Oracle> Lease<Lpn, Asset, Lpp, Oracle>
where
    Lpn: Currency,
    Lpp: LppLoanTrait<Lpn>,
    Oracle: OracleTrait<Lpn>,
    Asset: Currency,
{
    pub(crate) fn liquidation_status(&self, now: Timestamp) -> ContractResult<Status<Asset>> {
        let lpn_in_assets = self.price_of_lease_currency()?.inv();

        let LiabilityStatus {
            total: total_due,
            previous_interest,
        } = self.loan.liability_status(now);

        let overdue = if self.loan.grace_period_end() <= now {
            previous_interest
        } else {
            Coin::ZERO
        };

        let status = self
            .position
            .check_liability(total_due, overdue, lpn_in_assets);
        //TODO rename to #[cfg(debug_assertions)]
        #[cfg(debug_assertion)]
        debug_assert!(status.amount() <= self.amount());
        Ok(status)
    }

    pub(super) fn price_of_lease_currency(&self) -> ContractResult<Price<Asset, Lpn>> {
        Ok(self.oracle.price_of::<Asset>()?)
    }
}