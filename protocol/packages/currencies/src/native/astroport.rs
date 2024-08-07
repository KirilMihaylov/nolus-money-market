use sdk::schemars;

use crate::{define_currency, define_symbol};

define_symbol! {
    NLS {
        ["net_dev"]: {
            bank: "unls",
            // full ibc route: transfer/channel-585/unls
            dex: "ibc/0D9EB2C9961610CD2F04003188C0B713E72297DCBC32371602897069DC0E3055"
        },
        ["net_test"]: {
            bank: "unls",
            // full ibc route: transfer/channel-1061/unls
            dex: "ibc/E808FAAE7ADDA37453A8F0F67D74669F6580CBA5EF0F7889D46FB02D282098E3"
        },
        ["net_main"]: {
            bank: "unls",
            // full ibc route: transfer/channel-44/unls
            dex: "ibc/6C9E6701AC217C0FC7D74B0F7A6265B9B4E3C3CDA6E80AADE5F950A8F52F9972"
        },
    }
}
define_currency!(Nls, NLS, 6);
