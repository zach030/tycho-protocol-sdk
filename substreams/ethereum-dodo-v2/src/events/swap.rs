use crate::{
    abi::GSP::events::DodoSwap, events::EventTrait, pb::dodo::v2::Pool,
    storage::pool_storage::DoDoPoolStorage,
};
use substreams_ethereum::pb::eth::v2::StorageChange;
use substreams_helper::{hex::Hexable, storage_change::StorageChangesFilter};
use tycho_substreams::models::{Attribute, BalanceDelta, Transaction};

impl EventTrait for DodoSwap {
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
        let (token_in, token_out, amount_in, amount_out) = if self.from_token == pool.base_token {
            (&pool.base_token, &pool.quote_token, &self.from_amount, &self.to_amount)
        } else {
            (&pool.quote_token, &pool.base_token, &self.from_amount, &self.to_amount)
        };
        vec![
            BalanceDelta {
                ord: ordinal,
                tx: Some(tx.clone()),
                token: token_in.clone(),
                delta: amount_in.to_signed_bytes_be(),
                component_id: pool
                    .address
                    .clone()
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            },
            BalanceDelta {
                ord: ordinal,
                tx: Some(tx.clone()),
                token: token_out.clone(),
                delta: amount_out.neg().to_signed_bytes_be(),
                component_id: pool
                    .address
                    .clone()
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            },
        ]
    }
}
