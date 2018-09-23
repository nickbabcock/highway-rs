#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate highway;
use highway::{AvxHash, HighwayHash, Key};

#[cfg(target_arch = "x86_64")]
fuzz_target!(|data: &[u8]| {
    if !is_x86_feature_detected!("avx2") {
        panic!("avx2 is not supported");
    }

    unsafe {
        AvxHash::force_new(&Key([1, 2, 3, 4])).hash64(data);
    }
});
