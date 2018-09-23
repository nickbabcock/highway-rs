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
        SseHash::force_new(&Key([1, 2, 3, 4])).hash64(data);
    }
});
