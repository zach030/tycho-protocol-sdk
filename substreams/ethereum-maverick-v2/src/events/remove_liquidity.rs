use crate::{abi::pool::events::PoolRemoveLiquidity, events::EventTrait, pb::maverick::v2::Pool};
use tycho_substreams::prelude::*;

impl EventTrait for PoolRemoveLiquidity {
    fn get_balance_delta(&self, tx: &Transaction, pool: &Pool, ordinal: u64) -> Vec<BalanceDelta> {
        let changed_balance = vec![
            BalanceDelta {
                ord: ordinal,
                tx: Some(tx.clone()),
                token: pool.token_a.clone(),
                delta: self.token_a_out.clone().to_bytes_le().1,
                component_id: pool.address.clone(),
            },
            BalanceDelta {
                ord: ordinal,
                tx: Some(tx.clone()),
                token: pool.token_b.clone(),
                delta: self.token_b_out.clone().to_bytes_le().1,
                component_id: pool.address.clone(),
            },
        ];
        changed_balance
    }
}
