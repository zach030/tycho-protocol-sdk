use anyhow::Result;
use itertools::Itertools;
use std::collections::HashMap;
use substreams::{pb::substreams::StoreDeltas, prelude::StoreGetProto, store::StoreGet};
use substreams::store::StoreGetString;
use substreams_ethereum::pb::eth::v2::Block;
use tycho_substreams::{
    balances::aggregate_balances_changes, contract::extract_contract_changes_builder, prelude::*,
};

#[substreams::handlers::map]
pub fn map_protocol_changes(
    params: String,
    block: Block,
    protocol_components: BlockTransactionProtocolComponents,
    balance_deltas: BlockBalanceDeltas,
    pool_store: StoreGetString,
    balance_store: StoreDeltas,
) -> Result<BlockChanges> {
    Ok(BlockChanges::default())
}