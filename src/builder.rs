use key::Key;
use portable::PortableHash;
use traits::HighwayHash;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use avx::AvxHash;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use sse::SseHash;

pub enum HighwayBuilder {
    Portable(PortableHash),
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    Sse(SseHash),
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    Avx(AvxHash),
}

impl HighwayHash for HighwayBuilder {
    fn hash64(self, data: &[u8]) -> u64 {
        match self {
            HighwayBuilder::Portable(x) => x.hash64(data),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            HighwayBuilder::Avx(x) => x.hash64(data),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            HighwayBuilder::Sse(x) => x.hash64(data),
        }
    }

    fn hash128(self, data: &[u8]) -> u128 {
        match self {
            HighwayBuilder::Portable(x) => x.hash128(data),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            HighwayBuilder::Avx(x) => x.hash128(data),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            HighwayBuilder::Sse(x) => x.hash128(data),
        }
    }

    fn hash256(self, data: &[u8]) -> (u128, u128) {
        match self {
            HighwayBuilder::Portable(x) => x.hash256(data),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            HighwayBuilder::Avx(x) => x.hash256(data),
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            HighwayBuilder::Sse(x) => x.hash256(data),
        }
    }
}

impl HighwayBuilder {
    pub fn new(key: &Key) -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if let Some(h) = AvxHash::new(key) {
                return HighwayBuilder::Avx(h);
            }

            if let Some(h) = SseHash::new(key) {
                return HighwayBuilder::Sse(h);
            }
        }

        HighwayBuilder::Portable(PortableHash::new(key))
    }
}
