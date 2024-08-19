use enum_dispatch::enum_dispatch;

use platform::state_machine::Response as StateMachineResponse;
use sdk::cosmwasm_std::{Env, MessageInfo, QuerierWrapper, Reply, Timestamp};

use crate::{
    api::{position::PositionClose, query::StateResponse},
    error::{ContractError, ContractResult},
};

use super::State;

pub(crate) type Response = StateMachineResponse<State>;
#[enum_dispatch]
pub(crate) trait Handler
where
    Self: Sized,
{
    fn state(self, now: Timestamp, querier: QuerierWrapper<'_>) -> ContractResult<StateResponse>;

    fn reply(
        self,
        _querier: QuerierWrapper<'_>,
        _env: Env,
        _msg: Reply,
    ) -> ContractResult<Response> {
        err("reply")
    }

    fn repay(
        self,
        _querier: QuerierWrapper<'_>,
        _env: Env,
        _info: MessageInfo,
    ) -> ContractResult<Response> {
        err("repay")
    }

    fn close_position(
        self,
        _spec: PositionClose,
        _querier: QuerierWrapper<'_>,
        _env: Env,
        _info: MessageInfo,
    ) -> ContractResult<Response> {
        err("close position")
    }

    fn close(
        self,
        _querier: QuerierWrapper<'_>,
        _env: Env,
        _info: MessageInfo,
    ) -> ContractResult<Response> {
        err("close")
    }

    fn on_time_alarm(
        self,
        _querier: QuerierWrapper<'_>,
        _env: Env,
        _info: MessageInfo,
    ) -> ContractResult<Response> {
        err("on time alarm")
    }

    fn on_price_alarm(
        self,
        _querier: QuerierWrapper<'_>,
        _env: Env,
        _info: MessageInfo,
    ) -> ContractResult<Response> {
        err("on price alarm")
    }

    fn heal(
        self,
        _querier: QuerierWrapper<'_>,
        _env: Env,
        _info: MessageInfo,
    ) -> ContractResult<Response> {
        err("heal")
    }
}

fn err<R>(op: &str) -> ContractResult<R> {
    Err(ContractError::unsupported_operation(op))
}
