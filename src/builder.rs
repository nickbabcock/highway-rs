use key::Key;
use portable::PortableHash;
use traits::HighwayHash;

pub struct HighwayBuilder;

impl HighwayBuilder {
    pub fn hash64(data: &[u8], key: &Key) -> u64 {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if is_x86_feature_detected!("avx2") {
                use avx::AvxHash;
                return AvxHash::hash64(data, key);
            }

            if is_x86_feature_detected!("sse4.1") {
                use sse::SseHash;
                return SseHash::hash64(data, key);
            }
        }

        PortableHash::hash64(data, key)
    }
}
