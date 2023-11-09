use serde::{Deserialize, Serialize};

use sdk::{
    cosmwasm_std::{Addr, StdResult, Storage},
    cw_storage_plus::Item,
};

use crate::{error::ContractError, msg::Dex, result::ContractResult};

use super::reward_scale::RewardScale;

pub type CadenceHours = u16;

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    // Time duration in hours defining the periods of time this instance is awaken
    pub cadence_hours: CadenceHours,
    // All DEX-es the protocol works with
    pub dexes: Vec<Dex>,
    // address to treasury contract
    pub treasury: Addr,
    // A list of (minTVL_MNLS: u32, APR%o) which defines the APR as per the TVL.
    pub tvl_to_apr: RewardScale,
}

impl Config {
    const STORAGE: Item<'static, Self> = Item::new("dispatcher_config");

    pub fn new(
        cadence_hours: CadenceHours,
        dex: Dex,
        treasury: Addr,
        tvl_to_apr: RewardScale,
    ) -> Self {
        Config {
            cadence_hours,
            dexes: vec![dex],
            tvl_to_apr,
            treasury,
        }
    }

    pub fn store(self, storage: &mut dyn Storage) -> StdResult<()> {
        Self::STORAGE.save(storage, &self)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<Self> {
        Self::STORAGE.load(storage)
    }

    pub fn update_cadence_hours(
        storage: &mut dyn Storage,
        cadence_hours: CadenceHours,
    ) -> ContractResult<()> {
        Self::STORAGE
            .update(storage, |config| -> Result<Config, ContractError> {
                Ok(Self {
                    cadence_hours,
                    ..config
                })
            })
            .map(|_| ())
            .map_err(Into::into)
    }

    pub fn update_tvl_to_apr(
        storage: &mut dyn Storage,
        tvl_to_apr: RewardScale,
    ) -> ContractResult<()> {
        Self::STORAGE
            .update(storage, |config| -> Result<Config, ContractError> {
                Ok(Self {
                    tvl_to_apr,
                    ..config
                })
            })
            .map(|_| ())
            .map_err(Into::into)
    }

    pub fn add_dex(storage: &mut dyn Storage, dex: Dex) -> ContractResult<()> {
        Self::STORAGE
            .update(storage, |mut config| -> Result<Config, ContractError> {
                config.dexes.push(dex);
                Ok(config)
            })
            .map(|_| ())
            .map_err(Into::into)
    }
}

#[cfg(feature = "migration")]
pub(crate) mod migration {
    use serde::{Deserialize, Serialize, Serializer};

    use sdk::{
        cosmwasm_std::{Addr, Storage},
        cw_storage_plus::Item,
    };

    use crate::{
        msg::Dex,
        result::ContractResult,
        state::{reward_scale::RewardScale, CadenceHours, Config},
    };

    const STORAGE: Item<'static, OldConfig> = Item::new("dispatcher_config");

    #[derive(Deserialize)]
    struct OldConfig {
        cadence_hours: CadenceHours,
        lpp: Addr,
        treasury: Addr,
        oracle: Addr,
        tvl_to_apr: RewardScale,
    }

    impl Serialize for OldConfig {
        fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            unimplemented!("Required by `cw_storage_plus::Item::load`'s trait bounds.")
        }
    }

    impl From<OldConfig> for Config {
        fn from(value: OldConfig) -> Self {
            Config::new(
                value.cadence_hours,
                Dex {
                    lpp: value.lpp,
                    oracle: value.oracle,
                },
                value.treasury,
                value.tvl_to_apr,
            )
        }
    }

    pub fn migrate(storage: &mut dyn Storage) -> ContractResult<()> {
        STORAGE
            .load(storage)
            .map(Into::into)
            .and_then(|config: Config| config.store(storage))
            .map_err(Into::into)
    }
}