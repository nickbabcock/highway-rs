#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate highway;
extern crate byteorder;

mod common;
use highway::{HighwayHash, PortableHash};

fuzz_target!(|data: &[u8]| {
    let (key, data) = common::split_with_key(data);
    let hash1 = PortableHash::new(&key).hash64(data);
    let hash2 = PortableHash::new(&key).hash64(data);
    assert_eq!(hash1, hash2);
    PortableHash::new(&key).hash128(data);
    PortableHash::new(&key).hash256(data);
});
