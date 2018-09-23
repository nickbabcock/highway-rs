#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate highway;
use highway::{HighwayHash, Key, PortableHash};

fuzz_target!(|data: &[u8]| {
    PortableHash::new(&Key([1, 2, 3, 4])).hash64(data);
});
