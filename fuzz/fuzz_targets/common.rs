use byteorder::{ByteOrder, LE};
use highway::Key;

pub fn split_with_key(data: &[u8]) -> (Key, &[u8]) {
    let mut key_data = [0u8; 32];
    let mut rd: &[u8] = &[];
    if data.len() >= 32 {
        key_data.copy_from_slice(&data[..32]);
        rd = &data[32..];
    } else {
        key_data[..data.len()].copy_from_slice(&data[..data.len()]);
    }

    let mut true_key = [0u64; 4];
    LE::read_u64_into(&key_data, &mut true_key);
    let key = Key(true_key);
    (key, rd)
}
