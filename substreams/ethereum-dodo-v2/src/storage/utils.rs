use substreams::scalar::BigInt;
use tiny_keccak::{Hasher, Keccak};

pub fn calc_map_slot(map_index: &[u8; 32], base_slot: &[u8; 32]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let mut hasher = Keccak::v256();
    hasher.update(map_index);
    hasher.update(base_slot);
    hasher.finalize(&mut output);
    output
}

pub fn left_pad_from_bigint(input: &BigInt) -> [u8; 32] {
    if input.lt(&BigInt::zero()) {
        return left_pad(&input.to_signed_bytes_be(), 255);
    }

    left_pad(&input.to_signed_bytes_be(), 0)
}

pub fn left_pad(input: &[u8], padding_value: u8) -> [u8; 32] {
    if input.len() > 32 {
        panic!("cannot convert vec<u8> to H256");
    }
    let mut data = [padding_value; 32];
    let offset = 32 - input.len();
    data[offset..(input.len() + offset)].copy_from_slice(input);

    data
}

pub fn read_bytes(buf: &[u8], offset: usize, number_of_bytes: usize) -> &[u8] {
    let buf_length = buf.len();
    if buf_length < number_of_bytes {
        panic!(
            "attempting to read {number_of_bytes} bytes in buffer  size {buf_size}",
            number_of_bytes = number_of_bytes,
            buf_size = buf.len()
        )
    }

    if offset > (buf_length - 1) {
        panic!(
            "offset {offset} exceeds buffer size {buf_size}",
            offset = offset,
            buf_size = buf.len()
        )
    }

    let end = buf_length - 1 - offset;
    let start_opt = (end + 1).checked_sub(number_of_bytes);
    if start_opt.is_none() {
        panic!(
            "number of bytes {number_of_bytes} with offset {offset} exceeds buffer size
{buf_size}",
            number_of_bytes = number_of_bytes,
            offset = offset,
            buf_size = buf.len()
        )
    }
    let start = start_opt.unwrap();

    &buf[start..=end]
}