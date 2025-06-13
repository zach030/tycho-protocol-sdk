use std::str::FromStr;

use ethabi::ethereum_types::Address;
use substreams::prelude::BigInt;
use substreams_ethereum::pb::eth::v2::{self as eth};
use substreams_helper::{event_handler::EventHandler, hex::Hexable};
use crate::abi::factory::events::PoolCreated;
use tycho_substreams::prelude::*;

#[substreams::handlers::map]
pub fn map_pools_created(
    params: String,
    block: eth::Block,
) -> Result<BlockChanges, substreams::errors::Error> {
    let mut new_pools: Vec<TransactionChanges> = vec![];

    let factory_address = params.as_str();

    get_pools(&block, &mut new_pools, factory_address);

    let tycho_block: Block = (&block).into();

    Ok(BlockChanges { block: Some(tycho_block), changes: new_pools })
}

fn get_pools(block: &eth::Block, new_pools: &mut Vec<TransactionChanges>, factory_address: &str) {
    // Extract new pools from PairCreated events
    let mut on_pair_created = |event: PoolCreated, _tx: &eth::TransactionTrace, _log: &eth::Log| {
        let tycho_tx: Transaction = _tx.into();

        new_pools.push(TransactionChanges {
            tx: Some(tycho_tx.clone()),
            contract_changes: vec![],
            entity_changes: vec![EntityChanges {
                component_id: event.pool.to_hex(),
                attributes: vec![
                    Attribute {
                        name: "reserve0".to_string(),
                        value: BigInt::from(0).to_signed_bytes_be(),
                        change: ChangeType::Creation.into(),
                    },
                    Attribute {
                        name: "reserve1".to_string(),
                        value: BigInt::from(0).to_signed_bytes_be(),
                        change: ChangeType::Creation.into(),
                    },
                ],
            }],
            component_changes: vec![ProtocolComponent {
                id: event.pool.to_hex(),
                tokens: vec![event.token0.clone(), event.token1.clone()],
                contracts: vec![],
                static_att: vec![
                    Attribute {
                        name: "stable".to_string(),
                        value: vec![event.stable as u8],
                        change: ChangeType::Creation.into(),
                    },
                    Attribute {
                        name: "pool_address".to_string(),
                        value: event.pool.clone(),
                        change: ChangeType::Creation.into(),
                    },
                ],
                change: i32::from(ChangeType::Creation),
                protocol_type: Some(ProtocolType {
                    name: "aerodrome_v1_pool".into(),
                    financial_type: FinancialType::Swap.into(),
                    attribute_schema: vec![],
                    implementation_type: ImplementationType::Custom.into(),
                }),
                tx: Some(tycho_tx),
            }],
            balance_changes: vec![
                BalanceChange {
                    token: event.token0,
                    balance: BigInt::from(0).to_signed_bytes_be(),
                    component_id: event.pool.to_hex().as_bytes().to_vec(),
                },
                BalanceChange {
                    token: event.token1,
                    balance: BigInt::from(0).to_signed_bytes_be(),
                    component_id: event.pool.to_hex().as_bytes().to_vec(),
                },
            ],
        })
    };

    let mut eh = EventHandler::new(block);

    eh.filter_by_address(vec![Address::from_str(factory_address).unwrap()]);

    eh.on::<PoolCreated, _>(&mut on_pair_created);
    eh.handle_events();
}
