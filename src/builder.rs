use crate::key::Key;
use crate::portable::PortableHash;
use crate::traits::HighwayHash;
use core::default::Default;

#[cfg(target_arch = "x86_64")]
use crate::avx::AvxHash;
#[cfg(target_arch = "x86_64")]
use crate::sse::SseHash;

#[derive(Debug, Clone)]
enum HighwayChoices {
    Portable(PortableHash),
    #[cfg(target_arch = "x86_64")]
    Sse(SseHash),
    #[cfg(target_arch = "x86_64")]
    Avx(AvxHash),
}

/// HighwayHash implementation that selects best hash implementation at runtime.
#[derive(Debug, Clone)]
pub struct HighwayBuilder(HighwayChoices);

impl HighwayHash for HighwayBuilder {
    fn append(&mut self, data: &[u8]) {
        match &mut self.0 {
            HighwayChoices::Portable(x) => x.append(data),
            #[cfg(target_arch = "x86_64")]
            HighwayChoices::Avx(x) => x.append(data),
            #[cfg(target_arch = "x86_64")]
            HighwayChoices::Sse(x) => x.append(data),
        }
    }

    fn finalize64(self) -> u64 {
        match self.0 {
            HighwayChoices::Portable(x) => x.finalize64(),
            #[cfg(target_arch = "x86_64")]
            HighwayChoices::Avx(x) => x.finalize64(),
            #[cfg(target_arch = "x86_64")]
            HighwayChoices::Sse(x) => x.finalize64(),
        }
    }

    fn finalize128(self) -> [u64; 2] {
        match self.0 {
            HighwayChoices::Portable(x) => x.finalize128(),
            #[cfg(target_arch = "x86_64")]
            HighwayChoices::Avx(x) => x.finalize128(),
            #[cfg(target_arch = "x86_64")]
            HighwayChoices::Sse(x) => x.finalize128(),
        }
    }

    fn finalize256(self) -> [u64; 4] {
        match self.0 {
            HighwayChoices::Portable(x) => x.finalize256(),
            #[cfg(target_arch = "x86_64")]
            HighwayChoices::Avx(x) => x.finalize256(),
            #[cfg(target_arch = "x86_64")]
            HighwayChoices::Sse(x) => x.finalize256(),
        }
    }
}

impl HighwayBuilder {
    /// Creates a new hasher based on compilation and runtime capabilities
    pub fn new(key: Key) -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            if let Some(h) = AvxHash::new(key) {
                return HighwayBuilder(HighwayChoices::Avx(h));
            }

            if let Some(h) = SseHash::new(key) {
                return HighwayBuilder(HighwayChoices::Sse(h));
            }
        }

        HighwayBuilder(HighwayChoices::Portable(PortableHash::new(key)))
    }
}

impl Default for HighwayBuilder {
    fn default() -> Self {
        HighwayBuilder::new(Key::default())
    }
}

impl_write!(HighwayBuilder);
impl_hasher!(HighwayBuilder);
