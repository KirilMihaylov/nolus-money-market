use serde::{Deserialize, Serialize};

use dex::{Account, ConnectionParams, DexConnectable};
use sdk::cosmwasm_std::QuerierWrapper;

use crate::lease::{with_lease::WithLease, LeaseDTO};

pub use self::endpoins::{execute, instantiate, migrate, query, reply, sudo};
use self::finalize::FinalizerRef;

mod api;
mod cmd;
mod endpoins;
mod finalize;
pub mod msg;
mod state;
#[cfg(feature = "migration")]
mod v5;

#[derive(Serialize, Deserialize)]
pub(crate) struct Lease {
    lease: LeaseDTO,
    dex: Account,
    finalizer: FinalizerRef,
}

pub(crate) trait SplitDTOOut {
    type Other;

    fn split_into(self) -> (LeaseDTO, Self::Other);
}

impl Lease {
    fn new(lease: LeaseDTO, dex: Account, finalizer: FinalizerRef) -> Self {
        Self {
            lease,
            dex,
            finalizer,
        }
    }

    fn update<Cmd>(
        self,
        cmd: Cmd,
        querier: &QuerierWrapper<'_>,
    ) -> Result<(Self, <Cmd::Output as SplitDTOOut>::Other), Cmd::Error>
    where
        Cmd: WithLease,
        Cmd::Output: SplitDTOOut,
        Cmd::Error: From<lpp::error::ContractError>,
        currency::error::Error: Into<Cmd::Error>,
        timealarms::error::ContractError: Into<Cmd::Error>,
        oracle_platform::error::Error: Into<Cmd::Error>,
    {
        self.lease.execute(cmd, querier).map(|result| {
            let (lease, other) = result.split_into();
            (Self::new(lease, self.dex, self.finalizer), other)
        })
    }
}

impl DexConnectable for Lease {
    fn dex(&self) -> &ConnectionParams {
        self.dex.dex()
    }
}