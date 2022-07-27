use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdError, StdResult};
use cw_multi_test::{App, ContractWrapper, Executor};
use marketprice::storage::Price;

use super::{ADMIN, NATIVE_DENOM};

pub struct MarketOracleWrapper {
    contract_wrapper: Box<
        ContractWrapper<
            oracle::msg::ExecuteMsg,
            oracle::msg::InstantiateMsg,
            oracle::msg::QueryMsg,
            oracle::ContractError,
            oracle::ContractError,
            StdError,
        >,
    >,
}

impl MarketOracleWrapper {
    pub fn with_contract_wrapper(
        contract: ContractWrapper<
            oracle::msg::ExecuteMsg,
            oracle::msg::InstantiateMsg,
            oracle::msg::QueryMsg,
            oracle::ContractError,
            oracle::ContractError,
            StdError,
        >,
    ) -> Self {
        Self {
            contract_wrapper: Box::new(contract),
        }
    }
    #[track_caller]
    pub fn instantiate(self, app: &mut App, denom: &str, timealarms_addr: &str) -> Addr {
        let code_id = app.store_code(self.contract_wrapper);
        let msg = oracle::msg::InstantiateMsg {
            base_asset: denom.to_string(),
            price_feed_period_secs: 60,
            feeders_percentage_needed: 1,
            supported_denom_pairs: vec![("UST".to_string(), NATIVE_DENOM.to_string())],
            timealarms_addr: timealarms_addr.to_string(),
        };
        app.instantiate_contract(code_id, Addr::unchecked(ADMIN), &msg, &[], "oracle", None)
            .unwrap()
    }
}

impl Default for MarketOracleWrapper {
    fn default() -> Self {
        let contract = ContractWrapper::new(
            oracle::contract::execute,
            oracle::contract::instantiate,
            oracle::contract::query,
        );

        Self {
            contract_wrapper: Box::new(contract),
        }
    }
}

pub fn mock_oracle_query(deps: Deps, env: Env, msg: oracle::msg::QueryMsg) -> StdResult<Binary> {
    let res = match msg {
        oracle::msg::QueryMsg::PriceFor { denoms: _ } => to_binary(&oracle::msg::PriceResponse {
            prices: vec![Price::new(NATIVE_DENOM, 1000000000, "UST", 123456789)],
        }),
        _ => Ok(oracle::contract::query(deps, env, msg)?),
    }?;

    Ok(res)
}
