use crate::{
    abi::GSP::events::{BuyShares, DodoSwap, MtFeeRateChange, RChange, SellShares},
    pb::dodo::v2::Pool,
};
use substreams_ethereum::{
    pb::eth::v2::{Log, StorageChange},
    Event,
};
use tycho_substreams::{
    models::Transaction,
    prelude::{Attribute, BalanceDelta},
};

pub mod buy_shares;
pub mod mt_fee_rate_change;
pub mod rchange;
pub mod sell_shares;
pub mod swap;

/// A trait for extracting changed attributes and balance from an event.
pub trait EventTrait {
    /// Get all relevant changed attributes from the `[StorageChange]`.
    /// If an attribute is changed multiple times, only the last state will be returned.
    ///
    /// # Arguments
    ///
    /// * `storage_changes` - A slice of `StorageChange` that indicates the changes in storage.
    /// * `pool` - Reference to the `Pool`.
    ///
    /// # Returns
    ///
    /// A vector of `Attribute` that represents the changed attributes.
    fn get_changed_attributes(
        &self,
        storage_changes: &[StorageChange],
        pool: &Pool,
    ) -> Vec<Attribute>;

    /// Get all balance deltas from the event.
    ///
    /// # Arguments
    ///
    /// * `tx` - Reference to the `Transaction`.
    /// * `pool` - Reference to the `Pool`.
    /// * `ordinal` - The ordinal number of the event. This is used by the balance store to sort the
    ///
    /// # Returns
    ///
    /// A vector of `BalanceDelta` that represents the balance deltas.
    fn get_balance_delta(&self, tx: &Transaction, pool: &Pool, ordinal: u64) -> Vec<BalanceDelta>;
}

/// Represent every events of a Dodo pool.
pub enum EventType {
    DodoSwap(DodoSwap),
    BuyShares(BuyShares),
    SellShares(SellShares),
    RChange(RChange),
    MtFeeRateChange(MtFeeRateChange),
}

impl EventType {
    fn as_event_trait(&self) -> &dyn EventTrait {
        match self {
            EventType::DodoSwap(event) => event,
            EventType::BuyShares(event) => event,
            EventType::SellShares(event) => event,
            EventType::RChange(event) => event,
            EventType::MtFeeRateChange(event) => event,
        }
    }
}

/// Decodes a given log into an `EventType`.
///
/// # Arguments
///
/// * `event` - A reference to the `Log`.
///
/// # Returns
///
/// An `Option<EventType>` that represents the decoded event type.
pub fn decode_event(event: &Log) -> Option<EventType> {
    [
        DodoSwap::match_and_decode(event).map(EventType::DodoSwap),
        BuyShares::match_and_decode(event).map(EventType::BuyShares),
        SellShares::match_and_decode(event).map(EventType::SellShares),
    ]
    .into_iter()
    .find_map(std::convert::identity)
}

/// Gets the changed attributes from the log.
///
/// # Arguments
///
/// * `event` - A reference to the `Log`.
/// * `storage_changes` - A slice of `StorageChange` that indicates the changes in storage.
/// * `pool` - Reference to the `Pool` structure.
///
/// # Returns
///
/// A vector of `Attribute` that represents the changed attributes.
pub fn get_log_changed_attributes(
    event: &Log,
    storage_changes: &[StorageChange],
    pool: &Pool,
) -> Vec<Attribute> {
    decode_event(event)
        .map(|e| {
            e.as_event_trait()
                .get_changed_attributes(storage_changes, pool)
        })
        .unwrap_or_default()
}

/// Gets the changed balances from the log.
///
/// # Arguments
///
/// * `tx` - Reference to the `Transaction`.
/// * `event` - Reference to the `Log`.
/// * `pool` - Reference to the `Pool`.
///
/// # Returns
///
/// A vector of `BalanceDelta` that represents
pub fn get_log_changed_balances(tx: &Transaction, event: &Log, pool: &Pool) -> Vec<BalanceDelta> {
    decode_event(event)
        .map(|e| {
            e.as_event_trait()
                .get_balance_delta(tx, pool, event.ordinal)
        })
        .unwrap_or_default()
}
