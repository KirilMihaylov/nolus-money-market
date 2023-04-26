use serde::{Deserialize, Serialize};

use finance::currency::{self, AnyVisitor, Group, MaybeAnyVisitResult, Symbol, SymbolStatic};
use sdk::schemars::{self, JsonSchema};

use crate::{define_currency, define_symbol, SingleVisitorAdapter};

define_symbol! {
    USDC {
        {
            /// full ibc route: transfer/channel-0/transfer/channel-208/uusdc
            bank: "ibc/7FBDBEEEBA9C50C4BCDF7BF438EAB99E64360833D240B32655C96E319559E911",
            /// full ibc route: transfer/channel-208/uusdc
            dex: "ibc/D189335C6E4A68B513C10AB227BF1C1D38C746766278BA3EEB4FB14124F1D858",
        },
        alt: {
            /// full ibc route: transfer/channel-0/transfer/channel-1946/uausdc
            bank: "ibc/B37EE251476F383954F68EE99C2BFE1E78D45EE4AA792952A8C3545E3543A89F",
            /// full ibc route: transfer/channel-1946/uausdc
            dex: "ibc/FF3065989E34457F342D4EFB8692406D49D4E2B5C70F725F127862E22CE6BDCD",
        },
    }
}
define_currency!(Usdc, USDC);

#[derive(Clone, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[cfg_attr(any(test, feature = "testing"), derive(Debug))]
pub struct Lpns {}

impl Group for Lpns {
    const DESCR: SymbolStatic = "lpns";

    fn maybe_visit_on_ticker<V>(ticker: Symbol<'_>, visitor: V) -> MaybeAnyVisitResult<V>
    where
        V: AnyVisitor,
    {
        let v: SingleVisitorAdapter<_> = visitor.into();
        currency::maybe_visit_on_ticker::<Usdc, _>(ticker, v).map_err(|v| v.0)
    }

    fn maybe_visit_on_bank_symbol<V>(bank_symbol: Symbol<'_>, visitor: V) -> MaybeAnyVisitResult<V>
    where
        Self: Sized,
        V: AnyVisitor,
    {
        let v: SingleVisitorAdapter<_> = visitor.into();
        currency::maybe_visit_on_bank_symbol::<Usdc, _>(bank_symbol, v).map_err(|v| v.0)
    }
}

#[cfg(test)]
mod test {
    use finance::currency::Currency;

    use crate::{
        lease::Osmo,
        native::Nls,
        test::{
            maybe_visit_on_bank_symbol_err, maybe_visit_on_bank_symbol_impl,
            maybe_visit_on_ticker_err, maybe_visit_on_ticker_impl,
        },
    };

    use super::{Lpns, Usdc};

    #[test]
    fn maybe_visit_on_ticker() {
        maybe_visit_on_ticker_impl::<Usdc, Lpns>();
        maybe_visit_on_ticker_err::<Usdc, Lpns>(Usdc::BANK_SYMBOL);
        maybe_visit_on_ticker_err::<Usdc, Lpns>(Nls::TICKER);
        maybe_visit_on_ticker_err::<Usdc, Lpns>(Osmo::TICKER);
    }

    #[test]
    fn maybe_visit_on_bank_symbol() {
        maybe_visit_on_bank_symbol_impl::<Usdc, Lpns>();
        maybe_visit_on_bank_symbol_err::<Usdc, Lpns>(Usdc::TICKER);
        maybe_visit_on_bank_symbol_err::<Usdc, Lpns>(Nls::BANK_SYMBOL);
        maybe_visit_on_bank_symbol_err::<Usdc, Lpns>(Osmo::BANK_SYMBOL);
    }
}
