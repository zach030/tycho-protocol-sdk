use hex_literal::hex;

use super::pool_storage::StorageLocation;

const SLOT0: [u8; 32] = hex!("0000000000000000000000000000000000000000000000000000000000000000");

const LIQUIDITY_SLOT: StorageLocation = StorageLocation {
    name: "liquidity",
    slot: hex!("0000000000000000000000000000000000000000000000000000000000000004"),
    offset: 0,
    number_of_bytes: 16,
    signed: false,
};
