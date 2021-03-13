#![no_main]

use highway::{AvxHash, HighwayHash, Key, PortableHash, SseHash};
use libfuzzer_sys::arbitrary;

#[derive(Debug, arbitrary::Arbitrary)]
pub struct FuzzKey {
    pub key: [u64; 4],
    pub data: Vec<u8>,
}

libfuzzer_sys::fuzz_target!(|input: FuzzKey| {
    let data = &input.data;
    let key = Key(input.key);
    let mut hashes = [0u64; 2];
    for hash in &mut hashes {
        let portable64 = PortableHash::new(key).hash64(data);
        *hash = portable64;
        let portable128 = PortableHash::new(key).hash128(data);
        let portable256 = PortableHash::new(key).hash256(data);

        if let Some(hash) = AvxHash::new(key).map(|x| x.hash64(data)) {
            assert_eq!(hash, portable64)
        }

        if let Some(hash) = AvxHash::new(key).map(|x| x.hash128(data)) {
            assert_eq!(hash, portable128)
        }

        if let Some(hash) = AvxHash::new(key).map(|x| x.hash256(data)) {
            assert_eq!(hash, portable256)
        }

        if let Some(hash) = SseHash::new(key).map(|x| x.hash64(data)) {
            assert_eq!(hash, portable64)
        }

        if let Some(hash) = SseHash::new(key).map(|x| x.hash128(data)) {
            assert_eq!(hash, portable128)
        }

        if let Some(hash) = SseHash::new(key).map(|x| x.hash256(data)) {
            assert_eq!(hash, portable256)
        }
    }

    assert_eq!(hashes[0], hashes[1]);
});
