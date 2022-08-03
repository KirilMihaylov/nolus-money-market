use finance::{
    coin::Coin,
    currency::Currency
};
use platform::batch::Batch;

pub(crate) struct Result<C>
where
    C: Currency,
{
    pub batch: Batch,
    pub receipt: Receipt<C>,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub(crate) struct Receipt<C>
where
    C: Currency,
{
    previous_margin_paid: Coin<C>,
    current_margin_paid: Coin<C>,
    previous_interest_paid: Coin<C>,
    current_interest_paid: Coin<C>,
    principal_paid: Coin<C>,
    close: bool,
}

impl<C> Receipt<C>
where
    C: Currency,
{
    pub fn previous_margin_paid(&self) -> Coin<C> {
        self.previous_margin_paid
    }

    pub fn previous_interest_paid(&self) -> Coin<C> {
        self.previous_interest_paid
    }

    pub fn current_margin_paid(&self) -> Coin<C> {
        self.current_margin_paid
    }

    pub fn current_interest_paid(&self) -> Coin<C> {
        self.current_interest_paid
    }

    pub fn principal_paid(&self) -> Coin<C> {
        self.principal_paid
    }

    pub fn close(&self) -> bool {
        self.close
    }

    pub(super) fn pay_previous_margin(&mut self, payment: Coin<C>) {
        self.previous_margin_paid = payment;
    }

    pub(super) fn pay_previous_interest(&mut self, payment: Coin<C>) {
        self.previous_interest_paid = payment;
    }

    pub(super) fn pay_current_margin(&mut self, payment: Coin<C>) {
        self.current_margin_paid = payment;
    }

    pub(super) fn pay_current_interest(&mut self, payment: Coin<C>) {
        self.current_interest_paid = payment;
    }

    pub(super) fn pay_principal(&mut self, principal: Coin<C>, payment: Coin<C>) {
        self.principal_paid = payment;

        self.close = principal == payment;
    }
}

#[cfg(test)]
mod tests {
    use finance::{
        coin::Coin,
        currency::Nls
    };

    use crate::loan::Receipt;

    #[test]
    fn pay_principal_full() {
        let principal = Coin::<Nls>::new(10);

        let mut paid = Receipt::default();

        paid.pay_principal(principal, principal);

        assert_eq!(
            paid,
            Receipt {
                principal_paid: principal,
                close: true,
                .. Default::default()
            },
        );
    }
}
