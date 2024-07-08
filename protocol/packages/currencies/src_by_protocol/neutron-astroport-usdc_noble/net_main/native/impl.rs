use sdk::schemars;

use crate::{define_currency, define_symbol};

define_symbol! {
    NLS {
        bank: "unls",
        // full ibc route: transfer/channel-44/unls
        dex: "ibc/6C9E6701AC217C0FC7D74B0F7A6265B9B4E3C3CDA6E80AADE5F950A8F52F9972"
    }
}
define_currency!(Nls, NLS, 6);
