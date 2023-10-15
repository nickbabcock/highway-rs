use crate::key::Key;
use crate::traits::HighwayHash;
use core::{default::Default, fmt::Debug, mem::ManuallyDrop};

#[cfg(target_arch = "aarch64")]
use crate::{aarch64::NeonHash, portable::PortableHash};
#[cfg(all(target_arch = "aarch64", feature = "std"))]
use std::arch::is_aarch64_feature_detected;

#[cfg(not(any(
    all(target_family = "wasm", target_feature = "simd128"),
    target_arch = "aarch64"
)))]
use crate::portable::PortableHash;
#[cfg(all(target_family = "wasm", target_feature = "simd128"))]
use crate::wasm::WasmHash;
#[cfg(target_arch = "x86_64")]
use crate::{AvxHash, SseHash};

/// This union is purely for performance. Originally it was an enum, but Rust /
/// LLVM had a hard time optimizing it and would include memcpy's that would
/// dominate profiles.
union HighwayChoices {
    #[cfg(not(any(
        all(target_family = "wasm", target_feature = "simd128"),
        target_arch = "aarch64"
    )))]
    portable: ManuallyDrop<PortableHash>,
    #[cfg(target_arch = "x86_64")]
    avx: ManuallyDrop<AvxHash>,
    #[cfg(target_arch = "x86_64")]
    sse: ManuallyDrop<SseHash>,
    #[cfg(target_arch = "aarch64")]
    neon: ManuallyDrop<NeonHash>,
    #[cfg(target_arch = "aarch64")]
    portable: ManuallyDrop<PortableHash>,
    #[cfg(all(target_family = "wasm", target_feature = "simd128"))]
    wasm: ManuallyDrop<WasmHash>,
}

/// `HighwayHash` implementation that selects best hash implementation at runtime.
pub struct HighwayHasher {
    tag: u8,
    inner: HighwayChoices,
}

impl Debug for HighwayHasher {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("HighwayHasher")
            .field("tag", &self.tag)
            .finish()
    }
}

impl Clone for HighwayHasher {
    fn clone(&self) -> Self {
        let tag = self.tag;
        match tag {
            #[cfg(not(any(
                all(target_family = "wasm", target_feature = "simd128"),
                target_arch = "aarch64"
            )))]
            0 => HighwayHasher {
                tag,
                inner: HighwayChoices {
                    portable: unsafe { self.inner.portable.clone() },
                },
            },
            #[cfg(target_arch = "x86_64")]
            1 => HighwayHasher {
                tag,
                inner: HighwayChoices {
                    avx: unsafe { self.inner.avx.clone() },
                },
            },
            #[cfg(target_arch = "x86_64")]
            2 => HighwayHasher {
                tag,
                inner: HighwayChoices {
                    sse: unsafe { self.inner.sse.clone() },
                },
            },
            #[cfg(all(target_arch = "aarch64", feature = "std"))]
            3 if is_aarch64_feature_detected!("neon") => HighwayHasher {
                tag,
                inner: HighwayChoices {
                    neon: unsafe { self.inner.neon.clone() },
                },
            },
            #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
            3 => HighwayHasher {
                tag,
                inner: HighwayChoices {
                    neon: unsafe { self.inner.neon.clone() },
                },
            },
            #[cfg(all(target_family = "wasm", target_feature = "simd128"))]
            4 => HighwayHasher {
                tag,
                inner: HighwayChoices {
                    wasm: unsafe { self.inner.wasm.clone() },
                },
            },
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}

impl HighwayHash for HighwayHasher {
    #[inline]
    fn append(&mut self, data: &[u8]) {
        self.append(data)
    }

    #[inline]
    fn finalize64(mut self) -> u64 {
        Self::finalize64(&mut self)
    }

    #[inline]
    fn finalize128(mut self) -> [u64; 2] {
        Self::finalize128(&mut self)
    }

    #[inline]
    fn finalize256(mut self) -> [u64; 4] {
        Self::finalize256(&mut self)
    }
}

impl HighwayHasher {
    /// Creates a new hasher based on compilation and runtime capabilities
    #[must_use]
    pub fn new(key: Key) -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            if cfg!(target_feature = "avx2") {
                let avx = ManuallyDrop::new(unsafe { AvxHash::force_new(key) });
                return HighwayHasher {
                    tag: 1,
                    inner: HighwayChoices { avx },
                };
            } else if cfg!(target_feature = "sse4.1") {
                let sse = ManuallyDrop::new(unsafe { SseHash::force_new(key) });
                return HighwayHasher {
                    tag: 2,
                    inner: HighwayChoices { sse },
                };
            } else {
                // Ideally we'd use `AvxHash::new` here, but it triggers a memcpy, so we
                // duplicate the same logic to know if hasher can be enabled.
                #[cfg(feature = "std")]
                if is_x86_feature_detected!("avx2") {
                    let avx: ManuallyDrop<AvxHash> =
                        ManuallyDrop::new(unsafe { AvxHash::force_new(key) });
                    return HighwayHasher {
                        tag: 1,
                        inner: HighwayChoices { avx },
                    };
                }

                #[cfg(feature = "std")]
                if is_x86_feature_detected!("sse4.1") {
                    let sse = ManuallyDrop::new(unsafe { SseHash::force_new(key) });
                    return HighwayHasher {
                        tag: 2,
                        inner: HighwayChoices { sse },
                    };
                }
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            #[cfg(target_feature = "neon")]
            if cfg!(target_feature = "neon") {
                let neon = ManuallyDrop::new(unsafe { NeonHash::force_new(key) });
                return HighwayHasher {
                    tag: 3,
                    inner: HighwayChoices { neon },
                };
            }
            #[cfg(feature = "std")]
            if is_aarch64_feature_detected!("neon") {
                let neon = ManuallyDrop::new(unsafe { NeonHash::force_new(key) });
                return HighwayHasher {
                    tag: 3,
                    inner: HighwayChoices { neon },
                };
            }
            let portable = ManuallyDrop::new(PortableHash::new(key));
            HighwayHasher {
                tag: 0,
                inner: HighwayChoices { portable },
            }
        }

        #[cfg(all(target_family = "wasm", target_feature = "simd128"))]
        {
            let wasm = ManuallyDrop::new(WasmHash::new(key));
            HighwayHasher {
                tag: 4,
                inner: HighwayChoices { wasm },
            }
        }

        #[cfg(not(any(
            all(target_family = "wasm", target_feature = "simd128"),
            target_arch = "aarch64"
        )))]
        {
            let portable = ManuallyDrop::new(PortableHash::new(key));
            HighwayHasher {
                tag: 0,
                inner: HighwayChoices { portable },
            }
        }
    }

    fn append(&mut self, data: &[u8]) {
        match self.tag {
            #[cfg(not(any(
                all(target_family = "wasm", target_feature = "simd128"),
                target_arch = "aarch64"
            )))]
            0 => unsafe { &mut self.inner.portable }.append(data),
            #[cfg(target_arch = "x86_64")]
            1 => unsafe { &mut self.inner.avx }.append(data),
            #[cfg(target_arch = "x86_64")]
            2 => unsafe { &mut self.inner.sse }.append(data),
            #[cfg(target_arch = "aarch64")]
            3 => unsafe { &mut self.inner.neon }.append(data),
            #[cfg(all(target_family = "wasm", target_feature = "simd128"))]
            4 => unsafe { &mut self.inner.wasm }.append(data),
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    fn finalize64(&mut self) -> u64 {
        match self.tag {
            #[cfg(not(any(
                all(target_family = "wasm", target_feature = "simd128"),
                target_arch = "aarch64"
            )))]
            0 => unsafe { PortableHash::finalize64(&mut self.inner.portable) },
            #[cfg(target_arch = "x86_64")]
            1 => unsafe { AvxHash::finalize64(&mut self.inner.avx) },
            #[cfg(target_arch = "x86_64")]
            2 => unsafe { SseHash::finalize64(&mut self.inner.sse) },
            #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
            3 => unsafe { NeonHash::finalize64(&mut self.inner.neon) },
            #[cfg(all(target_arch = "aarch64", feature = "std"))]
            3 if is_aarch64_feature_detected!("neon") => unsafe {
                NeonHash::finalize64(&mut self.inner.neon)
            },
            #[cfg(all(target_family = "wasm", target_feature = "simd128"))]
            4 => unsafe { WasmHash::finalize64(&mut self.inner.wasm) },
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    fn finalize128(&mut self) -> [u64; 2] {
        match self.tag {
            #[cfg(not(any(
                all(target_family = "wasm", target_feature = "simd128"),
                target_arch = "aarch64"
            )))]
            0 => unsafe { PortableHash::finalize128(&mut self.inner.portable) },
            #[cfg(target_arch = "x86_64")]
            1 => unsafe { AvxHash::finalize128(&mut self.inner.avx) },
            #[cfg(target_arch = "x86_64")]
            2 => unsafe { SseHash::finalize128(&mut self.inner.sse) },
            #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
            3 => unsafe { NeonHash::finalize128(&mut self.inner.neon) },
            #[cfg(all(target_arch = "aarch64", feature = "std"))]
            3 if is_aarch64_feature_detected!("neon") => unsafe {
                NeonHash::finalize128(&mut self.inner.neon)
            },
            #[cfg(all(target_family = "wasm", target_feature = "simd128"))]
            4 => unsafe { WasmHash::finalize128(&mut self.inner.wasm) },
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    fn finalize256(&mut self) -> [u64; 4] {
        match self.tag {
            #[cfg(not(any(
                all(target_family = "wasm", target_feature = "simd128"),
                target_arch = "aarch64"
            )))]
            0 => unsafe { PortableHash::finalize256(&mut self.inner.portable) },
            #[cfg(target_arch = "x86_64")]
            1 => unsafe { AvxHash::finalize256(&mut self.inner.avx) },
            #[cfg(target_arch = "x86_64")]
            2 => unsafe { SseHash::finalize256(&mut self.inner.sse) },
            #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
            3 => unsafe { NeonHash::finalize256(&mut self.inner.neon) },
            #[cfg(all(target_arch = "aarch64", feature = "std"))]
            3 if is_aarch64_feature_detected!("neon") => unsafe {
                NeonHash::finalize256(&mut self.inner.neon)
            },
            #[cfg(all(target_family = "wasm", target_feature = "simd128"))]
            4 => unsafe { WasmHash::finalize256(&mut self.inner.wasm) },
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}

impl Default for HighwayHasher {
    fn default() -> Self {
        HighwayHasher::new(Key::default())
    }
}

impl_write!(HighwayHasher);
impl_hasher!(HighwayHasher);
