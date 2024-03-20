use currency::Currency;
use finance::coin::Coin;
use platform::batch::Batch;

use crate::{api::ExecuteMsg, error::Error};

use super::Ref;

pub trait Reserve<Lpn>
where
    Self: TryInto<Batch, Error = Error>,
{
    fn cover_liquidation_losses(&mut self, amount: Coin<Lpn>);
}

pub(crate) struct Impl<Lpn> {
    ref_: Ref<Lpn>,
    amount: Option<Coin<Lpn>>,
}

impl<Lpn> Impl<Lpn> {
    pub fn new(ref_: Ref<Lpn>) -> Self {
        Self { ref_, amount: None }
    }
}

impl<Lpn> Reserve<Lpn> for Impl<Lpn>
where
    Lpn: Currency,
{
    fn cover_liquidation_losses(&mut self, amount: Coin<Lpn>) {
        debug_assert!(self.amount.is_none());
        self.amount = Some(amount);
    }
}

impl<Lpn> TryFrom<Impl<Lpn>> for Batch
where
    Lpn: Currency,
{
    type Error = Error;

    fn try_from(stub: Impl<Lpn>) -> Result<Self, Self::Error> {
        let mut batch = Batch::default();
        if let Some(losses) = stub.amount {
            batch
                .schedule_execute_wasm_no_reply(
                    stub.ref_.into(),
                    &ExecuteMsg::CoverLiquidationLosses(losses.into()),
                    Some(losses),
                )
                .map_err(Into::into)
                .map(|()| batch)
        } else {
            Ok(batch)
        }
    }
}
