use currency::{
    AnyVisitor, Group, InPoolWith, Matcher, MaybeAnyVisitResult, MaybePairsVisitorResult, MemberOf,
    PairsGroup, PairsVisitor,
};
use sdk::schemars;

use crate::{define_currency, LeaseGroup, Lpn, PaymentGroup};

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
    Cro,
    "CRO",
    "ibc/E1BCC0F7B932E654B1A930F72B76C0678D55095387E2A4D8F00E941A8F82EE48", // transfer/channel-0/transfer/channel-5/basecro
    "ibc/E6931F78057F7CC5DA0FD6CEF82FF39373A6E0452BF1FD76910B93292CF356C1", // transfer/channel-5/basecro
    LeaseGroup,
    8
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
    StTia,
    "ST_TIA",
    "ibc/8D4FC51F696E03711B9B37A5787FB89BD2DDBAF788813478B002D552A12F9157", // transfer/channel-0/transfer/channel-326/stutia
    "ibc/698350B8A61D575025F3ED13E9AC9C0F45C89DEFE92F76D5838F1D3C1A7FF7C9", // transfer/channel-326/stutia
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
    AllBtc,
    "ALL_BTC",
    "ibc/E45CFCB959F4F6D1065B7033EE49A88E606E6AD82E75725219B3D68B0FA89987", // transfer/channel-0/allBTC
    "factory/osmo1z6r6qdknhgsc0zeracktgpcxf43j6sekq07nw8sxduc9lg0qjjlqfu25e3/alloyed/allBTC",
    LeaseGroup,
    8
);

define_currency!(
    AllSol,
    "ALL_SOL",
    "ibc/762E1E45658845A12E214A91C3C05FDFC5951D60404FAADA225A369A96DCD9A9", // transfer/channel-0/allSOL
    "factory/osmo1n3n75av8awcnw4jl62n3l48e6e4sxqmaf97w5ua6ddu4s475q5qq9udvx4/alloyed/allSOL",
    LeaseGroup,
    9
);

define_currency!(
    AllEth,
    "ALL_ETH",
    "ibc/7879B1CBBD2E07347002334792368E65C11A7D1629297D04B6A2155F557E02D4", // transfer/channel-0/allETH
    "factory/osmo1k6c8jln7ejuqwtqmay3yvzrg3kueaczl96pk067ldg8u835w0yhsw27twm/alloyed/allETH",
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
        .or_else(|visitor| maybe_visit::<_, Inj, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Scrt, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Cro, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Tia, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, StTia, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, Jkl, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, MilkTia, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, AllBtc, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, AllSol, VisitedG, _>(matcher, visitor))
        .or_else(|visitor| maybe_visit::<_, AllEth, VisitedG, _>(matcher, visitor))
}

impl PairsGroup for Atom {
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
impl InPoolWith<StAtom> for Atom {}
impl InPoolWith<Osmo> for Atom {}

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
        maybe_visit::<Atom, _, _>(matcher, visitor)
    }
}
impl InPoolWith<Weth> for Osmo {}
impl InPoolWith<Wbtc> for Osmo {}
impl InPoolWith<StOsmo> for Osmo {}
impl InPoolWith<Akt> for Osmo {}
impl InPoolWith<Axl> for Osmo {}
impl InPoolWith<Scrt> for Osmo {}
impl InPoolWith<Cro> for Osmo {}
impl InPoolWith<Tia> for Osmo {}
impl InPoolWith<Jkl> for Osmo {}

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

impl PairsGroup for Inj {
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
impl InPoolWith<StTia> for Tia {}
impl InPoolWith<MilkTia> for Tia {}

impl PairsGroup for StTia {
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

impl PairsGroup for AllBtc {
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
impl InPoolWith<AllSol> for AllBtc {}

impl PairsGroup for AllSol {
    type CommonGroup = PaymentGroup;

    fn maybe_visit<M, V>(matcher: &M, visitor: V) -> MaybePairsVisitorResult<V>
    where
        M: Matcher,
        V: PairsVisitor<Pivot = Self>,
    {
        use currency::maybe_visit_buddy as maybe_visit;
        maybe_visit::<AllBtc, _, _>(matcher, visitor)
    }
}

impl PairsGroup for AllEth {
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

    use super::{Atom, Osmo, StAtom, StOsmo, StTia, Tia, Wbtc, Weth};

    #[test]
    fn maybe_visit_on_ticker() {
        maybe_visit_on_ticker_impl::<Atom, LeaseGroup>();
        maybe_visit_on_ticker_impl::<StAtom, LeaseGroup>();
        maybe_visit_on_ticker_impl::<Osmo, LeaseGroup>();
        maybe_visit_on_ticker_impl::<StOsmo, LeaseGroup>();
        maybe_visit_on_ticker_impl::<Weth, LeaseGroup>();
        maybe_visit_on_ticker_impl::<Wbtc, LeaseGroup>();
        maybe_visit_on_ticker_impl::<Tia, LeaseGroup>();
        maybe_visit_on_ticker_impl::<StTia, LeaseGroup>();

        maybe_visit_on_ticker_err::<Lpn, Lpns>(Lpn::bank());
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
        maybe_visit_on_bank_symbol_err::<Lpn, Lpns>(Lpn::ticker());
        maybe_visit_on_bank_symbol_err::<Atom, LeaseGroup>(Atom::ticker());
        maybe_visit_on_bank_symbol_err::<Atom, LeaseGroup>(Lpn::ticker());
        maybe_visit_on_bank_symbol_err::<Atom, LeaseGroup>(Nls::bank());
        maybe_visit_on_bank_symbol_err::<Atom, LeaseGroup>(Nls::ticker());
    }
}
