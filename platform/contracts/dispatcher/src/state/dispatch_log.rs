use serde::{Deserialize, Serialize};

use sdk::{
    cosmwasm_std::{StdResult, Storage, Timestamp},
    cw_storage_plus::Item,
    schemars::{self, JsonSchema},
};

use crate::ContractError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct DispatchLog {
    pub last_dispatch: Timestamp,
}

impl DispatchLog {
    const STORAGE: Item<'static, Self> = Item::new("dispatch_log");

    pub fn new(last_dispatch: Timestamp) -> Self {
        DispatchLog { last_dispatch }
    }

    pub fn last_dispatch(storage: &dyn Storage) -> StdResult<Timestamp> {
        match Self::STORAGE.load(storage) {
            Ok(l) => Ok(l.last_dispatch),
            Err(_) => Ok(Timestamp::default()),
        }
    }

    pub fn update(
        storage: &mut dyn Storage,
        current_dispatch: Timestamp,
    ) -> Result<(), ContractError> {
        match Self::STORAGE.may_load(storage)? {
            None => Self::STORAGE.save(
                storage,
                &DispatchLog {
                    last_dispatch: current_dispatch,
                },
            )?,
            Some(l) => {
                if current_dispatch < l.last_dispatch {
                    return Err(ContractError::InvalidTimeConfiguration {});
                }
                Self::STORAGE.update(storage, |mut log| -> Result<DispatchLog, ContractError> {
                    log.last_dispatch = current_dispatch;
                    Ok(log)
                })?;
            }
        }

        Ok(())
    }
}
