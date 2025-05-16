use anyhow::{Ok, Result};
use substreams::{store::StoreGetString};
use substreams_ethereum::pb::eth::v2::Block;
use tycho_substreams::prelude::*;

#[substreams::handlers::map]
pub fn map_relative_balances(
    block: Block,
    store: StoreGetString,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    Ok(BlockBalanceDeltas::default())
}