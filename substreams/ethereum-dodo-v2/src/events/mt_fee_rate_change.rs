use crate::{
    abi::GSP::events::MtFeeRateChange, events::EventTrait, pb::dodo::v2::Pool,
    storage::pool_storage::DoDoPoolStorage,
};
use substreams_ethereum::pb::eth::v2::StorageChange;
use substreams_helper::storage_change::StorageChangesFilter;
use tycho_substreams::models::{Attribute, BalanceDelta, Transaction};

impl EventTrait for MtFeeRateChange {
    fn get_changed_attributes(
        &self,
        storage_changes: &[StorageChange],
        pool: &Pool,
    ) -> Vec<Attribute> {
        let storage_vec = storage_changes.to_vec();
        let filtered_storage_changes = storage_vec
            .filter_by_address(
                pool.address
                    .as_slice()
                    .try_into()
                    .expect("Address is wrong length"),
            )
            .into_iter()
            .cloned()
            .collect();

        let pool_type =
            String::from_utf8(pool.pool_type.clone()).expect("pool type is invalid UTF-8");

        let pool_storage = DoDoPoolStorage::new(pool_type.as_str(), &filtered_storage_changes);

        pool_storage.get_changed_attributes()
    }

    fn get_balance_delta(&self, tx: &Transaction, pool: &Pool, ordinal: u64) -> Vec<BalanceDelta> {
        vec![]
    }
}
