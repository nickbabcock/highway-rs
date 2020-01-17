#![no_main]

mod common;
use highway::{HighwayHash, SseHash};

#[cfg(target_arch = "x86_64")]
libfuzzer_sys::fuzz_target!(|input: common::FuzzKey| {
    let (key, data) = &(input.key, input.data);

    if !is_x86_feature_detected!("sse4.1") {
        panic!("sse4.1 is not supported");
    }

    unsafe {
        let hash1 = SseHash::force_new(&key).hash64(data);
        let hash2 = SseHash::force_new(&key).hash64(data);
        assert_eq!(hash1, hash2);
        SseHash::force_new(&key).hash128(data);
        SseHash::force_new(&key).hash256(data);
    }
});
