use std::slice;

use ::lease::api::{query::StateResponse, ExecuteMsg};
use ::swap::RequestMsg as SwapRequestMsg;
use currency::Currency;
use finance::{
    coin::{Amount, Coin},
    duration::Duration,
    fraction::Fraction,
    percent::Percent,
    price::{self, Price},
    ratio::Rational,
    zero::Zero,
};
use platform::coin_legacy::to_cosmwasm_on_dex;
use sdk::{
    cosmwasm_std::{Addr, Timestamp},
    cw_multi_test::AppResponse,
};

use crate::common::{
    self, cwcoin, ibc,
    leaser::Instantiator as LeaserInstantiator,
    swap::{self, DexDenom},
    test_case::{response::ResponseWithInterChainMsgs, TestCase},
    CwCoin, USER,
};

use super::{
    price_lpn_of, LeaseCoin, LeaseCurrency, LeaseTestCase, LpnCoin, LpnCurrency, PaymentCoin,
    PaymentCurrency, DOWNPAYMENT,
};

#[test]
fn partial_repay() {
    let mut test_case: LeaseTestCase = super::create_test_case::<PaymentCurrency>();
    let downpayment = DOWNPAYMENT;

    let amount = super::quote_borrow(&test_case, downpayment);
    let partial_payment: PaymentCoin = Fraction::<PaymentCoin>::of(
        &Rational::new(1, 2),
        super::create_payment_coin(amount.into()),
    );
    let expected_result =
        super::expected_newly_opened_state(&test_case, downpayment, partial_payment);

    let lease_addr: Addr = super::open_lease(&mut test_case, downpayment, None);

    repay(&mut test_case, lease_addr.clone(), partial_payment);

    let query_result = super::state_query(&test_case, lease_addr.as_str());

    assert_eq!(query_result, expected_result);
}

#[test]
fn partial_repay_after_time() {
    let mut test_case = super::create_test_case::<PaymentCurrency>();
    let downpayment: PaymentCoin = DOWNPAYMENT;

    let lease_address = super::open_lease(&mut test_case, downpayment, None);

    test_case.app.time_shift(Duration::from_nanos(
        LeaserInstantiator::REPAYMENT_PERIOD.nanos() >> 1,
    ));

    let query_result = super::state_query(&test_case, lease_address.as_ref());

    let StateResponse::Opened {
        previous_margin_due,
        previous_interest_due,
        current_margin_due,
        ..
    } = query_result
    else {
        unreachable!()
    };

    super::feed_price(&mut test_case);

    let current_margin_to_pay: LpnCoin = LpnCoin::try_from(current_margin_due)
        .unwrap()
        .checked_div(2)
        .unwrap();

    repay(
        &mut test_case,
        lease_address.clone(),
        price::total(
            LpnCoin::try_from(previous_margin_due).unwrap()
                + LpnCoin::try_from(previous_interest_due).unwrap()
                + current_margin_to_pay,
            super::price_lpn_of::<PaymentCurrency>().inv(),
        ),
    );

    let query_result = super::state_query(&test_case, lease_address.as_str());

    if let StateResponse::Opened {
        previous_margin_due,
        previous_interest_due,
        ..
    } = query_result
    {
        assert!(
            previous_margin_due.is_zero(),
            "Expected 0 for margin interest due, got {}",
            previous_margin_due.amount()
        );

        assert!(
            previous_interest_due.is_zero(),
            "Expected 0 for interest due, got {}",
            previous_interest_due.amount()
        );
    } else {
        unreachable!()
    }
}

#[test]
#[should_panic = "[Lease] Insufficient payment amount"]
fn insufficient_payment() {
    let mut test_case = super::create_test_case::<PaymentCurrency>();
    let downpayment = DOWNPAYMENT;

    let lease_address = super::open_lease(&mut test_case, downpayment, None);

    let payment: PaymentCoin = super::create_payment_coin(49);
    repay(&mut test_case, lease_address, payment);
}

#[test]
fn full_repay() {
    let mut test_case = super::create_test_case::<PaymentCurrency>();
    let downpayment: PaymentCoin = DOWNPAYMENT;
    let lease_address = super::open_lease(&mut test_case, downpayment, None);
    let borrowed_lpn = super::quote_borrow(&test_case, downpayment);
    let borrowed: PaymentCoin = price::total(borrowed_lpn, super::price_lpn_of().inv());

    repay(&mut test_case, lease_address.clone(), borrowed);

    let expected_amount: LeaseCoin = price::total(
        price::total(
            downpayment + borrowed,
            /* Payment -> LPN */ super::price_lpn_of(),
        ),
        /* LPN -> Lease */ super::price_lpn_of().inv(),
    );
    let expected_result = StateResponse::Paid {
        amount: LeaseCoin::into(expected_amount),
        in_progress: None,
    };
    let query_result = super::state_query(&test_case, lease_address.as_str());

    assert_eq!(query_result, expected_result);
}

#[test]
fn full_repay_with_max_ltd() {
    let mut test_case = super::create_test_case::<PaymentCurrency>();
    let downpayment = DOWNPAYMENT;
    let percent = Percent::from_percent(10);
    let borrowed = percent.of(DOWNPAYMENT);
    let lease_address = super::open_lease(&mut test_case, downpayment, Some(percent));

    let lease_amount = (Percent::HUNDRED + percent).of(price::total(
        downpayment,
        Price::<PaymentCurrency, LeaseCurrency>::identity(),
    ));
    let expected_result = StateResponse::Opened {
        amount: lease_amount.into(),
        loan_interest_rate: Percent::from_permille(70),
        margin_interest_rate: Percent::from_permille(30),
        principal_due: price::total(percent.of(downpayment), super::price_lpn_of()).into(),
        previous_margin_due: LpnCoin::ZERO.into(),
        previous_interest_due: LpnCoin::ZERO.into(),
        current_margin_due: LpnCoin::ZERO.into(),
        current_interest_due: LpnCoin::ZERO.into(),
        validity: Timestamp::from_nanos(1537237454879305533),
        in_progress: None,
    };
    let query_result = super::state_query(&test_case, lease_address.as_str());

    assert_eq!(query_result, expected_result);

    repay(&mut test_case, lease_address.clone(), borrowed);

    let expected_amount: LeaseCoin = price::total(
        price::total(
            downpayment + borrowed,
            /* Payment -> LPN */ super::price_lpn_of(),
        ),
        /* LPN -> Lease */ super::price_lpn_of().inv(),
    );
    let expected_result = StateResponse::Paid {
        amount: LeaseCoin::into(expected_amount),
        in_progress: None,
    };
    let query_result = super::state_query(&test_case, lease_address.as_str());

    assert_eq!(query_result, expected_result);
}

#[test]
fn full_repay_with_excess() {
    let mut test_case = super::create_test_case::<PaymentCurrency>();
    let downpayment: PaymentCoin = DOWNPAYMENT;
    let lease_address = super::open_lease(&mut test_case, downpayment, None);
    let borrowed: PaymentCoin = price::total(
        super::quote_borrow(&test_case, downpayment),
        super::price_lpn_of().inv(),
    );

    let overpayment = super::create_payment_coin(5);
    let payment: PaymentCoin = borrowed + overpayment;

    repay(&mut test_case, lease_address.clone(), payment);

    let query_result = super::state_query(&test_case, lease_address.as_str());

    assert_eq!(
        test_case
            .app
            .query()
            .query_all_balances(lease_address.clone())
            .unwrap(),
        &[cwcoin::<LpnCurrency, Amount>(overpayment.into())],
    );

    assert_eq!(
        test_case
            .app
            .query()
            .query_all_balances(TestCase::ica_addr(
                lease_address.as_str(),
                TestCase::LEASE_ICA_ID
            ))
            .unwrap(),
        &[to_cosmwasm_on_dex(price::total(
            price::total(downpayment + borrowed, price_lpn_of()),
            price_lpn_of::<LeaseCurrency>().inv()
        ))],
    );

    assert_eq!(
        query_result,
        StateResponse::Paid {
            amount: LeaseCoin::into(price::total(
                price::total(downpayment + borrowed, price_lpn_of()),
                price_lpn_of().inv(),
            )),
            in_progress: None,
        }
    );
}

pub(crate) fn repay<ProtocolsRegistry, Dispatcher, Treasury, Profit, Leaser, Lpp, Oracle>(
    test_case: &mut TestCase<
        ProtocolsRegistry,
        Dispatcher,
        Treasury,
        Profit,
        Leaser,
        Lpp,
        Oracle,
        Addr,
    >,
    lease_addr: Addr,
    payment: PaymentCoin,
) -> AppResponse {
    let mut response: ResponseWithInterChainMsgs<'_, ()> =
        send_payment_and_transfer(test_case, lease_addr.clone(), payment);

    let requests: Vec<SwapRequestMsg> = swap::expect_swap(
        &mut response,
        TestCase::DEX_CONNECTION_ID,
        TestCase::LEASE_ICA_ID,
    );

    () = response.unwrap_response();

    let swap_out_lpn: LpnCoin = price::total(payment, super::price_lpn_of());

    let ica_addr: Addr = TestCase::ica_addr(lease_addr.as_str(), TestCase::LEASE_ICA_ID);

    let mut response: ResponseWithInterChainMsgs<'_, ()> = swap::do_swap(
        &mut test_case.app,
        lease_addr.clone(),
        ica_addr.clone(),
        requests.into_iter(),
        |amount: u128, in_denom: DexDenom<'_>, out_denom: DexDenom<'_>| {
            assert_eq!(amount, payment.into());
            assert_eq!(in_denom, PaymentCurrency::DEX_SYMBOL);
            assert_eq!(out_denom, LpnCurrency::DEX_SYMBOL);

            swap_out_lpn.into()
        },
    )
    .ignore_response();

    let transfer_amount: CwCoin = ibc::expect_remote_transfer(
        &mut response,
        TestCase::DEX_CONNECTION_ID,
        TestCase::LEASE_ICA_ID,
    );

    assert_eq!(transfer_amount, to_cosmwasm_on_dex(swap_out_lpn));

    () = response.unwrap_response();

    ibc::do_transfer(
        &mut test_case.app,
        ica_addr,
        lease_addr,
        true,
        &transfer_amount,
    )
    .unwrap_response()
}

fn send_payment_and_transfer<
    ProtocolsRegistry,
    Dispatcher,
    Treasury,
    Profit,
    Leaser,
    Lpp,
    Oracle,
    PaymentC,
>(
    test_case: &mut TestCase<
        ProtocolsRegistry,
        Dispatcher,
        Treasury,
        Profit,
        Leaser,
        Lpp,
        Oracle,
        Addr,
    >,
    lease_addr: Addr,
    payment: Coin<PaymentC>,
) -> ResponseWithInterChainMsgs<'_, ()>
where
    PaymentC: Currency,
{
    let payment_cw: CwCoin = common::cwcoin(payment);
    let mut response: ResponseWithInterChainMsgs<'_, ()> = test_case
        .app
        .execute(
            Addr::unchecked(USER),
            lease_addr.clone(),
            &ExecuteMsg::Repay {},
            slice::from_ref(&payment_cw),
        )
        .unwrap()
        .ignore_response();

    let ica_addr: Addr = TestCase::ica_addr(lease_addr.as_str(), TestCase::LEASE_ICA_ID);

    let transfer_amount: CwCoin = ibc::expect_transfer(
        &mut response,
        TestCase::LEASER_IBC_CHANNEL,
        lease_addr.as_str(),
        ica_addr.as_str(),
    );

    assert_eq!(transfer_amount, payment_cw);

    () = response.unwrap_response();

    ibc::do_transfer(
        &mut test_case.app,
        lease_addr,
        ica_addr,
        false,
        &transfer_amount,
    )
    .ignore_response()
}
