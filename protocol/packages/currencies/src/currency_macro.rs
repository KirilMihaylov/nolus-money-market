#[macro_export]
macro_rules! define_currency {
    (
        $ident:ident,
        $ticker:path $(,)?
    ) => {
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Default,
            ::serde::Serialize,
            ::serde::Deserialize,
            ::sdk::schemars::JsonSchema,
        )]
        #[serde(deny_unknown_fields, rename_all = "snake_case")]
        pub struct $ident {}

        impl ::currency::Currency for $ident {
            const TICKER: ::currency::SymbolStatic = ::core::stringify!($ticker);

            const BANK_SYMBOL: ::currency::SymbolStatic = $ticker.bank;

            const DEX_SYMBOL: ::currency::SymbolStatic = $ticker.dex;
        }
    };
}
