use anyhow::{Ok, Result};
use substreams_ethereum::pb::eth::v2::{Block};
use tycho_substreams::prelude::*;
#[substreams::handlers::map]
pub fn map_components(params: String, block: Block) -> Result<BlockTransactionProtocolComponents> {
    Ok(BlockTransactionProtocolComponents::default())
}