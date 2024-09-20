// brings `LeaseC*` types
pub use crate::lease::impl_mod::definitions::*;
use crate::{lpn::Lpn, native::Nls};

pub type PaymentC1 = Nls;
pub type PaymentC2 = Lpn;
pub type PaymentC3 = LeaseC1;
pub type PaymentC4 = LeaseC2;
pub type PaymentC5 = LeaseC3;
pub type PaymentC6 = LeaseC4;
pub type PaymentC7 = LeaseC5;
pub type PaymentC8 = LeaseC6;
pub type PaymentC9 = LeaseC7;
