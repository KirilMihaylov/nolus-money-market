use serde::{Deserialize, Serialize};

use currencies::test::StableC;
use currency::{Currency, SymbolStatic};
use finance::{
    coin::{Amount, Coin},
    duration::Duration,
    liability::Liability,
    percent::Percent,
};
use lease::api::{
    open::{ConnectionParams, Ics20Channel, InterestPaymentSpec, PositionSpecDTO},
    LpnCoinDTO,
};
use platform::contract::{Code, CodeId};

use sdk::{
    cosmwasm_std::{
        coins, from_json,
        testing::{mock_env, mock_info},
        to_json_binary, Addr, CosmosMsg, Deps, DepsMut, MessageInfo, SubMsg, WasmMsg,
    },
    schemars::{self, JsonSchema},
    testing::mock_deps_with_contracts,
};

use crate::{
    cmd::Borrow,
    contract::{execute, instantiate, query, sudo},
    msg::{ConfigResponse, ExecuteMsg, QueryMsg, SudoMsg},
    state::config::Config,
};

const CREATOR: &str = "creator";
const LPP_ADDR: &str = "test";
const TIMEALARMS_ADDR: &str = "timealarms";
const ORACLE_ADDR: &str = "oracle";
const PROFIT_ADDR: &str = "profit";
const RESERVE_ADDR: &str = "reserve";

type TheCurrency = StableC;

const DENOM: SymbolStatic = TheCurrency::TICKER;
const MARGIN_INTEREST_RATE: Percent = Percent::from_permille(30);

fn leaser_instantiate_msg(lease_code: Code, lpp: Addr) -> crate::msg::InstantiateMsg {
    crate::msg::InstantiateMsg {
        lease_code: CodeId::from(lease_code).into(),
        lpp,
        profit: Addr::unchecked(PROFIT_ADDR),
        reserve: Addr::unchecked(RESERVE_ADDR),
        time_alarms: Addr::unchecked(TIMEALARMS_ADDR),
        market_price_oracle: Addr::unchecked(ORACLE_ADDR),
        lease_position_spec: PositionSpecDTO::new(
            Liability::new(
                Percent::from_percent(65),
                Percent::from_percent(70),
                Percent::from_percent(73),
                Percent::from_percent(75),
                Percent::from_percent(78),
                Percent::from_percent(80),
                Duration::from_hours(1),
            ),
            lpn_coin(1000),
            lpn_coin(10),
        ),
        lease_interest_rate_margin: MARGIN_INTEREST_RATE,
        lease_due_period: Duration::from_days(90),
        dex: dex_params(),
    }
}

fn owner() -> MessageInfo {
    mock_info(CREATOR, &coins(2, DENOM))
}

fn customer() -> MessageInfo {
    mock_info("addr0000", &coins(2, DENOM))
}

fn setup_test_case(deps: DepsMut<'_>) {
    let lpp_addr = Addr::unchecked(LPP_ADDR);
    let msg = leaser_instantiate_msg(Code::unchecked(1), lpp_addr);

    let resp = instantiate(deps, mock_env(), owner(), msg).unwrap();
    assert_eq!(0, resp.messages.len());
}

fn query_config(deps: Deps<'_>) -> Config {
    let res = query(deps, mock_env(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_json(res).unwrap();
    config_response.config
}

fn dex_params() -> ConnectionParams {
    ConnectionParams {
        connection_id: "connection-0".into(),
        transfer_channel: Ics20Channel {
            local_endpoint: "channel-0".into(),
            remote_endpoint: "channel-2048".into(),
        },
    }
}

#[test]
fn proper_initialization() {
    let mut deps = mock_deps_with_contracts([LPP_ADDR, TIMEALARMS_ADDR, PROFIT_ADDR, ORACLE_ADDR]);

    let lpp_addr = Addr::unchecked(LPP_ADDR);
    let lease_code = Code::unchecked(1);
    let msg = leaser_instantiate_msg(lease_code, lpp_addr.clone());

    let res = instantiate(deps.as_mut(), mock_env(), owner(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config_response: ConfigResponse = from_json(res).unwrap();
    let config = config_response.config;
    assert_eq!(lease_code, config.lease_code);
    assert_eq!(lpp_addr, config.lpp);
}

#[test]
fn test_update_config() {
    let mut deps = mock_deps_with_contracts([LPP_ADDR, TIMEALARMS_ADDR, PROFIT_ADDR, ORACLE_ADDR]);

    let expected_liability = Liability::new(
        Percent::from_percent(55),
        Percent::from_percent(60),
        Percent::from_percent(61),
        Percent::from_percent(62),
        Percent::from_percent(64),
        Percent::from_percent(65),
        Duration::from_hours(12),
    );
    let expected_position_spec = PositionSpecDTO::new(
        expected_liability,
        lpn_coin(4_211_442_000),
        lpn_coin(100_000),
    );
    let expected_due_period = Duration::from_secs(100);

    setup_test_case(deps.as_mut());

    let msg = SudoMsg::Config {
        lease_interest_rate_margin: Percent::from_percent(5),
        lease_position_spec: expected_position_spec.clone(),
        lease_due_period: expected_due_period,
    };

    sudo(deps.as_mut(), mock_env(), msg).unwrap();

    let config = query_config(deps.as_ref());
    assert_eq!(expected_position_spec, config.lease_position_spec);
    assert_eq!(expected_due_period, config.lease_due_period);
}

#[test]
#[should_panic(expected = "Healthy % should be < first liquidation %")]
fn test_update_config_invalid_liability() {
    let mut deps = mock_deps_with_contracts([LPP_ADDR, TIMEALARMS_ADDR, PROFIT_ADDR, ORACLE_ADDR]);

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum MockSudoMsg {
        Config {
            lease_interest_rate_margin: Percent,
            lease_position_spec: PositionSpecDTO,
            lease_interest_payment: InterestPaymentSpec,
        },
    }

    let liability = Liability::new(
        Percent::from_percent(55),
        Percent::from_percent(110),
        Percent::ZERO,
        Percent::from_percent(55),
        Percent::from_percent(110),
        Percent::from_percent(165),
        Duration::from_secs(100),
    );

    let mock_msg = MockSudoMsg::Config {
        lease_interest_rate_margin: Percent::from_percent(5),
        lease_position_spec: PositionSpecDTO::new(liability, lpn_coin(6_433_000), lpn_coin(99_000)),
        lease_interest_payment: InterestPaymentSpec::new(
            Duration::from_secs(20),
            Duration::from_secs(10),
        ),
    };

    let msg: SudoMsg = from_json(to_json_binary(&mock_msg).unwrap()).unwrap();

    setup_test_case(deps.as_mut());

    sudo(deps.as_mut(), mock_env(), msg).unwrap();
}

fn open_lease_with(max_ltd: Option<Percent>) {
    let mut deps = mock_deps_with_contracts([LPP_ADDR, TIMEALARMS_ADDR, PROFIT_ADDR, ORACLE_ADDR]);

    setup_test_case(deps.as_mut());

    let config = query_config(deps.as_ref());

    let msg = ExecuteMsg::OpenLease {
        currency: DENOM.to_string(),
        max_ltd,
    };
    let info = customer();
    let env = mock_env();
    let admin = env.contract.address.clone();
    let finalizer = admin.clone();
    let res = execute(deps.as_mut(), env, info.clone(), msg).unwrap();

    let msg = Borrow::open_lease_msg(info.sender, config, DENOM.to_string(), max_ltd, finalizer);
    assert_eq!(
        res.messages,
        vec![SubMsg::reply_on_success(
            CosmosMsg::Wasm(WasmMsg::Instantiate {
                funds: info.funds,
                msg: to_json_binary(&msg).unwrap(),
                admin: Some(admin.into()),
                code_id: 1,
                label: "lease".to_string(),
            }),
            0,
        )]
    );
}

#[test]
fn test_open_lease() {
    open_lease_with(None);
}

#[test]
fn test_open_lease_with_max_ltd() {
    open_lease_with(None);
    open_lease_with(Some(Percent::from_percent(5)));
}

fn lpn_coin(amount: Amount) -> LpnCoinDTO {
    Coin::<TheCurrency>::from(amount).into()
}
