use serde::{Deserialize, Serialize};

use platform::batch::Batch;
use sdk::{
    cosmwasm_std::{Deps, DepsMut, Env, QuerierWrapper, Timestamp},
    neutron_sdk::sudo::msg::SudoMsg,
};

use crate::{
    api::{opened::RepayTrx, LpnCoin, PaymentCoin, StateQuery, StateResponse},
    contract::{
        state::{opened::repay, Controller, Response},
        Lease,
    },
    error::ContractResult,
};

use super::transfer_in_finish::TransferInFinish;

#[derive(Serialize, Deserialize)]
pub struct TransferInInit {
    pub(super) lease: Lease,
    pub(super) payment: PaymentCoin,
    pub(super) payment_lpn: LpnCoin,
}

impl TransferInInit {
    pub(in crate::contract::state) fn new(
        lease: Lease,
        payment: PaymentCoin,
        payment_lpn: LpnCoin,
    ) -> Self {
        Self {
            lease,
            payment,
            payment_lpn,
        }
    }

    pub(in crate::contract::state::opened) fn enter_state(
        &self,
        now: Timestamp,
    ) -> ContractResult<Batch> {
        let mut sender = self.lease.dex.transfer_from(now);
        sender.send(&self.payment_lpn)?;
        Ok(sender.into())
    }

    fn on_response(self, querier: &QuerierWrapper<'_>, env: &Env) -> ContractResult<Response> {
        TransferInFinish::from(self).try_complete(querier, env)
    }
}

impl Controller for TransferInInit {
    fn sudo(self, deps: &mut DepsMut<'_>, env: Env, msg: SudoMsg) -> ContractResult<Response> {
        match msg {
            SudoMsg::Response {
                request: _,
                data: _,
            } => self.on_response(&deps.querier, &env),
            SudoMsg::Timeout { request: _ } => todo!(),
            SudoMsg::Error {
                request: _,
                details: _,
            } => todo!(),
            _ => unreachable!(),
        }
    }

    fn query(self, deps: Deps<'_>, env: Env, _msg: StateQuery) -> ContractResult<StateResponse> {
        repay::query(
            self.lease.lease,
            self.payment,
            RepayTrx::TransferInInit,
            &deps,
            &env,
        )
    }
}