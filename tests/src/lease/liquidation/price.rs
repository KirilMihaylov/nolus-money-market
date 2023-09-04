use ::lease::api::{ExecuteMsg, StateResponse};
use currency::Currency;
use finance::{coin::Amount, percent::Percent};
use sdk::{
    cosmwasm_std::{Addr, Binary, Event},
    cw_multi_test::AppResponse,
};

use crate::{
    common::{
        self, cwcoin,
        leaser::{self, Instantiator as LeaserInstantiator},
        test_case::{
            response::{RemoteChain, ResponseWithInterChainMsgs},
            TestCase,
        },
        ADMIN, USER,
    },
    lease::{self, LeaseTestCase},
};

use super::{LeaseCoin, LeaseCurrency, LpnCoin, PaymentCurrency, DOWNPAYMENT};

#[test]
#[should_panic = "No liquidation warning emitted!"]
fn liquidation_warning_price_0() {
    liquidation_warning(
        2085713.into(),
        1857159.into(),
        LeaserInstantiator::liability().max(), //not used
        "N/A",
    );
}

#[test]
fn liquidation_warning_price_1() {
    liquidation_warning(
        // ref: 2085713
        2085713.into(),
        // ref: 1857159
        1827159.into(),
        LeaserInstantiator::liability().first_liq_warn(),
        "1",
    );
}

#[test]
fn liquidation_warning_price_2() {
    liquidation_warning(
        // ref: 2085713
        2085713.into(),
        // ref: 1857159
        1757159.into(),
        LeaserInstantiator::liability().second_liq_warn(),
        "2",
    );
}

#[test]
fn liquidation_warning_price_3() {
    liquidation_warning(
        // ref: 2085713
        2085713.into(),
        // ref: 1857159
        1707159.into(),
        LeaserInstantiator::liability().third_liq_warn(),
        "3",
    );
}

#[test]
fn full_liquidation() {
    let mut test_case = lease::create_test_case::<PaymentCurrency>();
    let lease = lease::open_lease(
        &mut test_case,
        lease::create_payment_coin(DOWNPAYMENT),
        None,
    );

    // loan = 1857142857142
    // asset = 2857142857142
    // the base is chosen to be close to the asset amount to trigger a full liquidation
    let base = 2857142857140.into();
    let quote = 1857142857142.into();

    let mut response_with_ica = deliver_new_price(&mut test_case, lease.clone(), base, quote);

    //swap
    response_with_ica.expect_submit_tx(TestCase::LEASER_CONNECTION_ID, "0", 1);
    let liquidation_start_response = response_with_ica.unwrap_response();
    liquidation_start_response.assert_event(&Event::new("wasm-ls-liquidation-start"));

    let liquidated_in_lpn: LpnCoin = quote;
    let liquidated_amount: Amount = liquidated_in_lpn.into();
    let mut response: ResponseWithInterChainMsgs<'_, ()> = test_case
        .app
        .sudo(
            lease.clone(),
            &sdk::neutron_sdk::sudo::msg::SudoMsg::Response {
                request: sdk::neutron_sdk::sudo::msg::RequestPacket {
                    sequence: None,
                    source_port: None,
                    source_channel: None,
                    destination_port: None,
                    destination_channel: None,
                    data: None,
                    timeout_height: None,
                    timeout_timestamp: None,
                },
                data: Binary(platform::trx::encode_msg_responses(
                    [swap::trx::build_exact_amount_in_resp(liquidated_amount)].into_iter(),
                )),
            },
        )
        .unwrap()
        .ignore_response();

    //transfer in
    response.expect_submit_tx(TestCase::LEASER_CONNECTION_ID, "0", 1);
    () = response.unwrap_response();

    test_case.send_funds_from_admin(lease.clone(), &[cwcoin(liquidated_in_lpn)]);

    let response_transfer_in = test_case
        .app
        .sudo(
            lease.clone(),
            &sdk::neutron_sdk::sudo::msg::SudoMsg::Response {
                request: sdk::neutron_sdk::sudo::msg::RequestPacket {
                    sequence: None,
                    source_port: None,
                    source_channel: None,
                    destination_port: None,
                    destination_channel: None,
                    data: None,
                    timeout_height: None,
                    timeout_timestamp: None,
                },
                data: Binary::default(),
            },
        )
        .unwrap()
        .unwrap_response();
    response_transfer_in.assert_event(
        &Event::new("wasm-ls-liquidation")
            .add_attribute("payment-amount", liquidated_amount.to_string())
            .add_attribute("loan-close", true.to_string()),
    );

    assert_eq!(
        test_case
            .app
            .query()
            .query_all_balances(lease.clone())
            .unwrap(),
        &[],
    );

    let state = lease::state_query(&test_case, lease.as_str());
    assert!(
        matches!(state, StateResponse::Liquidated()),
        "should have been in Liquidated state"
    );
    leaser::assert_no_leases(
        &test_case.app,
        test_case.address_book.leaser().clone(),
        Addr::unchecked(USER),
    )
}

fn liquidation_warning(base: LeaseCoin, quote: LpnCoin, liability: Percent, level: &str) {
    let mut test_case = lease::create_test_case::<PaymentCurrency>();
    let lease = lease::open_lease(
        &mut test_case,
        lease::create_payment_coin(DOWNPAYMENT),
        None,
    );

    let response: AppResponse =
        deliver_new_price(&mut test_case, lease, base, quote).unwrap_response();

    let event = response
        .events
        .iter()
        .find(|event| event.ty == "wasm-ls-liquidation-warning")
        .expect("No liquidation warning emitted!");

    let attribute = event
        .attributes
        .iter()
        .find(|attribute| attribute.key == "customer")
        .expect("Customer attribute not present!");

    assert_eq!(attribute.value, USER);

    let attribute = event
        .attributes
        .iter()
        .find(|attribute| attribute.key == "ltv")
        .expect("LTV attribute not present!");

    assert_eq!(attribute.value, liability.units().to_string());

    let attribute = event
        .attributes
        .iter()
        .find(|attribute| attribute.key == "level")
        .expect("Level attribute not present!");

    assert_eq!(attribute.value, level);

    let attribute = event
        .attributes
        .iter()
        .find(|attribute| attribute.key == "lease-asset")
        .expect("Lease Asset attribute not present!");

    assert_eq!(&attribute.value, LeaseCurrency::TICKER);
}

fn deliver_new_price(
    test_case: &mut LeaseTestCase,
    lease: Addr,
    base: LeaseCoin,
    quote: LpnCoin,
) -> ResponseWithInterChainMsgs<'_, AppResponse> {
    common::oracle::feed_price(test_case, Addr::unchecked(ADMIN), base, quote);

    test_case
        .app
        .execute(
            test_case.address_book.oracle().clone(),
            lease,
            &ExecuteMsg::PriceAlarm(),
            &[],
        )
        .unwrap()
}