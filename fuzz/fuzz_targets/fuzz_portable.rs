#![no_main]

mod common;
use highway::{HighwayHash, PortableHash};

libfuzzer_sys::fuzz_target!(|input: common::FuzzKey| {
    let (key, data) = (input.key, &input.data);
    let hash1 = PortableHash::new(key).hash64(data);
    let hash2 = PortableHash::new(key).hash64(data);
    assert_eq!(hash1, hash2);
    PortableHash::new(key).hash128(data);
    PortableHash::new(key).hash256(data);
});
