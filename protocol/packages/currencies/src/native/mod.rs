use currency::{AnyVisitor, Group, Matcher, MaybeAnyVisitResult, SymbolSlice};

#[cfg(dex = "astroport")]
pub(crate) mod astroport;
#[cfg(dex = "astroport")]
pub type Nls = astroport::Nls;

#[cfg(all(not(dex = "astroport"), dex = "osmosis"))]
pub(crate) mod osmosis;
#[cfg(all(not(dex = "astroport"), dex = "osmosis"))]
pub type Nls = osmosis::Nls;

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "testing"), derive(Debug))]
pub struct Native {}
impl Group for Native {
    const DESCR: &'static str = "native";

    fn maybe_visit<M, V>(matcher: &M, symbol: &SymbolSlice, visitor: V) -> MaybeAnyVisitResult<V>
    where
        M: Matcher + ?Sized,
        V: AnyVisitor,
    {
        currency::maybe_visit_any::<_, Nls, _>(matcher, symbol, visitor)
    }
}