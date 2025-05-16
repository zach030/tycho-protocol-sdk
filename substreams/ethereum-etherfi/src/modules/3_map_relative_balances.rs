use crate::{
    abi,
    modules::consts::{
        ADDRESS_ZERO, EETH_ADDRESS, ETH_ADDRESS, LIQUIDITY_POOL_ADDRESS, WEETH_ADDRESS,
    },
};
use anyhow::{Ok, Result};
use substreams::store::{StoreGet, StoreGetString};
use substreams_ethereum::{block_view::LogView, pb::eth::v2::Block, Event};
use tycho_substreams::prelude::*;

#[substreams::handlers::map]
pub fn map_relative_balances(
    block: Block,
    store: StoreGetString,
) -> Result<BlockBalanceDeltas, anyhow::Error> {
    let lp_hex = format!("0x{}", hex::encode(LIQUIDITY_POOL_ADDRESS));
    let eeth_hex = format!("0x{}", hex::encode(EETH_ADDRESS));
    let weeth_hex = format!("0x{}", hex::encode(WEETH_ADDRESS));
    let balance_deltas = block
        .logs()
        .flat_map(|log| match hex::encode(log.address()) {
            addr if addr == lp_hex => handle_liquidity_pool(&log, &store),
            addr if addr == eeth_hex => handle_eeth(&log, &store),
            addr if addr == weeth_hex => handle_weeth(&log, &store),
            _ => vec![],
        })
        .collect();

    Ok(BlockBalanceDeltas { balance_deltas })
}

fn handle_liquidity_pool(log: &LogView, store: &StoreGetString) -> Vec<BalanceDelta> {
    let mut deltas = Vec::new();
    let pool_key = format!("pool:0x{}", hex::encode(LIQUIDITY_POOL_ADDRESS));

    if let Some(ev) = abi::liquidity_pool::events::Deposit::match_and_decode(log) {
        if let Some(cid) = store.get_last(pool_key.clone()) {
            substreams::log::info!("Liquidity Pool Deposit: +ETH {}", ev.amount);
            deltas.push(BalanceDelta {
                ord: log.ordinal(),
                tx: Some(log.receipt.transaction.into()),
                token: ETH_ADDRESS.to_vec(),
                delta: ev.amount.to_signed_bytes_be(),
                component_id: cid.as_bytes().to_vec(),
            });
        }
    } else if let Some(ev) = abi::liquidity_pool::events::Withdraw::match_and_decode(log.log) {
        if let Some(cid) = store.get_last(pool_key) {
            substreams::log::info!("Liquidity Pool Withdraw: -ETH {}", ev.amount);
            deltas.push(BalanceDelta {
                ord: log.ordinal(),
                tx: Some(log.receipt.transaction.into()),
                token: ETH_ADDRESS.to_vec(),
                delta: ev.amount.neg().to_signed_bytes_be(),
                component_id: cid.as_bytes().to_vec(),
            });
        }
    }
    deltas
}

fn handle_eeth(log: &LogView, store: &StoreGetString) -> Vec<BalanceDelta> {
    let mut deltas = Vec::new();
    let pool_key = format!("pool:0x{}", hex::encode(LIQUIDITY_POOL_ADDRESS));
    let weeth_key = format!("pool:0x{}", hex::encode(WEETH_ADDRESS));

    if let Some(ev) = abi::eeth::events::TransferShares::match_and_decode(log.log) {
        if let Some(cid) = store.get_last(pool_key.clone()) {
            if ev.from == ADDRESS_ZERO {
                substreams::log::info!("Liquidity Pool Deposit eETH: +eETH {}", ev.shares_value);
                deltas.push(BalanceDelta {
                    ord: log.ordinal(),
                    tx: Some(log.receipt.transaction.into()),
                    token: EETH_ADDRESS.to_vec(),
                    delta: ev.shares_value.to_signed_bytes_be(),
                    component_id: cid.as_bytes().to_vec(),
                });
            } else if ev.to == ADDRESS_ZERO {
                substreams::log::info!("Liquidity Pool Withdraw eETH: -eETH {}", ev.shares_value);
                deltas.push(BalanceDelta {
                    ord: log.ordinal(),
                    tx: Some(log.receipt.transaction.into()),
                    token: EETH_ADDRESS.to_vec(),
                    delta: ev
                        .shares_value
                        .neg()
                        .to_signed_bytes_be(),
                    component_id: cid.as_bytes().to_vec(),
                });
            }
        }
        if let Some(cid) = store.get_last(weeth_key) {
            if ev.to == WEETH_ADDRESS {
                substreams::log::info!("Deposit eETH into weETH: +eETH {}", ev.shares_value);
                deltas.push(BalanceDelta {
                    ord: log.ordinal(),
                    tx: Some(log.receipt.transaction.into()),
                    token: EETH_ADDRESS.to_vec(),
                    delta: ev.shares_value.to_signed_bytes_be(),
                    component_id: cid.as_bytes().to_vec(),
                });
            } else if ev.from == WEETH_ADDRESS {
                substreams::log::info!("Withdraw eETH from weETH: -eETH {}", ev.shares_value);
                deltas.push(BalanceDelta {
                    ord: log.ordinal(),
                    tx: Some(log.receipt.transaction.into()),
                    token: EETH_ADDRESS.to_vec(),
                    delta: ev
                        .shares_value
                        .neg()
                        .to_signed_bytes_be(),
                    component_id: cid.as_bytes().to_vec(),
                });
            }
        }
    }
    deltas
}

fn handle_weeth(log: &LogView, store: &StoreGetString) -> Vec<BalanceDelta> {
    let mut deltas = Vec::new();
    let weeth_key = format!("pool:0x{}", hex::encode(WEETH_ADDRESS));

    if let Some(ev) = tycho_substreams::abi::erc20::events::Transfer::match_and_decode(log.log) {
        if let Some(cid) = store.get_last(weeth_key) {
            if ev.from == ADDRESS_ZERO {
                substreams::log::info!("Mint weETH: +weETH {}", ev.value);
                deltas.push(BalanceDelta {
                    ord: log.ordinal(),
                    tx: Some(log.receipt.transaction.into()),
                    token: WEETH_ADDRESS.to_vec(),
                    delta: ev.value.to_signed_bytes_be(),
                    component_id: cid.as_bytes().to_vec(),
                });
            } else if ev.to == ADDRESS_ZERO {
                substreams::log::info!("Burn weETH: -weETH {}", ev.value);
                deltas.push(BalanceDelta {
                    ord: log.ordinal(),
                    tx: Some(log.receipt.transaction.into()),
                    token: WEETH_ADDRESS.to_vec(),
                    delta: ev.value.neg().to_signed_bytes_be(),
                    component_id: cid.as_bytes().to_vec(),
                });
            }
        }
    }
    deltas
}
