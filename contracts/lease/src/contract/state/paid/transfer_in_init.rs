use serde::{Deserialize, Serialize};

use platform::batch::Batch;
use sdk::cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, QuerierWrapper, Timestamp};

use crate::{
    api::{dex::ConnectionParams, paid::ClosingTrx, ExecuteMsg, StateResponse},
    contract::{
        dex::{self, DexConnectable},
        state::{
            self, controller, ica_connector::Enterable, ica_post_connector::Postpone, Controller,
            Response,
        },
        Contract, Lease,
    },
    error::ContractResult,
    event::Type,
};

use super::transfer_in_finish::TransferInFinish;

#[derive(Serialize, Deserialize)]
pub struct TransferInInit {
    lease: Lease,
}

impl TransferInInit {
    pub(in crate::contract::state) fn new(lease: Lease) -> Self {
        Self { lease }
    }

    pub(super) fn enter(&self, now: Timestamp) -> ContractResult<Batch> {
        let mut sender = self.lease.dex.transfer_from(now);
        sender.send(&self.lease.lease.amount)?;
        Ok(sender.into())
    }

    fn on_response(self, env: &Env, querier: &QuerierWrapper<'_>) -> ContractResult<Response> {
        let finish = TransferInFinish::new(self.lease, env.block.time + dex::IBC_TIMEOUT);
        finish.try_complete(querier, env)
    }
}

impl DexConnectable for TransferInInit {
    fn dex(&self) -> &ConnectionParams {
        self.lease.dex()
    }
}

impl Enterable for TransferInInit {
    fn enter(&self, _deps: Deps<'_>, env: &Env) -> ContractResult<Batch> {
        self.enter(env.block.time)
    }
}

impl Controller for TransferInInit {
    fn execute(
        self,
        deps: &mut DepsMut<'_>,
        env: Env,
        _info: MessageInfo,
        msg: ExecuteMsg,
    ) -> ContractResult<Response> {
        match msg {
            ExecuteMsg::Repay() => controller::err("repay", deps.api),
            ExecuteMsg::Close() => controller::err("close", deps.api),
            ExecuteMsg::PriceAlarm() | ExecuteMsg::TimeAlarm {} => {
                state::ignore_msg(self)?.attach_alarm_response(&env)
            }
        }
    }

    fn on_response(self, _data: Binary, deps: Deps<'_>, env: Env) -> ContractResult<Response> {
        self.on_response(&env, &deps.querier)
    }

    fn on_timeout(self, _deps: Deps<'_>, env: Env) -> ContractResult<Response> {
        state::on_timeout_repair_channel(self, Type::ClosingTransferIn, env)
    }
}

impl Contract for TransferInInit {
    fn state(
        self,
        _now: Timestamp,
        _querier: &QuerierWrapper<'_>,
    ) -> ContractResult<StateResponse> {
        Ok(StateResponse::Paid {
            amount: self.lease.lease.amount,
            in_progress: Some(ClosingTrx::TransferInInit),
        })
    }
}

impl Postpone for TransferInInit {
    fn setup_alarm(&self, when: Timestamp, _querier: &QuerierWrapper<'_>) -> ContractResult<Batch> {
        let time_alarms = self.lease.lease.time_alarms.clone();
        time_alarms.setup_alarm(when).map_err(Into::into)
    }
}
