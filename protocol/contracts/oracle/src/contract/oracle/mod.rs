use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use currency::{Currency, CurrencyDTO, CurrencyDef, Group, MemberOf};
use finance::price::{
    base::{
        with_price::{self, WithPrice},
        BasePrice,
    },
    dto::PriceDTO,
    Price,
};
use marketprice::{config::Config as PriceConfig, Repo};
use platform::{
    dispatcher::{AlarmsDispatcher, Id},
    message::Response as MessageResponse,
};
use sdk::cosmwasm_std::{Addr, Storage, Timestamp};

use crate::{
    api::{AlarmsStatusResponse, Config, ExecuteAlarmMsg},
    contract::{alarms::MarketAlarms, oracle::feed::Feeds},
    error::ContractError,
    result::ContractResult,
    state::supported_pairs::SupportedPairs,
};

use self::feeder::Feeders;

pub mod feed;
pub mod feeder;

const ROOT_NAMESPACE: &str = "o11s";

pub(crate) type PriceResult<PriceG, OracleBase, OracleBaseG> =
    ContractResult<BasePrice<PriceG, OracleBase, OracleBaseG>>;

pub(crate) struct Oracle<'storage, S, PriceG, BaseC, BaseG>
where
    S: Deref<Target = dyn Storage + 'storage>,
    PriceG: Group,
    BaseC: Currency + MemberOf<BaseG>,
    BaseG: Group,
{
    storage: S,
    feeders: usize,
    config: Config,
    _price_g: PhantomData<PriceG>,
    _base_c: PhantomData<BaseC>,
    _base_g: PhantomData<BaseG>,
}

impl<'storage, S, PriceG, BaseC, BaseG> Oracle<'storage, S, PriceG, BaseC, BaseG>
where
    S: Deref<Target = dyn Storage + 'storage>,
    PriceG: Group<TopG = PriceG>,
    BaseC: CurrencyDef,
    BaseC::Group: MemberOf<BaseG> + MemberOf<PriceG>,
    BaseG: Group + MemberOf<PriceG>,
{
    pub fn load(storage: S) -> ContractResult<Self> {
        let feeders =
            Feeders::total_registered(storage.deref()).map_err(ContractError::LoadFeeders)?;
        Config::load(storage.deref()).map(|config| Self {
            storage,
            feeders,
            config,
            _price_g: PhantomData,
            _base_c: PhantomData,
            _base_g: PhantomData,
        })
    }

    pub(super) fn try_query_alarms(
        &self,
        block_time: Timestamp,
    ) -> ContractResult<AlarmsStatusResponse> {
        self.tree().and_then(|tree| {
            MarketAlarms::new(self.storage.deref())
                .try_query_alarms::<_, BaseC, BaseG>(self.calc_all_prices(
                    &tree,
                    &self.feeds_read_only(),
                    block_time,
                ))
                .map(|remaining_alarms| AlarmsStatusResponse { remaining_alarms })
        })
    }

    pub(super) fn try_query_prices(
        &self,
        block_time: Timestamp,
    ) -> ContractResult<Vec<BasePrice<PriceG, BaseC, BaseG>>> {
        self.tree().and_then(|tree| {
            self.calc_all_prices(&tree, &self.feeds_read_only(), block_time)
                .collect()
        })
    }

    pub(super) fn try_query_base_price(
        &self,
        at: Timestamp,
        currency: &CurrencyDTO<PriceG>,
    ) -> ContractResult<BasePrice<PriceG, BaseC, BaseG>> {
        self.tree().and_then(|tree| {
            self.feeds_read_only()
                .calc_base_price(&tree, currency, at, self.feeders)
        })
    }

    pub(super) fn try_query_stable_price<StableCurrency>(
        &self,
        at: Timestamp,
        currency: &CurrencyDTO<PriceG>,
    ) -> ContractResult<BasePrice<PriceG, StableCurrency, PriceG>>
    where
        StableCurrency: CurrencyDef,
        StableCurrency::Group: MemberOf<PriceG>,
    {
        struct StablePriceCalc<G, StableCurrency, StableG, BaseCurrency> {
            _currency_group: PhantomData<G>,
            stable_to_base: Price<StableCurrency, BaseCurrency>,
            _quote_group: PhantomData<StableG>,
        }
        impl<G, StableCurrency, StableG, BaseCurrency> WithPrice<BaseCurrency>
            for StablePriceCalc<G, StableCurrency, StableG, BaseCurrency>
        where
            StableCurrency: CurrencyDef,
            StableCurrency::Group: MemberOf<StableG> + MemberOf<G::TopG>,
            BaseCurrency: CurrencyDef,
            G: Group,
            StableG: Group,
        {
            type PriceG = G;

            type Output = BasePrice<G, StableCurrency, StableG>;

            type Error = ContractError;

            fn exec<BaseC>(
                self,
                base_price: Price<BaseC, BaseCurrency>,
            ) -> Result<Self::Output, Self::Error>
            where
                BaseC: CurrencyDef,
                BaseC::Group: MemberOf<Self::PriceG>,
            {
                Ok((base_price * self.stable_to_base.inv()).into())
            }
        }
        self.try_query_base_price(at, &currency::dto::<StableCurrency, _>())
            .and_then(|stable_price| {
                Price::try_from(&stable_price).map_err(Into::<ContractError>::into)
            })
            .and_then(|stable_price: Price<StableCurrency, BaseC>| {
                self.try_query_base_price(at, currency)
                    .and_then(|ref base_price| {
                        with_price::execute(
                            base_price,
                            StablePriceCalc {
                                _currency_group: PhantomData::<PriceG>,
                                stable_to_base: stable_price,
                                _quote_group: PhantomData::<PriceG>,
                            },
                        )
                    })
            })
    }

    fn calc_all_prices<'self_, 'tree, 'feeds, 'st>(
        &'self_ self,
        tree: &'tree SupportedPairs<PriceG, BaseC>,
        feeds: &'feeds Feeds<'_, PriceG, BaseC, BaseG, Repo<'st, &(dyn Storage + 'st), PriceG>>,
        at: Timestamp,
    ) -> impl Iterator<Item = PriceResult<PriceG, BaseC, BaseG>> + 'feeds
    where
        'storage: 'self_,
        'self_: 'feeds,
        'tree: 'feeds,
        'storage: 'feeds,
    {
        feeds.all_prices_iter(tree.swap_pairs_df(), at, self.feeders)
    }

    fn tree(&self) -> ContractResult<SupportedPairs<PriceG, BaseC>> {
        SupportedPairs::load(self.storage.deref())
    }

    fn feeds_read_only(
        &self,
    ) -> Feeds<'_, PriceG, BaseC, BaseG, Repo<'storage, &(dyn Storage + 'storage), PriceG>> {
        Self::feeds(&self.config.price_config, self.storage.deref())
    }

    fn feeds<'repo_storage, RepoStorage>(
        config: &PriceConfig,
        repo_storage: RepoStorage,
    ) -> Feeds<'_, PriceG, BaseC, BaseG, Repo<'repo_storage, RepoStorage, PriceG>>
    where
        RepoStorage: Deref<Target = dyn Storage + 'repo_storage>,
    {
        Feeds::with(config, Repo::new(ROOT_NAMESPACE, repo_storage))
    }
}

impl<'storage, S, PriceG, BaseC, BaseG> Oracle<'storage, S, PriceG, BaseC, BaseG>
where
    S: Deref<Target = dyn Storage + 'storage> + DerefMut,
    PriceG: Group<TopG = PriceG> + Clone,
    BaseC: CurrencyDef,
    BaseC::Group: MemberOf<BaseG> + MemberOf<PriceG>,
    BaseG: Group + MemberOf<PriceG>,
{
    const REPLY_ID: Id = 0;
    const EVENT_TYPE: &'static str = "pricealarm";

    pub(super) fn wipe_out_v2(store: &mut dyn Storage) {
        Feeds::<PriceG, BaseC, BaseG, Repo<'_, S, PriceG>>::wipe_out_v2(store)
    }

    pub(super) fn try_feed_prices(
        &mut self,
        block_time: Timestamp,
        sender: Addr,
        prices: Vec<PriceDTO<PriceG>>,
    ) -> ContractResult<()> {
        self.tree().and_then(|tree| {
            self.feeds_read_write()
                .feed_prices(&tree, block_time, sender, &prices)
        })
    }

    pub(super) fn try_notify_alarms(
        &mut self,
        block_time: Timestamp,
        max_count: u32,
    ) -> ContractResult<(u32, MessageResponse)> {
        let subscribers: Vec<Addr> = self.tree().and_then(|tree| {
            MarketAlarms::<'_, _, PriceG>::new(self.storage.deref())
                .ensure_no_in_delivery()?
                .notify_alarms_iter::<_, BaseC, BaseG>(self.calc_all_prices(
                    &tree,
                    &self.feeds_read_only(),
                    block_time,
                ))?
                .take(max_count.try_into()?)
                .collect::<ContractResult<Vec<Addr>>>()
        })?;

        #[cfg(debug_assertions)]
        Self::assert_unique_subscribers(&subscribers);

        let mut alarms: MarketAlarms<'_, &mut (dyn Storage + 'storage), PriceG> =
            MarketAlarms::new(self.storage.deref_mut());

        subscribers
            .into_iter()
            .try_fold(
                AlarmsDispatcher::new(ExecuteAlarmMsg::PriceAlarm(), Self::EVENT_TYPE),
                move |dispatcher: AlarmsDispatcher<ExecuteAlarmMsg>, subscriber: Addr| {
                    dispatcher
                        .send_to(subscriber.clone(), Self::REPLY_ID)
                        .map_err(Into::into)
                        .and_then(|dispatcher| {
                            alarms.out_for_delivery(subscriber).map(|()| dispatcher)
                        })
                },
            )
            .map(|dispatcher| (dispatcher.nb_sent(), dispatcher.into()))
    }

    #[cfg(debug_assertions)]
    fn assert_unique_subscribers(subscribers: &[Addr]) {
        use std::collections::HashSet;

        let set: HashSet<&Addr> = HashSet::from_iter(subscribers);

        assert_eq!(set.len(), subscribers.len());
    }

    fn feeds_read_write(
        &mut self,
    ) -> Feeds<'_, PriceG, BaseC, BaseG, Repo<'storage, &mut (dyn Storage + 'storage), PriceG>>
    {
        Self::feeds(&self.config.price_config, self.storage.deref_mut())
    }
}

#[cfg(test)]
mod test_normalized_price_not_found {
    use currencies::{
        Lpn as BaseCurrency, Lpns as BaseCurrencies, Nls, PaymentGroup as PriceCurrencies,
        PaymentGroup as AlarmCurrencies, Stable as StableCurrency,
    };
    use finance::{coin::Coin, duration::Duration, percent::Percent, price};
    use marketprice::{config::Config as PriceConfig, Repo};
    use sdk::{
        cosmwasm_std::{
            testing::{MockApi, MockQuerier, MockStorage},
            Addr, DepsMut, Empty, QuerierWrapper, Storage, Timestamp,
        },
        testing,
    };

    use crate::{
        api::{Alarm, Config},
        contract::alarms::MarketAlarms,
        state::supported_pairs::SupportedPairs,
        test_tree,
    };

    use super::{feed::Feeds, feeder::Feeders, Oracle};

    type NlsCoin = Coin<Nls>;
    type BaseCoin = Coin<BaseCurrency>;
    type TestSupportedPairs = SupportedPairs<PriceCurrencies, BaseCurrency>;

    const NOW: Timestamp = Timestamp::from_seconds(1);
    const TEST_NS: &str = "test_feeds";

    const PRICE_BASE: NlsCoin = Coin::new(1);
    const PRICE_QUOTE: BaseCoin = Coin::new(1);

    #[test]
    fn test() {
        let mut storage: MockStorage = MockStorage::new();

        let price_config: PriceConfig = PriceConfig::new(
            Percent::HUNDRED,
            Duration::from_secs(1),
            1,
            Percent::HUNDRED,
        );

        init(&mut storage, &price_config);

        add_alarm(&mut storage);

        feed_price(
            &price_config,
            &TestSupportedPairs::load(&storage).unwrap(),
            &mut storage,
        );

        dispatch_and_deliver(&mut storage, 1);

        // Bug happens on this step.
        dispatch_and_deliver(&mut storage, 0);
    }

    // #[track_caller]
    fn init(storage: &mut dyn Storage, price_config: &PriceConfig) {
        Feeders::try_register(
            DepsMut {
                storage,
                api: &MockApi::default(),
                querier: QuerierWrapper::new(&MockQuerier::<Empty>::new(&[])),
            },
            testing::user("feeder").to_string(),
        )
        .unwrap();

        Config::new(price_config.clone()).store(storage).unwrap();

        TestSupportedPairs::new::<StableCurrency>(test_tree::dummy_swap_tree().into_tree())
            .unwrap()
            .save(storage)
            .unwrap();
    }

    #[track_caller]
    fn add_alarm(storage: &mut dyn Storage) {
        let mut alarms: MarketAlarms<'_, &mut dyn Storage, _> = MarketAlarms::new(storage);

        alarms
            .try_add_price_alarm::<BaseCurrency, BaseCurrencies>(
                Addr::unchecked("1"),
                Alarm::<AlarmCurrencies, BaseCurrency, BaseCurrencies>::new(
                    price::total_of(PRICE_BASE).is(PRICE_QUOTE),
                    Some(price::total_of(PRICE_BASE).is(PRICE_QUOTE)),
                ),
            )
            .unwrap();
    }

    #[track_caller]
    fn feed_price(
        price_config: &PriceConfig,
        tree: &TestSupportedPairs,
        storage: &mut dyn Storage,
    ) {
        Feeds::<_, _, BaseCurrencies, _>::with(price_config, Repo::new(TEST_NS, storage))
            .feed_prices(
                tree,
                NOW,
                Addr::unchecked("feeder"),
                &[price::total_of(PRICE_BASE).is(PRICE_QUOTE).into()],
            )
            .unwrap();
    }

    #[track_caller]
    fn dispatch(storage: &mut dyn Storage, expected_count: u32) {
        let mut oracle: Oracle<
            '_,
            &mut dyn Storage,
            PriceCurrencies,
            BaseCurrency,
            BaseCurrencies,
        > = Oracle::load(storage).unwrap();

        let alarms: u32 = oracle.try_notify_alarms(NOW, 16).unwrap().0;

        assert_eq!(alarms, expected_count);
    }

    #[track_caller]
    fn deliver(storage: &mut dyn Storage, count: u32) {
        let mut alarms: MarketAlarms<'_, &mut dyn Storage, PriceCurrencies> =
            MarketAlarms::new(storage);

        for _ in 0..count {
            alarms.last_delivered().unwrap();
        }
    }

    #[track_caller]
    fn dispatch_and_deliver(storage: &mut dyn Storage, expected_count: u32) {
        dispatch(storage, expected_count);

        deliver(storage, expected_count)
    }
}
