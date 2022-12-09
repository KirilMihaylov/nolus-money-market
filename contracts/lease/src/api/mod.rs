use currency::{lease::LeaseGroup, payment::PaymentGroup};
use finance::coin::CoinDTO;
use serde::{Deserialize, Serialize};

use sdk::{
    cosmwasm_std::Timestamp,
    schemars::{self, JsonSchema},
};

pub use self::{
    open::{InterestPaymentSpec, LoanForm, NewLeaseForm},
    query::{StateQuery, StateResponse},
};

pub mod dex;
// TODO consider defining the modules public instead of just selected items
mod open;
mod query;

pub type DownpaymentCoin = CoinDTO<PaymentGroup>;
pub type LeaseCoin = CoinDTO<LeaseGroup>;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Repay(), // it is not an enum variant to represent it as a JSON object instead of JSON string
    Close(), // that is a limitation of cosmjs library
    PriceAlarm(),
    TimeAlarm(Timestamp),
}