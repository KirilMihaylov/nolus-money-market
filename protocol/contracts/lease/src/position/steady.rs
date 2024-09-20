use currency::{CurrencyDef, MemberOf};
use finance::{
    duration::Duration,
    range::{Descending, RightOpenRange},
};
use oracle::{
    api::alarms::Alarm,
    stub::{AsAlarms, PriceAlarms},
};
use platform::batch::Batch;
use sdk::cosmwasm_std::Timestamp;
use timealarms::stub::TimeAlarmsRef;

use crate::{api::LeaseAssetCurrencies, error::ContractResult, finance::OracleRef};

use super::Price;

/// The position would be steady, i.e. no warnings, automatic close, liquidations,
/// if the asset price is within a range and is guaranteed for a period of time.
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct Steadiness<Asset>
where
    Asset: 'static,
{
    r#for: Duration,
    within: RightOpenRange<Price<Asset>, Descending>,
}

impl<Asset> Steadiness<Asset>
where
    Asset: 'static,
{
    pub(super) fn new(r#for: Duration, within: RightOpenRange<Price<Asset>, Descending>) -> Self {
        Self { r#for, within }
    }
}

impl<Asset> Steadiness<Asset>
where
    Asset: CurrencyDef,
    Asset::Group: MemberOf<LeaseAssetCurrencies>,
{
    pub fn try_into_alarms(
        self,
        when: &Timestamp,
        time_alarms: &TimeAlarmsRef,
        price_alarms: &OracleRef,
    ) -> ContractResult<Batch> {
        time_alarms
            .setup_alarm(when + self.r#for)
            .map_err(Into::into)
            .and_then(|schedule_time_alarm| {
                let mut price_alarms = price_alarms.as_alarms::<LeaseAssetCurrencies>();
                price_alarms
                    .add_alarm(Alarm::<LeaseAssetCurrencies, _, _>::new(
                        // NOTE: we miss alarms in the exact case when the price == SteadyPriceRange::above_excl
                        // This is due to the discrepancy of the openness of liability LTV ranges and alarms.
                        // While the former are closed at the start and open at the end, the latter are in reverse.
                        // The best solution is to turn 'below' into 'below_or_equal' and 'above_or_equal' into 'above'.
                        self.within.above(),
                        self.within.may_below_or_equal(),
                    ))
                    .map(|_| schedule_time_alarm.merge(price_alarms.into()))
                    .map_err(Into::into)
            })
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::Timestamp;
    use currencies::{LeaseGroup, Lpns};
    use currencies::{Lpn, PaymentC3};
    use finance::fraction::Fraction;
    use finance::percent::Percent;
    use finance::range::RightOpenRange;
    use finance::{coin::Coin, duration::Duration, price};
    use oracle::api::alarms::{Alarm, ExecuteMsg as PriceAlarmsCmd};
    use oracle_platform::OracleRef;
    use platform::batch::Batch;
    use sdk::cosmwasm_std::{self, Addr, WasmMsg};
    use timealarms::{msg::ExecuteMsg as TimeAlarmsCmd, stub::TimeAlarmsRef};

    use crate::api::LeaseAssetCurrencies;
    use crate::position::Steadiness;

    const TIME_ALARMS_ADDR: &str = "timealarms";
    const ORACLE_ADDR: &str = "oracle";

    type TestCurrency = PaymentC3;
    type TestLpn = Lpn;

    #[test]
    fn try_into_alarms() {
        let now = Timestamp::from_seconds(1732016180);
        let recheck_in = Duration::from_secs(765758);
        let ltv_to_price = |ltv: Percent| {
            price::total_of::<TestCurrency>(ltv.of(Coin::from(100))).is(Coin::from(45))
        };

        let steady_below_ltv = Percent::from_percent(60);
        let steady_above_ltv = Percent::from_percent(43);
        let steady_above = Steadiness {
            r#for: recheck_in,
            within: RightOpenRange::up_to(steady_below_ltv).invert(ltv_to_price),
        };

        try_into_alarms_int(
            steady_above,
            now,
            recheck_in,
            Alarm::new(ltv_to_price(steady_below_ltv), None),
        );

        let steady_above_below = Steadiness {
            r#for: recheck_in,
            within: RightOpenRange::up_to(steady_below_ltv)
                .cut_to(steady_above_ltv)
                .invert(ltv_to_price),
        };
        try_into_alarms_int(
            steady_above_below,
            now,
            recheck_in,
            Alarm::new(
                ltv_to_price(steady_below_ltv),
                Some(ltv_to_price(steady_above_ltv)),
            ),
        );
    }

    fn try_into_alarms_int(
        s: Steadiness<TestCurrency>,
        now: Timestamp,
        recheck_in: Duration,
        exp_alarm: Alarm<LeaseAssetCurrencies, TestLpn, Lpns>,
    ) {
        assert_eq!(s.try_into_alarms(&now, &timealarms(), &pricealarms()), {
            let mut batch = Batch::default();

            batch.schedule_execute_no_reply(WasmMsg::Execute {
                contract_addr: TIME_ALARMS_ADDR.into(),
                msg: cosmwasm_std::to_json_binary(&TimeAlarmsCmd::AddAlarm {
                    time: now + recheck_in,
                })
                .unwrap(),
                funds: vec![],
            });

            batch.schedule_execute_no_reply(WasmMsg::Execute {
                contract_addr: ORACLE_ADDR.into(),
                msg: cosmwasm_std::to_json_binary(&PriceAlarmsCmd::AddPriceAlarm::<
                    LeaseGroup,
                    TestLpn,
                    Lpns,
                > {
                    alarm: exp_alarm,
                })
                .unwrap(),
                funds: vec![],
            });

            Ok(batch)
        });
    }

    fn timealarms() -> TimeAlarmsRef {
        TimeAlarmsRef::unchecked(TIME_ALARMS_ADDR)
    }

    fn pricealarms() -> OracleRef<TestLpn, Lpns> {
        OracleRef::unchecked(Addr::unchecked(ORACLE_ADDR))
    }
}