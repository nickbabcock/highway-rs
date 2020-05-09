#![no_main]

mod common;
use highway::{AvxHash, HighwayHash};

#[cfg(target_arch = "x86_64")]
libfuzzer_sys::fuzz_target!(|input: common::FuzzKey| {
    let (key, data) = (input.key, &input.data);

    if !is_x86_feature_detected!("avx2") {
        panic!("avx2 is not supported");
    }

    unsafe {
        let hash1 = AvxHash::force_new(key).hash64(data);
        let hash2 = AvxHash::force_new(key).hash64(data);
        assert_eq!(hash1, hash2);
        AvxHash::force_new(key).hash128(data);
        AvxHash::force_new(key).hash256(data);
    }
});
