use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{msg::InstantiateMsg, ContractError};
use cosmwasm_std::{Addr, Decimal256, Storage, Uint256};
use cw_storage_plus::{Item, Map};

pub type InstantiateReplyId = u64;

pub const CONFIG: Item<Config> = Item::new("config");
pub const INSTANTIATE_REPLY_IDS: InstantiateReplyIdSeq =
    InstantiateReplyIdSeq::new("instantiate_reply_ids");
pub const PENDING_INSTANCE_CREATIONS: Map<InstantiateReplyId, Addr> =
    Map::new("pending_instance_creations");
pub const LOANS: Map<&Addr, Addr> = Map::new("loans");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub lease_code_id: u64,
    pub lpp_ust_addr: Addr,
    pub lease_interest_rate_margin: Decimal256,
    pub lease_max_liability: Decimal256,
    pub lease_healthy_liability: Decimal256,
    pub lease_initial_liability: Decimal256,
    pub repayment_period_nano_sec: Uint256,
    pub grace_period_nano_sec: Uint256,
}

impl Config {
    pub fn new(sender: Addr, msg: InstantiateMsg) -> Result<Self, ContractError> {
        Ok(Config {
            owner: sender,
            lease_code_id: msg.lease_code_id,
            lpp_ust_addr: msg.lpp_ust_addr,
            lease_interest_rate_margin: Decimal256::percent(msg.lease_interest_rate_margin),
            lease_max_liability: Decimal256::percent(msg.lease_max_liability),
            lease_healthy_liability: Config::validate_lease_healthy_liability(
                msg.lease_healthy_liability,
                msg.lease_max_liability,
            )?,
            lease_initial_liability: Config::validate_lease_initial_liability(
                msg.lease_initial_liability,
                msg.lease_healthy_liability,
            )?,
            repayment_period_nano_sec: msg.repayment_period_nano_sec,
            grace_period_nano_sec: msg.grace_period_nano_sec,
        })
    }

    fn validate_lease_healthy_liability(
        lease_healthy_liability: u64,
        lease_max_liability: u64,
    ) -> Result<Decimal256, ContractError> {
        if lease_healthy_liability < lease_max_liability {
            Ok(Decimal256::percent(lease_healthy_liability))
        } else {
            Err(ContractError::ValidationError {})
        }
    }

    fn validate_lease_initial_liability(
        lease_initial_liability: u64,
        lease_healthy_liability: u64,
    ) -> Result<Decimal256, ContractError> {
        if lease_initial_liability <= lease_healthy_liability {
            Ok(Decimal256::percent(lease_initial_liability))
        } else {
            Err(ContractError::ValidationError {})
        }
    }
}

pub struct InstantiateReplyIdSeq<'a>(Item<'a, InstantiateReplyId>);

impl<'a> InstantiateReplyIdSeq<'a> {
    pub const fn new(namespace: &'a str) -> InstantiateReplyIdSeq {
        InstantiateReplyIdSeq(Item::new(namespace))
    }

    pub fn next(&self, store: &mut dyn Storage) -> Result<InstantiateReplyId, ContractError> {
        let mut next_seq = self.0.load(store).unwrap_or(0);
        next_seq += 1;
        self.0.save(store, &next_seq)?;
        Ok(next_seq)
    }
}
