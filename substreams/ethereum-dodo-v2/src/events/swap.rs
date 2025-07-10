use num_bigint::Sign;
use substreams::prelude::BigInt;
use substreams_ethereum::pb::eth::v2::StorageChange;
use substreams_helper::hex::Hexable;
use tycho_substreams::models::{Attribute, BalanceDelta, Transaction};
use crate::abi::GSP::events::DodoSwap;
use crate::events::EventTrait;
use crate::pb::dodo::v2::Pool;

impl EventTrait for DodoSwap{
    fn get_changed_attributes(&self, storage_changes: &[StorageChange], pool_address: &[u8; 20]) -> Vec<Attribute> {
        todo!()
    }

    fn get_balance_delta(&self, tx: &Transaction, pool: &Pool, ordinal: u64) -> Vec<BalanceDelta> {
        let (token_in, token_out, amount_in, amount_out) = if self.from_token == pool.base_token {
            (&pool.base_token, &pool.quote_token, &self.from_amount, &self.to_amount)
        } else {
            (&pool.quote_token, &pool.base_token, &self.from_amount, &self.to_amount)
        };
        vec![
            BalanceDelta {
                ord: ordinal,
                tx: Some(tx.clone()),
                token: token_in.clone(),
                delta: amount_in.to_signed_bytes_be(),
                component_id: pool
                    .address
                    .clone()
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            },
            BalanceDelta {
                ord: ordinal,
                tx: Some(tx.clone()),
                token: token_out.clone(),
                delta: amount_out.neg().to_signed_bytes_be(),
                component_id: pool
                    .address
                    .clone()
                    .to_hex()
                    .as_bytes()
                    .to_vec(),
            },
        ]
    }
}