use cosmwasm_std::{Storage, Timestamp};
use marketprice::SpotPrice;
use serde::de::DeserializeOwned;

use access_control::SingleUserAccess;
use currency::lpn::Lpns;
use finance::currency::{visit_any_on_ticker, AnyVisitor, Currency};
use sdk::{
    cosmwasm_ext::Response,
    cosmwasm_std::{Addr, DepsMut, Env},
};

use crate::{
    error::ContractError,
    msg::ExecuteMsg,
    state::{config::Config, supported_pairs::SupportedPairs},
};

use super::{
    feed::{self, Feeds},
    feeder::Feeders,
};

pub struct ExecWithOracleBase<'a> {
    deps: DepsMut<'a>,
    env: Env,
    msg: ExecuteMsg,
    sender: Addr,
}

impl<'a> ExecWithOracleBase<'a> {
    pub fn cmd(
        deps: DepsMut<'a>,
        env: Env,
        msg: ExecuteMsg,
        sender: Addr,
    ) -> Result<Response, ContractError> {
        let visitor = Self {
            deps,
            env,
            msg,
            sender,
        };

        let config = Config::load(visitor.deps.storage)?;
        visit_any_on_ticker::<Lpns, _>(&config.base_asset, visitor)
    }
}

impl<'a> AnyVisitor for ExecWithOracleBase<'a> {
    type Output = Response;
    type Error = ContractError;

    fn on<OracleBase>(self) -> Result<Self::Output, Self::Error>
    where
        OracleBase: Currency + DeserializeOwned,
    {
        match self.msg {
            ExecuteMsg::SwapTree { tree } => {
                SingleUserAccess::load_and_check_owner_access::<ContractError>(
                    self.deps.storage,
                    &self.sender,
                )?;

                SupportedPairs::<OracleBase>::new(tree)?
                    .validate_tickers()?
                    .save(self.deps.storage)?;

                Ok(Response::default())
            }
            ExecuteMsg::FeedPrices { prices } => {
                if !Feeders::is_feeder(self.deps.storage, &self.sender)? {
                    return Err(ContractError::UnknownFeeder {});
                }

                try_feed_prices::<OracleBase>(
                    self.deps.storage,
                    self.env.block.time,
                    self.sender,
                    prices,
                )
            }
            ExecuteMsg::DispatchAlarms { max_count } => feed::try_notify_alarms::<OracleBase>(
                self.deps.storage,
                self.env.block.time,
                max_count,
            ),
            _ => {
                unreachable!()
            }
        }
    }
}

fn try_feed_prices<OracleBase>(
    storage: &mut dyn Storage,
    block_time: Timestamp,
    sender: Addr,
    prices: Vec<SpotPrice>,
) -> Result<Response, ContractError>
where
    OracleBase: Currency + DeserializeOwned,
{
    let config = Config::load(storage)?;
    let oracle = Feeds::<OracleBase>::with(config.price_config);

    if !prices.is_empty() {
        oracle.feed_prices(storage, block_time, &sender, &prices)?;
    }

    Ok(Response::default())
}
