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
        let key = Key([1, 2, 3, 4]);
        let hash1 = AvxHash::force_new(&key).hash64(data);
        let hash2 = AvxHash::force_new(&key).hash64(data);
        assert_eq!(hash1, hash2);
        AvxHash::force_new(&key).hash128(data);
        AvxHash::force_new(&key).hash256(data);
    }
});
