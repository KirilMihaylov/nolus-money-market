use std::marker::PhantomData;

use finance::{coin::Coin, currency::Currency, price};
use sdk::cosmwasm_std::QuerierWrapper;

use crate::{
    stub::{Oracle, OracleRef, WithOracle},
    ContractError,
};

pub fn to_base<BaseC, InC>(
    oracle_ref: OracleRef,
    in_amount: Coin<InC>,
    querier: &QuerierWrapper<'_>,
) -> Result<Coin<BaseC>, ContractError>
where
    BaseC: Currency,
    InC: Currency,
{
    struct PriceConvert<BaseC, In>
    where
        BaseC: Currency,
        In: Currency,
    {
        in_amount: Coin<In>,
        _out: PhantomData<BaseC>,
    }

    impl<BaseC, In> WithOracle<BaseC> for PriceConvert<BaseC, In>
    where
        BaseC: Currency,
        In: Currency,
    {
        type Output = Coin<BaseC>;
        type Error = ContractError;

        fn exec<OracleImpl>(self, oracle: OracleImpl) -> Result<Self::Output, Self::Error>
        where
            OracleImpl: Oracle<BaseC>,
        {
            Ok(price::total(self.in_amount, oracle.price_of()?))
        }
    }

    oracle_ref.execute_as_oracle(
        PriceConvert {
            in_amount,
            _out: PhantomData,
        },
        querier,
    )
}

pub fn from_base<BaseC, OutC>(
    oracle_ref: OracleRef,
    in_amount: Coin<BaseC>,
    querier: &QuerierWrapper<'_>,
) -> Result<Coin<OutC>, ContractError>
where
    BaseC: Currency,
    OutC: Currency,
{
    struct PriceConvert<BaseC, Out>
    where
        BaseC: Currency,
        Out: Currency,
    {
        in_amount: Coin<BaseC>,
        _out: PhantomData<Out>,
    }

    impl<BaseC, Out> WithOracle<BaseC> for PriceConvert<BaseC, Out>
    where
        BaseC: Currency,
        Out: Currency,
    {
        type Output = Coin<Out>;
        type Error = ContractError;

        fn exec<OracleImpl>(self, oracle: OracleImpl) -> Result<Self::Output, Self::Error>
        where
            OracleImpl: Oracle<BaseC>,
        {
            Ok(price::total(self.in_amount, oracle.price_of()?.inv()))
        }
    }

    oracle_ref.execute_as_oracle(
        PriceConvert {
            in_amount,
            _out: PhantomData,
        },
        querier,
    )
}
