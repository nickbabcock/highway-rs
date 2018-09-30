#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate highway;
use highway::{HighwayHash, Key, PortableHash};

fuzz_target!(|data: &[u8]| {
    let key = Key([1, 2, 3, 4]);
    let hash1 = PortableHash::new(&key).hash64(data);
    let hash2 = PortableHash::new(&key).hash64(data);
    assert_eq!(hash1, hash2);
    PortableHash::new(&key).hash128(data);
    PortableHash::new(&key).hash256(data);
});
