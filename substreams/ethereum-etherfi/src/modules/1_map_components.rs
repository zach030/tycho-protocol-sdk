use crate::modules::consts::{
    EETH_ADDRESS, ETH_ADDRESS, LIQUIDITY_POOL_ADDRESS, LIQUIDITY_POOL_CREATION_HASH, WEETH_ADDRESS,
    WEETH_CREATION_HASH,
};
use anyhow::{Ok, Result};
use substreams_ethereum::pb::eth::v2::Block;
use tycho_substreams::prelude::*;

#[substreams::handlers::map]
pub fn map_components(block: Block) -> Result<BlockTransactionProtocolComponents> {
    Ok(BlockTransactionProtocolComponents {
        tx_components: block
            .transactions()
            .filter_map(|tx| {
                let mut components: Vec<ProtocolComponent> = vec![];
                if tx.hash == LIQUIDITY_POOL_CREATION_HASH {
                    components.push(
                        ProtocolComponent::at_contract(&LIQUIDITY_POOL_ADDRESS)
                            .with_tokens(&[ETH_ADDRESS, EETH_ADDRESS])
                            .as_swap_type("etherfi_liquidity_pool", ImplementationType::Vm),
                    )
                } else if tx.hash == WEETH_CREATION_HASH {
                    components.push(
                        ProtocolComponent::at_contract(&WEETH_ADDRESS)
                            .with_tokens(&[EETH_ADDRESS, WEETH_ADDRESS])
                            .as_swap_type("etherfi_weeth", ImplementationType::Vm),
                    )
                }
                if !components.is_empty() {
                    Some(TransactionProtocolComponents { tx: Some(tx.into()), components })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
    })
}
