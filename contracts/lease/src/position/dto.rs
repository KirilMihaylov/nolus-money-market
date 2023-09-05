use finance::liability::Liability;
use serde::{Deserialize, Serialize};

use crate::api::{LeaseCoin, LpnCoin, PositionSpec};

#[derive(Serialize, Deserialize, Clone)]
#[cfg_attr(test, derive(Debug))]
pub struct PositionDTO {
    pub spec: PositionSpec,
    pub amount: LeaseCoin,
}

impl PositionDTO {
    pub(crate) fn new(
        liability: Liability,
        min_asset: LpnCoin,
        min_sell_asset: LpnCoin,
        amount: LeaseCoin,
    ) -> Self {
        Self {
            spec: PositionSpec::new(liability, min_asset, min_sell_asset),
            amount,
        }
    }
}
