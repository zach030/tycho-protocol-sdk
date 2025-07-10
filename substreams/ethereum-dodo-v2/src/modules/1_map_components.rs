use anyhow::{Ok, Result};
use ethabi::ethereum_types::Address;
use substreams_ethereum::pb::eth::v2::{Block, TransactionTrace};
use tycho_substreams::prelude::*;

use crate::{abi::DPPFactory::events::NewDpp,abi::DSPFactory::events::NewDsp,abi::DVMFactory::events::NewDvm, abi::GSPFactory::events::NewGsp};
use crate::modules::utils::Params;
use substreams_helper::{event_handler::EventHandler, hex::Hexable};

#[substreams::handlers::map]
pub fn map_components(params: String, block: Block) -> Result<BlockTransactionProtocolComponents> {
    let mut new_pools: Vec<TransactionProtocolComponents> = vec![];
    let params = Params::parse_from_query(&params)?;
    get_new_pools(params, &block, &mut new_pools);
    Ok(BlockTransactionProtocolComponents { tx_components: new_pools })
}

fn handle_new_dpp(event: NewDpp, tx: &TransactionTrace) -> TransactionProtocolComponents{
    let tycho_tx: Transaction = tx.into();
    let tokens = [event.base_token.as_slice(), event.quote_token.as_slice()];
    let contracts = [event.dpp.as_slice()];
    let component = ProtocolComponent::new(&event.dpp.to_hex())
        .with_tokens(&tokens)
        .with_contracts(&contracts)
        .with_attributes(&[("pool_type", "dpp")])
        .as_swap_type("dodo_v2_pool", ImplementationType::Custom.into());

    TransactionProtocolComponents {
        tx: Some(tycho_tx),
        components: vec![component],
    }
}

fn handle_new_dsp(event: NewDsp, tx: &TransactionTrace)-> TransactionProtocolComponents {
    let tycho_tx: Transaction = tx.into();
    let tokens = [event.base_token.as_slice(), event.quote_token.as_slice()];
    let contracts = [event.dsp.as_slice()];
    let component = ProtocolComponent::new(&event.dsp.to_hex())
        .with_tokens(&tokens)
        .with_contracts(&contracts)
        .with_attributes(&[("pool_type", "dsp")])
        .as_swap_type("dodo_v2_pool", ImplementationType::Custom.into());

    TransactionProtocolComponents {
        tx: Some(tycho_tx),
        components: vec![component],
    }
}

fn handle_new_dvm(event: NewDvm, tx: &TransactionTrace) ->TransactionProtocolComponents {
    let tycho_tx: Transaction = tx.into();
    let tokens = [event.base_token.as_slice(), event.quote_token.as_slice()];
    let contracts = [event.dvm.as_slice()];
    let component = ProtocolComponent::new(&event.dvm.to_hex())
        .with_tokens(&tokens)
        .with_contracts(&contracts)
        .with_attributes(&[("pool_type", "dvm")])
        .as_swap_type("dodo_v2_pool", ImplementationType::Custom.into());

    TransactionProtocolComponents {
        tx: Some(tycho_tx),
        components: vec![component],
    }
}

fn handle_new_gsp(event: NewGsp, tx: &TransactionTrace) -> TransactionProtocolComponents {
    let tycho_tx: Transaction = tx.into();
    let tokens = [event.base_token.as_slice(), event.quote_token.as_slice()];
    let contracts = [event.gsp.as_slice()];
    let component = ProtocolComponent::new(&event.gsp.to_hex())
        .with_tokens(&tokens)
        .with_contracts(&contracts)
        .with_attributes(&[("pool_type", "gsp")])
        .as_swap_type("dodo_v2_pool", ImplementationType::Custom.into());

    TransactionProtocolComponents {
        tx: Some(tycho_tx),
        components: vec![component],
    }
}

fn get_new_pools(
    params: Params,
    block: &Block,
    new_pools: &mut Vec<TransactionProtocolComponents>,
) {
    let (dpp_factory, dsp_factory, dvm_factory, gsp_factory) = params.decode_addresses().unwrap();

    let mut eh = EventHandler::new(block);
    eh.filter_by_address(vec![
        Address::from_slice(&dpp_factory),
        Address::from_slice(&dsp_factory),
        Address::from_slice(&dvm_factory),
        Address::from_slice(&gsp_factory),
    ]);

    let pools_ptr = new_pools as *mut Vec<TransactionProtocolComponents>;

    eh.on::<NewDpp, _>(move |event, tx, log| {
        if log.address == dpp_factory {
            unsafe {
                (*pools_ptr).push(handle_new_dpp(event, tx));
            }
        }
    });

    eh.on::<NewDsp, _>(move |event, tx, log| {
        if log.address == dsp_factory {
            unsafe {
                (*pools_ptr).push(handle_new_dsp(event, tx));
            }
        }
    });

    eh.on::<NewDvm, _>(move |event, tx, log| {
        if log.address == dvm_factory {
            unsafe {
                (*pools_ptr).push(handle_new_dvm(event, tx));
            }
        }
    });

    eh.on::<NewGsp, _>(move |event, tx, log| {
        if log.address == gsp_factory {
            unsafe {
                (*pools_ptr).push(handle_new_gsp(event, tx));
            }
        }
    });

    eh.handle_events();
}
