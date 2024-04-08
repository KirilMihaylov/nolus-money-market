use currencies::test::{NativeC, StableC};
use currency::{Currency, NlsPlatform};
use finance::coin::{Amount, Coin};
use sdk::{
    cosmwasm_std::{Addr, Event, QuerierWrapper},
    cw_multi_test::{AppResponse, ContractWrapper},
};
use treasury::msg::ConfigResponse;

use crate::common::{
    cwcoin,
    lpp::{self as lpp_mod, LppExecuteMsg, LppQueryMsg},
    oracle as oracle_mod,
    protocols::Registry,
    test_case::{builder::BlankBuilder as TestCaseBuilder, TestCase},
    ADDON_OPTIMAL_INTEREST_RATE, BASE_INTEREST_RATE, USER, UTILIZATION_OPTIMAL,
};

type Lpn = StableC;

type DispatcherTestCase = TestCase<Addr, Addr, (), (), (), Addr, Addr, Addr>;

#[test]
fn on_alarm_zero_reward() {
    let mut test_case = new_test_case(Registry::NoProtocol);

    test_case.send_funds_from_admin(Addr::unchecked(USER), &[cwcoin::<Lpn, _>(500)]);

    test_case.send_funds_from_admin(
        test_case.address_book.time_alarms().clone(),
        &[cwcoin::<Lpn, _>(500)],
    );

    let treasury_balance_before: Coin<NlsPlatform> = treasury_balance(&test_case);
    let resp = test_case
        .app
        .execute(
            test_case.address_book.time_alarms().clone(),
            test_case.address_book.treasury().clone(),
            &treasury::msg::ExecuteMsg::TimeAlarm {},
            &[],
        )
        .unwrap()
        .unwrap_response();

    assert_eq!(None, resp.data);
    assert_eq!(treasury_balance_before, treasury_balance(&test_case));
}

#[test]
fn on_alarm_one_protocol() {
    on_alarm_n_protocols(Registry::SingleProtocol, 1);
}

#[test]
fn on_alarm_two_protocols() {
    on_alarm_n_protocols(Registry::TwoProtocols, 2);
}

#[test]
fn test_config() {
    let mut test_case = new_test_case(Registry::TwoProtocols);

    let resp = query_config(&test_case);
    assert_eq!(resp.cadence_hours, 10);

    let response: AppResponse = test_case
        .app
        .sudo(
            test_case.address_book.treasury().clone(),
            &treasury::msg::SudoMsg::Config { cadence_hours: 30 },
        )
        .unwrap()
        .unwrap_response();
    assert_eq!(response.data, None);
    assert_eq!(
        &response.events,
        &[Event::new("sudo").add_attribute("_contract_address", "contract4"),]
    );

    let resp = query_config(&test_case);
    assert_eq!(resp.cadence_hours, 30);
}

fn new_test_case(registry: Registry) -> DispatcherTestCase {
    TestCaseBuilder::<Lpn>::new()
        .init_lpp(
            Some(
                ContractWrapper::new(
                    lpp::contract::execute,
                    lpp::contract::instantiate,
                    lpp_mod::mock_query,
                )
                .with_sudo(lpp::contract::sudo),
            ),
            BASE_INTEREST_RATE,
            UTILIZATION_OPTIMAL,
            ADDON_OPTIMAL_INTEREST_RATE,
            TestCase::DEFAULT_LPP_MIN_UTILIZATION,
        )
        .init_oracle(Some(
            ContractWrapper::new(
                oracle::contract::execute,
                oracle::contract::instantiate,
                oracle_mod::mock_query,
            )
            .with_reply(oracle::contract::reply)
            .with_sudo(oracle::contract::sudo),
        ))
        .init_protocols_registry(registry)
        .init_time_alarms()
        .init_treasury()
        .into_generic()
}

fn on_alarm_n_protocols(registry: Registry, protocols_nb: usize) {
    const REWARD: Coin<NlsPlatform> = Coin::new(4);
    let lender = Addr::unchecked(USER);

    let mut test_case = new_test_case(registry);

    let treasury = test_case.address_book.treasury().clone();
    test_case
        .send_funds_from_admin(lender.clone(), &[cwcoin::<Lpn, _>(500)])
        .send_funds_from_admin(treasury, &[cwcoin::<NlsPlatform, _>(123)]);

    assert!(lpp_balance(&test_case).is_zero());

    let treasury_balance_before: Coin<NlsPlatform> = treasury_balance(&test_case);

    () = test_case
        .app
        .execute(
            lender.clone(),
            test_case.address_book.lpp().clone(),
            &LppExecuteMsg::Deposit {},
            &[cwcoin::<Lpn, _>(500)],
        )
        .unwrap()
        .ignore_response()
        .unwrap_response();

    let res: AppResponse = test_case
        .app
        .execute(
            test_case.address_book.time_alarms().clone(),
            test_case.address_book.treasury().clone(),
            &treasury::msg::ExecuteMsg::TimeAlarm {},
            &[],
        )
        .unwrap()
        .unwrap_response();

    let rewards_total = REWARD
        .checked_mul(protocols_nb.try_into().unwrap())
        .unwrap();
    assert_eq!(
        treasury_balance_before - treasury_balance(&test_case),
        rewards_total
    );
    assert_eq!(lpp_balance(&test_case), rewards_total);

    let resp: lpp::msg::RewardsResponse = test_case
        .app
        .query()
        .query_wasm_smart(
            test_case.address_book.lpp().clone(),
            &LppQueryMsg::Rewards { address: lender },
        )
        .unwrap();

    assert_eq!(resp.rewards, rewards_total);
    check_events(&test_case, &res.events, protocols_nb, REWARD);
}

fn check_events(
    test_case: &DispatcherTestCase,
    events: &Vec<Event>,
    protocols_nb: usize,
    exp_reward: Coin<NlsPlatform>,
) {
    assert_eq!(events.len(), 2 + protocols_nb * 2, "{:?}", events);

    let mut event_index = 0;
    {
        let dispatch_exec = &events[event_index];
        event_index += 1;
        assert_eq!(dispatch_exec.ty, "execute");
        assert_eq!(
            dispatch_exec.attributes,
            [("_contract_address", test_case.address_book.treasury())]
        );
    }

    (0..protocols_nb).for_each(|_| {
        let dispatch_exec = &events[event_index];
        event_index += 1;
        assert_eq!(dispatch_exec.ty.as_str(), "wasm-tr-rewards");
        assert_eq!(
            dispatch_exec.attributes,
            [
                (
                    "_contract_address",
                    test_case.address_book.treasury().as_str()
                ),
                ("height", &test_case.app.block_info().height.to_string()),
                ("at", &test_case.app.block_info().time.nanos().to_string()),
                ("idx", "0"),
                ("to", test_case.address_book.lpp().as_str()),
                (
                    "rewards-amount",
                    &Into::<Amount>::into(exp_reward).to_string()
                ),
                ("rewards-symbol", NativeC::TICKER),
            ]
        );
    });

    {
        (0..protocols_nb).for_each(|_| {
            let lpp_exec = &events[event_index];
            event_index += 1;
            assert_eq!(lpp_exec.ty.as_str(), "execute");
            assert_eq!(
                lpp_exec.attributes,
                [("_contract_address", &test_case.address_book.lpp())]
            );
        });
    }

    let time_alarms_exec = &events[event_index];
    assert_eq!(time_alarms_exec.ty.as_str(), "execute");
    assert_eq!(
        time_alarms_exec.attributes,
        [("_contract_address", &test_case.address_book.time_alarms())]
    );
}

fn query_config(test_case: &DispatcherTestCase) -> ConfigResponse {
    test_case
        .app
        .query()
        .query_wasm_smart(
            test_case.address_book.treasury().clone(),
            &treasury::msg::QueryMsg::Config {},
        )
        .unwrap()
}

fn treasury_balance(test_case: &DispatcherTestCase) -> Coin<NlsPlatform> {
    balance(test_case.address_book.treasury(), test_case.app.query())
}

fn lpp_balance(test_case: &DispatcherTestCase) -> Coin<NlsPlatform> {
    balance(test_case.address_book.lpp(), test_case.app.query())
}

fn balance(account: &Addr, querier: QuerierWrapper<'_>) -> Coin<NlsPlatform> {
    platform::bank::balance(account, querier).unwrap()
}