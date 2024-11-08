use currency::{
    AnyVisitor, Group, InPoolWith, Matcher, MaybeAnyVisitResult, MaybePairsVisitorResult, MemberOf,
    PairsGroup, PairsVisitor,
};
use sdk::schemars;

use crate::{define_currency, payment::only::impl_mod::UsdcNoble, LeaseGroup, Lpn, PaymentGroup};

// Resources:
// 1. Symbol hashes are computed using the SHA256 Hash Generator https://coding.tools/sha256
// 2. Currencies that come from Axelar are documented at https://docs.axelar.dev/resources
// 3. IBC routes from https://github.com/Nolus-Protocol/Wiki/blob/main/testnet-rila/currencies.json

define_currency!(
    Atom,
    "ATOM",
    "ibc/6CDD4663F2F09CD62285E2D45891FC149A3568E316CE3EBBE201A71A78A69388", // transfer/channel-0/transfer/channel-0/uatom
    "ibc/27394FB092D2ECCD56123C74F36E4C1F926001CEADA9CA97EA622B25F41E5EB2", // transfer/channel-0/uatom
    LeaseGroup,
    6
);

define_currency!(
    StAtom,
    "ST_ATOM",
    "ibc/FCFF8B19C61677F3B78E2A5AE3B4A34A8D23858D16905F253B8438B3AFD07FF8", // transfer/channel-0/transfer/channel-326/stuatom
    "ibc/C140AFD542AE77BD7DCC83F13FDD8C5E5BB8C4929785E6EC2F4C636F98F17901", // transfer/channel-326/stuatom
    LeaseGroup,
    6
);

define_currency!(
    Osmo,
    "OSMO",
    "ibc/ED07A3391A112B175915CD8FAF43A2DA8E4790EDE12566649D0C2F97716B8518", // transfer/channel-0/uosmo
    "uosmo",
    LeaseGroup,
    6
);

define_currency!(
    StOsmo,
    "ST_OSMO",
    "ibc/AF5559D128329B6C753F15481BEC26E533B847A471074703FA4903E7E6F61BA1", // transfer/channel-0/transfer/channel-326/stuosmo
    "ibc/D176154B0C63D1F9C6DCFB4F70349EBF2E2B5A87A05902F57A6AE92B863E9AEC", // transfer/channel-326/stuosmo
    LeaseGroup,
    6
);

define_currency!(
    Weth,
    "WETH",
    "ibc/A7C4A3FB19E88ABE60416125F9189DA680800F4CDD14E3C10C874E022BEFF04C", // transfer/channel-0/transfer/channel-208/weth-wei
    "ibc/EA1D43981D5C9A1C4AAEA9C23BB1D4FA126BA9BC7020A25E0AE4AA841EA25DC5", // transfer/channel-208/weth-wei
    LeaseGroup,
    18
);

define_currency!(
    Wbtc,
    "WBTC",
    "ibc/84E70F4A34FB2DE135FD3A04FDDF53B7DA4206080AA785C8BAB7F8B26299A221", // transfer/channel-0/transfer/channel-208/wbtc-satoshi
    "ibc/D1542AA8762DB13087D8364F3EA6509FD6F009A34F00426AF9E4F9FA85CBBF1F", // transfer/channel-208/wbtc-satoshi
    LeaseGroup,
    8
);

define_currency!(
    Akt,
    "AKT",
    "ibc/ADC63C00000CA75F909D2BE3ACB5A9980BED3A73B92746E0FCE6C67414055459", // transfer/channel-0/transfer/channel-1/uakt
    "ibc/1480B8FD20AD5FCAE81EA87584D269547DD4D436843C1D20F15E00EB64743EF4", // transfer/channel-1/uakt
    LeaseGroup,
    6
);

define_currency!(
    Axl,
    "AXL",
    "ibc/1B03A71B8E6F6EF424411DC9326A8E0D25D096E4D2616425CFAF2AF06F0FE717", // transfer/channel-0/transfer/channel-208/uaxl
    "ibc/903A61A498756EA560B85A85132D3AEE21B5DEDD41213725D22ABF276EA6945E", // transfer/channel-208/uaxl
    LeaseGroup,
    6
);

define_currency!(
    QAtom,
    "Q_ATOM",
    "ibc/317FCA2D7554F55BBCD0019AB36F7FEA18B6D161F462AF5E565068C719A29F20", // transfer/channel-0/transfer/channel-522/uqatom
    "ibc/FA602364BEC305A696CBDF987058E99D8B479F0318E47314C49173E8838C5BAC", // transfer/channel-522/uqatom
    LeaseGroup,
    6
);

define_currency!(
    Inj,
    "INJ",
    "ibc/4DE84C92C714009D07AFEA7350AB3EC383536BB0FAAD7AF9C0F1A0BEA169304E", // transfer/channel-0/transfer/channel-122/inj
    "ibc/64BA6E31FE887D66C6F8F31C7B1A80C7CA179239677B4088BB55F5EA07DBE273", // transfer/channel-122/inj
    LeaseGroup,
    18
);

define_currency!(
    Scrt,
    "SCRT",
    "ibc/EA00FFF0335B07B5CD1530B7EB3D2C710620AE5B168C71AFF7B50532D690E107", // transfer/channel-0/transfer/channel-88/uscrt
    "ibc/0954E1C28EB7AF5B72D24F3BC2B47BBB2FDF91BDDFD57B74B99E133AED40972A", // transfer/channel-88/uscrt
    LeaseGroup,
    6
);

define_currency!(
    Stars,
    "STARS",
    "ibc/11E3CF372E065ACB1A39C531A3C7E7E03F60B5D0653AD2139D31128ACD2772B5", // transfer/channel-0/transfer/channel-75/ustars
    "ibc/987C17B11ABC2B20019178ACE62929FE9840202CE79498E29FE8E5CB02B7C0A4", // transfer/channel-75/ustars
    LeaseGroup,
    6
);

define_currency!(
    Cro,
    "CRO",
    "ibc/E1BCC0F7B932E654B1A930F72B76C0678D55095387E2A4D8F00E941A8F82EE48", // transfer/channel-0/transfer/channel-5/basecro
    "ibc/E6931F78057F7CC5DA0FD6CEF82FF39373A6E0452BF1FD76910B93292CF356C1", // transfer/channel-5/basecro
    LeaseGroup,
    8
);

define_currency!(
    Juno,
    "JUNO",
    "ibc/4F3E83AB35529435E4BFEA001F5D935E7250133347C4E1010A9C77149EF0394C", // transfer/channel-0/transfer/channel-42/ujuno
    "ibc/46B44899322F3CD854D2D46DEEF881958467CDD4B3B10086DA49296BBED94BED", // transfer/channel-42/ujuno
    LeaseGroup,
    6
);

define_currency!(
    Tia,
    "TIA",
    "ibc/6C349F0EB135C5FA99301758F35B87DB88403D690E5E314AB080401FEE4066E5", // transfer/channel-0/transfer/channel-6994/utia
    "ibc/D79E7D83AB399BFFF93433E54FAA480C191248FC556924A2A8351AE2638B3877", // transfer/channel-6994/utia
    LeaseGroup,
    6
);

define_currency!(
    Jkl,
    "JKL",
    "ibc/28F026607184B151F1F7D7F5D8AE644528550EB05203A28B6233DFA923669876", // transfer/channel-0/transfer/channel-412/ujkl
    "ibc/8E697BDABE97ACE8773C6DF7402B2D1D5104DD1EEABE12608E3469B7F64C15BA", // transfer/channel-412/ujkl
    LeaseGroup,
    6
);

define_currency!(
    MilkTia,
    "MILK_TIA",
    "ibc/16065EE5282C5217685C8F084FC44864C25C706AC37356B0D62811D50B96920F", // transfer/channel-0/factory/osmo1f5vfcph2dvfeqcqkhetwv75fda69z7e5c2dldm3kvgj23crkv6wqcn47a0/umilkTIA
    "factory/osmo1f5vfcph2dvfeqcqkhetwv75fda69z7e5c2dldm3kvgj23crkv6wqcn47a0/umilkTIA",
    LeaseGroup,
    6
);

define_currency!(
    Pica,
    "PICA",
    "ibc/7F2DC2A595EDCAEC1C03D607C6DC3C79EDDC029A53D16C0788835C1A9AA06306", // transfer/channel-0/transfer/channel-1279/ppica
    "ibc/56D7C03B8F6A07AD322EEE1BEF3AE996E09D1C1E34C27CF37E0D4A0AC5972516", // transfer/channel-1279/ppica
    LeaseGroup,
    12
);

define_currency!(
    Dym,
    "DYM",
    "ibc/9C7F70E92CCBA0F2DC94796B0682955E090676EA7A2F8E0A4611956B79CB4406", // transfer/channel-0/transfer/channel-19774/adym
    "ibc/9A76CDF0CBCEF37923F32518FA15E5DC92B9F56128292BC4D63C4AEA76CBB110", // transfer/channel-19774/adym
    LeaseGroup,
    18
);

pub(super) fn maybe_visit<M, V, VisitedG>(
    matcher: &M,
    visitor: V,
) -> MaybeAnyVisitResult<VisitedG, V>
where
    M: Matcher,
    V: AnyVisitor<VisitedG>,
    LeaseGroup: MemberOf<VisitedG>,
    VisitedG: Group<TopG = PaymentGroup>,
{
    use currency::maybe_visit_member as maybe_visit;
    maybe_visit::<_, Atom, VisitedG, _>(matcher, visitor)
        .or_else(|visitor| maybe_visit::<_, StAtom, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Osmo, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, StOsmo, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Weth, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Wbtc, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Akt, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Axl, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, QAtom, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Inj, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Scrt, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Stars, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Cro, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Juno, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Tia, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Jkl, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, MilkTia, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Pica, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Dym, VisitedG, _>(matcher, visitor))
}

impl PairsGroup for Atom {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Osmo, _, _>(matcher, visitor)
    }
}
impl InPoolWith<StAtom> for Atom {}
impl InPoolWith<QAtom> for Atom {}

impl PairsGroup for StAtom {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Atom, _, _>(matcher, visitor)
    }
}

impl PairsGroup for Osmo {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Lpn, _, _>(matcher, visitor)
    }
}
impl InPoolWith<Atom> for Osmo {}
impl InPoolWith<Weth> for Osmo {}
impl InPoolWith<Wbtc> for Osmo {}
impl InPoolWith<StOsmo> for Osmo {}
impl InPoolWith<Akt> for Osmo {}
impl InPoolWith<Axl> for Osmo {}
impl InPoolWith<Scrt> for Osmo {}
impl InPoolWith<Stars> for Osmo {}
impl InPoolWith<Cro> for Osmo {}
impl InPoolWith<Juno> for Osmo {}
impl InPoolWith<Tia> for Osmo {}
impl InPoolWith<Jkl> for Osmo {}
impl InPoolWith<Pica> for Osmo {}
impl InPoolWith<Dym> for Osmo {}

impl PairsGroup for StOsmo {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Osmo, _, _>(matcher, visitor)
    }
}

impl PairsGroup for Weth {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Osmo, _, _>(matcher, visitor)
    }
}

impl PairsGroup for Wbtc {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Osmo, _, _>(matcher, visitor)
    }
}

impl PairsGroup for Akt {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Osmo, _, _>(matcher, visitor)
    }
}

impl PairsGroup for Axl {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Osmo, _, _>(matcher, visitor)
    }
}

impl PairsGroup for QAtom {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Atom, _, _>(matcher, visitor)
    }
}

impl PairsGroup for Inj {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<UsdcNoble, _, _>(matcher, visitor)
    }
}

impl PairsGroup for Scrt {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Osmo, _, _>(matcher, visitor)
    }
}

impl PairsGroup for Stars {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Osmo, _, _>(matcher, visitor)
    }
}

impl PairsGroup for Cro {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Osmo, _, _>(matcher, visitor)
    }
}

impl PairsGroup for Juno {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Osmo, _, _>(matcher, visitor)
    }
}

impl PairsGroup for Tia {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Osmo, _, _>(matcher, visitor)
    }
}
impl InPoolWith<MilkTia> for Tia {}

impl PairsGroup for MilkTia {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Tia, _, _>(matcher, visitor)
    }
}

impl PairsGroup for Jkl {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Osmo, _, _>(matcher, visitor)
    }
}

impl PairsGroup for Pica {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Osmo, _, _>(matcher, visitor)
    }
}

impl PairsGroup for Dym {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<Osmo, _, _>(matcher, visitor)
    }
}

#[cfg(test)]
mod test {
    use currency::CurrencyDef as _;

    use crate::{
        test_impl::{
            maybe_visit_on_bank_symbol_err, maybe_visit_on_bank_symbol_impl,
            maybe_visit_on_ticker_err, maybe_visit_on_ticker_impl,
        },
        {
            lease::LeaseGroup,
            lpn::{Lpn, Lpns},
            native::Nls,
        },
    };

    use super::{Atom, Dym, Osmo, Pica, StAtom, StOsmo, Tia, Wbtc, Weth};

    #[test]
    fn maybe_visit_on_ticker() {
        maybe_visit_on_ticker_impl::<Atom, LeaseGroup>();
        maybe_visit_on_ticker_impl::<StAtom, LeaseGroup>();
        maybe_visit_on_ticker_impl::<Osmo, LeaseGroup>();
        maybe_visit_on_ticker_impl::<StOsmo, LeaseGroup>();
        maybe_visit_on_ticker_impl::<Weth, LeaseGroup>();
        maybe_visit_on_ticker_impl::<Wbtc, LeaseGroup>();
        maybe_visit_on_ticker_impl::<Tia, LeaseGroup>();
        maybe_visit_on_ticker_impl::<Dym, LeaseGroup>();
        maybe_visit_on_ticker_err::<Lpn, Lpns>(Lpn::dex());
        maybe_visit_on_ticker_err::<Atom, LeaseGroup>(Atom::bank());
        maybe_visit_on_ticker_err::<Atom, LeaseGroup>(Nls::ticker());
        maybe_visit_on_ticker_err::<Atom, LeaseGroup>(Nls::bank());
        maybe_visit_on_ticker_err::<Atom, LeaseGroup>(Lpn::bank());
    }

    #[test]
    fn maybe_visit_on_bank_symbol() {
        maybe_visit_on_bank_symbol_impl::<Atom, LeaseGroup>();
        maybe_visit_on_bank_symbol_impl::<StAtom, LeaseGroup>();
        maybe_visit_on_bank_symbol_impl::<Osmo, LeaseGroup>();
        maybe_visit_on_bank_symbol_impl::<StOsmo, LeaseGroup>();
        maybe_visit_on_bank_symbol_impl::<Weth, LeaseGroup>();
        maybe_visit_on_bank_symbol_impl::<Wbtc, LeaseGroup>();
        maybe_visit_on_bank_symbol_impl::<Tia, LeaseGroup>();
        maybe_visit_on_bank_symbol_impl::<Pica, LeaseGroup>();
        maybe_visit_on_bank_symbol_impl::<Dym, LeaseGroup>();
        maybe_visit_on_bank_symbol_err::<Lpn, Lpns>(Lpn::ticker());
        maybe_visit_on_bank_symbol_err::<Atom, LeaseGroup>(Atom::ticker());
        maybe_visit_on_bank_symbol_err::<Atom, LeaseGroup>(Lpn::ticker());
        maybe_visit_on_bank_symbol_err::<Atom, LeaseGroup>(Nls::bank());
        maybe_visit_on_bank_symbol_err::<Atom, LeaseGroup>(Nls::ticker());
    }
}
