use sdk::schemars;

use crate::{define_currency, define_symbol};

define_symbol! {
    USDC_NOBLE {
        ["net_dev"]: {
            // full ibc route: transfer/channel-0/transfer/channel-???/uusdc
            bank: "ibc/NA_USDC_NOBLE",
            // full ibc route: transfer/channel-???/uusdc
            dex: "ibc/NA_USDC_NOBLE_DEX",
        },
        ["net_test"]: {
            // full ibc route: transfer/channel-0/transfer/channel-4280/uusdc
            bank: "ibc/83C68CA1189A7DAC4FDA8B89F9166FFA6BA3A8C5534B0E3D84D831B4F350FE59",
            // full ibc route: transfer/channel-4280/uusdc
            dex: "ibc/DE6792CF9E521F6AD6E9A4BDF6225C9571A3B74ACC0A529F92BC5122A39D2E58",
        },
        ["net_main"]: {
            // full ibc route: transfer/channel-0/transfer/channel-750/uusdc
            bank: "ibc/F5FABF52B54E65064B57BF6DBD8E5FAD22CEE9F4B8A57ADBB20CCD0173AA72A4",
            // full ibc route: transfer/channel-750/uusdc
            dex: "ibc/498A0751C798A0D9A389AA3691123DADA57DAA4FE165D5C75894505B876BA6E4",
        },
    }
}
define_currency!(UsdcNoble, USDC_NOBLE, 6);
