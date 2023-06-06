use serde::{de::DeserializeOwned, Serialize};

use finance::currency::{AnyVisitor, Currency, SingleVisitor};

mod currency_macro;
pub mod lease;
pub mod lpn;
pub mod native;
pub mod non_native_payment;
pub mod payment;
mod symbols_macro;

struct SingleVisitorAdapter<V>(V);

impl<V> From<V> for SingleVisitorAdapter<V> {
    fn from(any_visitor: V) -> Self {
        Self(any_visitor)
    }
}

impl<C, V> SingleVisitor<C> for SingleVisitorAdapter<V>
where
    C: 'static + Currency + Serialize + DeserializeOwned,
    V: AnyVisitor,
{
    type Output = V::Output;
    type Error = V::Error;

    fn on(self) -> Result<Self::Output, Self::Error> {
        self.0.on::<C>()
    }
}

#[cfg(test)]
mod test {
    use finance::{
        currency::{Currency, Group, Symbol},
        test::visitor::Expect,
    };

    #[track_caller]
    pub fn maybe_visit_on_ticker_impl<C, G>()
    where
        C: Currency,
        G: Group,
    {
        let v = Expect::<C>::default();
        assert_eq!(G::maybe_visit_on_ticker(C::TICKER, v), Ok(Ok(true)));
    }

    #[track_caller]
    pub fn maybe_visit_on_ticker_err<C, G>(unknown_ticker: Symbol<'_>)
    where
        C: Currency,
        G: Group,
    {
        let v = Expect::<C>::default();
        assert_eq!(G::maybe_visit_on_ticker(unknown_ticker, v.clone()), Err(v));
    }

    #[track_caller]
    pub fn maybe_visit_on_bank_symbol_impl<C, G>()
    where
        C: Currency,
        G: Group,
    {
        let v = Expect::<C>::default();
        assert_eq!(
            G::maybe_visit_on_bank_symbol(C::BANK_SYMBOL, v),
            Ok(Ok(true))
        );
    }

    #[track_caller]
    pub fn maybe_visit_on_bank_symbol_err<C, G>(unknown_ticker: Symbol<'_>)
    where
        C: Currency,
        G: Group,
    {
        let v = Expect::<C>::default();
        assert_eq!(
            G::maybe_visit_on_bank_symbol(unknown_ticker, v.clone()),
            Err(v)
        );
    }
}
