use cosmwasm_std::{Addr, BankMsg, Coin as CwCoin, Env, QuerierWrapper};
use std::result::Result as StdResult;

use finance::{
    coin::Coin,
    currency::{Currency, Group},
};

use crate::{
    batch::Batch,
    coin_legacy::{from_cosmwasm_any_impl, from_cosmwasm_impl, to_cosmwasm_impl, CoinVisitor},
    error::{Error, Result},
};

pub trait BankAccountView {
    fn balance<C>(&self) -> Result<Coin<C>>
    where
        C: Currency;
}

pub trait BankAccount
where
    Self: BankAccountView + Into<Batch>,
{
    fn send<C>(&mut self, amount: Coin<C>, to: &Addr)
    where
        C: Currency;
}

pub trait FixedAddressSenderBuilder {
    type Built: FixedAddressSender;

    fn build(self, address: Addr) -> Self::Built;
}

pub trait FixedAddressSender
where
    Self: Into<Batch>,
{
    fn send<C>(&mut self, amount: Coin<C>)
    where
        C: Currency;
}

pub fn received<C>(cw_amount: Vec<CwCoin>) -> Result<Coin<C>>
where
    C: Currency,
{
    received_one(
        cw_amount,
        Error::no_funds::<C>,
        Error::unexpected_funds::<C>,
    )
    .and_then(from_cosmwasm_impl)
}

pub fn received_any<G, V>(cw_amount: Vec<CwCoin>, visitor: V) -> StdResult<V::Output, V::Error>
where
    V: CoinVisitor,
    G: Group,
    Error: Into<V::Error>,
    G::ResolveError: Into<V::Error>,
{
    received_one(cw_amount, Error::NoFundsAny, Error::UnexpectedFundsAny)
        .map_err(Into::into)
        .and_then(|coin| from_cosmwasm_any_impl::<G, _>(coin, visitor))
}

pub struct BankView<'a> {
    addr: &'a Addr,
    querier: &'a QuerierWrapper<'a>,
}

impl<'a> BankView<'a> {
    pub fn my_account(env: &'a Env, querier: &'a QuerierWrapper) -> Self {
        Self {
            addr: &env.contract.address,
            querier,
        }
    }
}

impl<'a> BankAccountView for BankView<'a> {
    fn balance<C>(&self) -> Result<Coin<C>>
    where
        C: Currency,
    {
        let coin = self.querier.query_balance(self.addr, C::SYMBOL)?;
        from_cosmwasm_impl(coin)
    }
}

pub struct BankStub<'a> {
    view: BankView<'a>,
    batch: Batch,
}

impl<'a> BankStub<'a> {
    pub fn my_account(env: &'a Env, querier: &'a QuerierWrapper) -> Self {
        Self {
            view: BankView::my_account(env, querier),
            batch: Batch::default(),
        }
    }
}

impl<'a> BankAccountView for BankStub<'a> {
    fn balance<C>(&self) -> Result<Coin<C>>
    where
        C: Currency,
    {
        self.view.balance()
    }
}

impl<'a> BankAccount for BankStub<'a>
where
    Self: BankAccountView + Into<Batch>,
{
    fn send<C>(&mut self, amount: Coin<C>, to: &Addr)
    where
        C: Currency,
    {
        debug_assert!(!amount.is_zero());
        self.batch.schedule_execute_no_reply(BankMsg::Send {
            to_address: to.into(),
            amount: vec![to_cosmwasm_impl(amount)],
        });
    }
}

impl<'a> From<BankStub<'a>> for Batch {
    fn from(stub: BankStub) -> Self {
        stub.batch
    }
}

fn received_one<NoFundsErr, UnexpFundsErr>(
    cw_amount: Vec<CwCoin>,
    no_funds_err: NoFundsErr,
    unexp_funds_err: UnexpFundsErr,
) -> Result<CwCoin>
where
    NoFundsErr: FnOnce() -> Error,
    UnexpFundsErr: FnOnce() -> Error,
{
    match cw_amount.len() {
        0 => Err(no_funds_err()),
        1 => {
            let first = cw_amount
                .into_iter()
                .next()
                .expect("there is at least a coin");
            Ok(first)
        }
        _ => Err(unexp_funds_err()),
    }
}

#[derive(Default)]
pub struct LazySenderStubBuilder;

impl FixedAddressSenderBuilder for LazySenderStubBuilder {
    type Built = LazySenderStub;

    fn build(self, address: Addr) -> Self::Built {
        LazySenderStub {
            address,
            amounts: Vec::new(),
        }
    }
}

pub struct LazySenderStub {
    address: Addr,
    amounts: Vec<CwCoin>,
}

impl FixedAddressSender for LazySenderStub
where
    Self: Into<Batch>,
{
    fn send<C>(&mut self, amount: Coin<C>)
    where
        C: Currency,
    {
        debug_assert!(!amount.is_zero());

        if amount.is_zero() {
            return;
        }

        self.amounts.push(to_cosmwasm_impl(amount));
    }
}

impl From<LazySenderStub> for Batch {
    fn from(stub: LazySenderStub) -> Self {
        let mut batch = Batch::default();

        if !stub.amounts.is_empty() {
            batch.schedule_execute_no_reply(BankMsg::Send {
                to_address: stub.address.to_string(),
                amount: stub.amounts,
            });
        }

        batch
    }
}
