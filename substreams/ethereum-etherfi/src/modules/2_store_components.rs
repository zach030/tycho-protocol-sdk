use substreams::{
    prelude::{StoreSet, StoreSetString},
    store::StoreNew,
};
use tycho_substreams::prelude::*;

#[substreams::handlers::store]
pub fn store_components(map: BlockTransactionProtocolComponents, store: StoreSetString) {
    map.tx_components
        .iter()
        .for_each(|tx_components| {
            tx_components
                .components
                .iter()
                .for_each(|pc| store.set(0, format!("pool:{0}", &pc.id), &pc.id))
        })
}
