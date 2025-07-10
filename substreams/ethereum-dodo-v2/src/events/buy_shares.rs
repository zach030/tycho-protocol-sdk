use substreams_ethereum::pb::eth::v2::StorageChange;
use tycho_substreams::models::{Attribute, BalanceDelta, Transaction};
use crate::abi::GSP::events::BuyShares;
use crate::events::EventTrait;
use crate::pb::dodo::v2::Pool;

impl EventTrait for BuyShares{
    fn get_changed_attributes(&self, storage_changes: &[StorageChange], pool_address: &[u8; 20]) -> Vec<Attribute> {
        todo!()
    }

    fn get_balance_delta(&self, tx: &Transaction, pool: &Pool, ordinal: u64) -> Vec<BalanceDelta> {
        todo!()
    }
}