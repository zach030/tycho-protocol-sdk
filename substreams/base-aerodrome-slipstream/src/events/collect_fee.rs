use substreams_ethereum::pb::eth::v2::StorageChange;
use substreams_helper::storage_change::StorageChangesFilter;

use super::{BalanceDelta, EventTrait};
use crate::{
    abi::clpool::events::CollectFees,
    pb::aerodrome::slipstream::Pool,
    storage::{constants::TRACKED_SLOTS, pool_storage::UniswapPoolStorage},
};
use tycho_substreams::prelude::Attribute;

impl EventTrait for CollectFees {
    fn get_changed_attributes(
        &self,
        storage_changes: &[StorageChange],
        pool_address: &[u8; 20],
    ) -> Vec<Attribute> {
        let storage_vec = storage_changes.to_vec();

        let filtered_storage_changes = storage_vec
            .filter_by_address(pool_address)
            .into_iter()
            .cloned()
            .collect();

        let pool_storage = UniswapPoolStorage::new(&filtered_storage_changes);

        pool_storage.get_changed_attributes(TRACKED_SLOTS.to_vec().iter().collect())
    }

    fn get_balance_delta(&self, pool: &Pool, ordinal: u64) -> Vec<BalanceDelta> {
        let changed_balance = vec![
            BalanceDelta {
                token_address: pool.token0.clone(),
                amount: self.amount0.clone().to_bytes_le().1,
                sign: false,
                pool_address: pool.address.clone(),
                ordinal,
            },
            BalanceDelta {
                token_address: pool.token1.clone(),
                amount: self.amount1.clone().to_bytes_le().1,
                sign: false,
                pool_address: pool.address.clone(),
                ordinal,
            },
        ];
        changed_balance
    }
}
