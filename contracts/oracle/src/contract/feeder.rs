use std::{collections::HashSet, convert::TryInto};

use cosmwasm_std::{Addr, DepsMut, MessageInfo, Response, StdResult, Storage};
use finance::duration::Duration;
use marketprice::{feeders::PriceFeeders, market_price::QueryConfig};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use std::convert::TryFrom;

use crate::{state::config::Config, ContractError};
const PRECISION_FACTOR: u128 = 1_000_000_000;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Feeders {
    config: Config,
}

impl Feeders {
    const FEEDERS: PriceFeeders<'static> = PriceFeeders::new("feeders");

    pub fn get(storage: &dyn Storage) -> StdResult<HashSet<Addr>> {
        Self::FEEDERS.get(storage)
    }

    pub fn is_feeder(storage: &dyn Storage, address: &Addr) -> StdResult<bool> {
        Self::FEEDERS.is_registered(storage, address)
    }

    pub fn try_register(
        deps: DepsMut,
        info: MessageInfo,
        address: String,
    ) -> Result<Response, ContractError> {
        let config = Config::load(deps.storage)?;
        if info.sender != config.owner {
            return Err(ContractError::Unauthorized {});
        }
        // check if address is valid
        let f_address = deps.api.addr_validate(&address)?;
        Self::FEEDERS.register(deps, f_address)?;

        Ok(Response::new())
    }

    // this is a helper function so Decimal works with u64 rather than Uint128
    // also, we must *round up* here, as we need 8, not 7 feeders to reach 50% of 15 total
    fn feeders_needed(weight: usize, percentage: u8) -> usize {
        let weight128 = u128::try_from(weight).expect("usize to u128 overflow");
        let res = (PRECISION_FACTOR * weight128) * u128::from(percentage) / 100;
        ((res + PRECISION_FACTOR - 1) / PRECISION_FACTOR)
            .try_into()
            .expect("usize overflow")
    }

    pub fn query_config(storage: &dyn Storage, config: &Config) -> StdResult<QueryConfig> {
        let registered_feeders = Self::FEEDERS.get(storage)?;
        let all_feeders_cnt = registered_feeders.len();
        let feeders_needed =
            Self::feeders_needed(all_feeders_cnt, config.feeders_percentage_needed);

        Ok(QueryConfig::new(
            Duration::from_secs(config.price_feed_period_secs),
            feeders_needed,
        ))
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use cosmwasm_std::{from_binary, testing::mock_env, Addr};

    use crate::{
        contract::{execute, feeder::Feeders, query},
        msg::{ExecuteMsg, QueryMsg},
        tests::{dummy_default_instantiate_msg, setup_test},
    };

    #[test]
    // we ensure this rounds up (as it calculates needed votes)
    fn feeders_needed_rounds_properly() {
        // round up right below 1
        assert_eq!(8, Feeders::feeders_needed(3, 255));
        // round up right over 1
        assert_eq!(8, Feeders::feeders_needed(3, 254));
        assert_eq!(77, Feeders::feeders_needed(30, 254));

        // exact matches don't round
        assert_eq!(17, Feeders::feeders_needed(34, 50));
        assert_eq!(12, Feeders::feeders_needed(48, 25));
        assert_eq!(2, Feeders::feeders_needed(132, 1));
        assert_eq!(2, Feeders::feeders_needed(189, 1));
    }

    #[test]
    fn register_feeder() {
        let (mut deps, info) = setup_test(dummy_default_instantiate_msg());

        // register new feeder address
        let msg = ExecuteMsg::RegisterFeeder {
            feeder_address: "addr0000".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // check if the new address is added to FEEDERS Item
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Feeders {}).unwrap();
        let resp: HashSet<Addr> = from_binary(&res).unwrap();
        assert_eq!(2, resp.len());
        assert!(resp.contains(&Addr::unchecked("addr0000")));

        // should not add the same address twice
        let msg = ExecuteMsg::RegisterFeeder {
            feeder_address: "addr0000".to_string(),
        };
        let _ = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        // validate that the address in not added twice
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Feeders {}).unwrap();
        let resp: HashSet<Addr> = from_binary(&res).unwrap();
        assert_eq!(2, resp.len());

        // register new feeder address
        let msg = ExecuteMsg::RegisterFeeder {
            feeder_address: "addr0001".to_string(),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        // check if the new address is added to FEEDERS Item
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Feeders {}).unwrap();
        let resp: HashSet<Addr> = from_binary(&res).unwrap();
        assert_eq!(3, resp.len());
        assert!(resp.contains(&Addr::unchecked("addr0000")));
        assert!(resp.contains(&Addr::unchecked("addr0001")));
    }
}
