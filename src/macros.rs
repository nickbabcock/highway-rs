#[cfg(target_arch = "x86_64")]
/// The function, [_MM_SHUFFLE](https://doc.rust-lang.org/core/arch/x86_64/fn._MM_SHUFFLE.html) is
/// only supported on nightly and there has been [some controversy
/// around](https://github.com/rust-lang-nursery/stdsimd/issues/522) it regarding the type
/// signature, so the safe route here is to just go with our own macro.
macro_rules! _mm_shuffle {
    ($z:expr, $y:expr, $x:expr, $w:expr) => {
        ($z << 6) | ($y << 4) | ($x << 2) | $w
    };
}

macro_rules! impl_write {
    ($hasher_struct:ident) => {
        impl ::std::io::Write for $hasher_struct {
            fn write(&mut self, bytes: &[u8]) -> ::std::io::Result<usize> {
                crate::HighwayHash::append(self, bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> ::std::io::Result<()> {
                Ok(())
            }
        }
    };
}

macro_rules! impl_hasher {
    ($hasher_struct:ident) => {
        impl ::core::hash::Hasher for $hasher_struct {
            fn write(&mut self, bytes: &[u8]) {
                crate::HighwayHash::append(self, bytes);
            }
            fn finish(&self) -> ::std::primitive::u64 {
                crate::HighwayHash::finalize64(self.clone())
            }
        }
    };
}
