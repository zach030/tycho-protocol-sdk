use crate::{pb::dodo::v2::Pool, storage::pool_storage::DoDoPoolStorage};
use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{pb::substreams::StoreDeltas, prelude::StoreGetProto, store::StoreGet};
use substreams_ethereum::pb::eth::v2::{Block, StorageChange};
use substreams_helper::{hex::Hexable, storage_change::StorageChangesFilter};
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};

#[substreams::handlers::map]
pub fn map_protocol_changes(
    block: Block,
    protocol_components: BlockTransactionProtocolComponents,
    balance_deltas: BlockBalanceDeltas,
    pool_store: StoreGetProto<Pool>,
    balance_store: StoreDeltas,
) -> Result<BlockChanges> {
    let mut transaction_changes: HashMap<_, TransactionChangesBuilder> = HashMap::new();

    protocol_components
        .tx_components
        .iter()
        .for_each(|tx_component| {
            let tx = tx_component.tx.as_ref().unwrap();
            let builder = transaction_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionChangesBuilder::new(tx));

            tx_component
                .components
                .iter()
                .for_each(|c| {
                    builder.add_protocol_component(c);
                });
        });

    for transaction in block.transactions() {
        let tx = Transaction {
            to: transaction.to.clone(),
            from: transaction.from.clone(),
            hash: transaction.hash.clone(),
            index: transaction.index.into(),
        };
        let builder = transaction_changes
            .entry(tx.index)
            .or_insert_with(|| TransactionChangesBuilder::new(&tx));

        for (log, call_view) in transaction.logs_with_calls() {
            // Skip if the log is not from a known dodo pool.
            if let Some(pool) = pool_store.get_last(format!("Pool:{}", &log.address.to_hex())) {
                let changed_attributes =
                    decode_event_changed_attributes(&call_view.call.storage_changes, &pool);

                let entity_change = EntityChanges {
                    component_id: pool.address.clone().to_hex(),
                    attributes: changed_attributes,
                };

                builder.add_entity_change(&entity_change);
            }
        }
    }

    aggregate_balances_changes(balance_store, balance_deltas)
        .into_iter()
        .for_each(|(_, (tx, balances))| {
            let builder = transaction_changes
                .entry(tx.index)
                .or_insert_with(|| TransactionChangesBuilder::new(&tx));
            balances
                .values()
                .for_each(|token_bc_map| {
                    token_bc_map
                        .values()
                        .for_each(|bc| builder.add_balance_change(bc))
                });
        });

    extract_contract_changes_builder(
        &block,
        |addr| {
            pool_store
                .get_last(format!("Pool:0x{}", hex::encode(addr)))
                .is_some()
        },
        &mut transaction_changes,
    );

    transaction_changes
        .iter_mut()
        .for_each(|(_, change)| {
            // this indirection is necessary due to borrowing rules.
            let addresses = change
                .changed_contracts()
                .map(|e| e.to_vec())
                .collect::<Vec<_>>();
            addresses
                .into_iter()
                .for_each(|address| {
                    let pool = pool_store
                        .get_last(format!("Pool:0x{}", hex::encode(address)))
                        .unwrap();
                    change.mark_component_as_updated(&pool.address.to_hex());
                })
        });

    Ok(BlockChanges {
        block: Some((&block).into()),
        changes: transaction_changes
            .drain()
            .sorted_unstable_by_key(|(index, _)| *index)
            .filter_map(|(_, builder)| builder.build())
            .collect::<Vec<_>>(),
    })
}

fn decode_event_changed_attributes(
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

    let pool_type = String::from_utf8(pool.pool_type.clone()).expect("pool type is invalid UTF-8");

    let pool_storage = DoDoPoolStorage::new(pool_type.as_str(), &filtered_storage_changes);

    pool_storage.get_changed_attributes()
}
