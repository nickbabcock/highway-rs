use crate::key::Key;
use crate::traits::HighwayHash;
use core::default::Default;

#[cfg(target_arch = "aarch64")]
use crate::aarch64::NeonHash;
#[cfg(target_arch = "x86_64")]
use crate::avx::AvxHash;
#[cfg(not(any(
    all(target_family = "wasm", target_feature = "simd128"),
    target_arch = "aarch64"
)))]
use crate::portable::PortableHash;
#[cfg(target_arch = "x86_64")]
use crate::sse::SseHash;
#[cfg(all(target_family = "wasm", target_feature = "simd128"))]
use crate::wasm::WasmHash;

#[derive(Debug, Clone)]
enum HighwayChoices {
    #[cfg(not(any(
        all(target_family = "wasm", target_feature = "simd128"),
        target_arch = "aarch64"
    )))]
    Portable(PortableHash),
    #[cfg(target_arch = "x86_64")]
    Sse(SseHash),
    #[cfg(target_arch = "x86_64")]
    Avx(AvxHash),
    #[cfg(target_arch = "aarch64")]
    Neon(NeonHash),
    #[cfg(all(target_family = "wasm", target_feature = "simd128"))]
    Wasm(WasmHash),
}

/// HighwayHash implementation that selects best hash implementation at runtime.
#[derive(Debug, Clone)]
pub struct HighwayBuilder(HighwayChoices);

impl HighwayHash for HighwayBuilder {
    fn append(&mut self, data: &[u8]) {
        match &mut self.0 {
            #[cfg(not(any(
                all(target_family = "wasm", target_feature = "simd128"),
                target_arch = "aarch64"
            )))]
            HighwayChoices::Portable(x) => x.append(data),
            #[cfg(target_arch = "x86_64")]
            HighwayChoices::Avx(x) => x.append(data),
            #[cfg(target_arch = "x86_64")]
            HighwayChoices::Sse(x) => x.append(data),
            #[cfg(target_arch = "aarch64")]
            HighwayChoices::Neon(x) => x.append(data),
            #[cfg(all(target_family = "wasm", target_feature = "simd128"))]
            HighwayChoices::Wasm(x) => x.append(data),
        }
    }

    fn finalize64(self) -> u64 {
        match self.0 {
            #[cfg(not(any(
                all(target_family = "wasm", target_feature = "simd128"),
                target_arch = "aarch64"
            )))]
            HighwayChoices::Portable(x) => x.finalize64(),
            #[cfg(target_arch = "x86_64")]
            HighwayChoices::Avx(x) => x.finalize64(),
            #[cfg(target_arch = "x86_64")]
            HighwayChoices::Sse(x) => x.finalize64(),
            #[cfg(target_arch = "aarch64")]
            HighwayChoices::Neon(x) => x.finalize64(),
            #[cfg(all(target_family = "wasm", target_feature = "simd128"))]
            HighwayChoices::Wasm(x) => x.finalize64(),
        }
    }

    fn finalize128(self) -> [u64; 2] {
        match self.0 {
            #[cfg(not(any(
                all(target_family = "wasm", target_feature = "simd128"),
                target_arch = "aarch64"
            )))]
            HighwayChoices::Portable(x) => x.finalize128(),
            #[cfg(target_arch = "x86_64")]
            HighwayChoices::Avx(x) => x.finalize128(),
            #[cfg(target_arch = "x86_64")]
            HighwayChoices::Sse(x) => x.finalize128(),
            #[cfg(target_arch = "aarch64")]
            HighwayChoices::Neon(x) => x.finalize128(),
            #[cfg(all(target_family = "wasm", target_feature = "simd128"))]
            HighwayChoices::Wasm(x) => x.finalize128(),
        }
    }

    fn finalize256(self) -> [u64; 4] {
        match self.0 {
            #[cfg(not(any(
                all(target_family = "wasm", target_feature = "simd128"),
                target_arch = "aarch64"
            )))]
            HighwayChoices::Portable(x) => x.finalize256(),
            #[cfg(target_arch = "x86_64")]
            HighwayChoices::Avx(x) => x.finalize256(),
            #[cfg(target_arch = "x86_64")]
            HighwayChoices::Sse(x) => x.finalize256(),
            #[cfg(target_arch = "aarch64")]
            HighwayChoices::Neon(x) => x.finalize256(),
            #[cfg(all(target_family = "wasm", target_feature = "simd128"))]
            HighwayChoices::Wasm(x) => x.finalize256(),
        }
    }
}

impl HighwayBuilder {
    /// Creates a new hasher based on compilation and runtime capabilities
    pub fn new(key: Key) -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            if cfg!(target_feature = "avx2") {
                let h = unsafe { AvxHash::force_new(key) };
                return HighwayBuilder(HighwayChoices::Avx(h));
            } else if cfg!(target_feature = "sse4.1") {
                let h = unsafe { SseHash::force_new(key) };
                return HighwayBuilder(HighwayChoices::Sse(h));
            } else {
                if let Some(h) = AvxHash::new(key) {
                    return HighwayBuilder(HighwayChoices::Avx(h));
                }

                if let Some(h) = SseHash::new(key) {
                    return HighwayBuilder(HighwayChoices::Sse(h));
                }
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            // Based on discussions here:
            // https://github.com/nickbabcock/highway-rs/pull/51#discussion_r815247129
            //
            // The following aren't stable:
            //  - std::is_aarch64_feature_detected
            //  - aarch64 target features
            //
            // We're not really able to detect at runtime or compile time
            // if neon support is available
            //
            // The good news is that it seems reasonable to assume the
            // aarch64 environment is neon capable. If a use case is found
            // where neon is not available, we can patch that in later.
            let h = unsafe { NeonHash::force_new(key) };
            HighwayBuilder(HighwayChoices::Neon(h))
        }

        #[cfg(all(target_family = "wasm", target_feature = "simd128"))]
        {
            HighwayBuilder(HighwayChoices::Wasm(WasmHash::new(key)))
        }

        #[cfg(not(any(
            all(target_family = "wasm", target_feature = "simd128"),
            target_arch = "aarch64"
        )))]
        {
            HighwayBuilder(HighwayChoices::Portable(PortableHash::new(key)))
        }
    }
}

impl Default for HighwayBuilder {
    fn default() -> Self {
        HighwayBuilder::new(Key::default())
    }
}

impl_write!(HighwayBuilder);
impl_hasher!(HighwayBuilder);
