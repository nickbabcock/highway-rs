#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate highway;
use highway::{HighwayHash, Key, SseHash};

#[cfg(target_arch = "x86_64")]
fuzz_target!(|data: &[u8]| {
    if !is_x86_feature_detected!("sse4.1") {
        panic!("sse4.1 is not supported");
    }

    unsafe {
        let key = Key([1, 2, 3, 4]);
        let hash1 = SseHash::force_new(&key).hash64(data);
        let hash2 = SseHash::force_new(&key).hash64(data);
        assert_eq!(hash1, hash2);
        SseHash::force_new(&key).hash128(data);
        SseHash::force_new(&key).hash256(data);
    }
});
