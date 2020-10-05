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
    ($hasher_struct:ty) => {
        impl ::std::io::Write for $hasher_struct {
            fn write(&mut self, bytes: &[u8]) -> ::std::io::Result<usize> {
                $crate::HighwayHash::append(self, bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> ::std::io::Result<()> {
                Ok(())
            }
        }
    };
}

macro_rules! impl_hasher {
    ($hasher_struct:ty) => {
        impl ::core::hash::Hasher for $hasher_struct {
            fn write(&mut self, bytes: &[u8]) {
                $crate::HighwayHash::append(self, bytes);
            }
            fn finish(&self) -> u64 {
                // Reasons why we need to clone. finalize64` mutates internal state so either we need our
                // Hasher to consume itself or receive a mutable reference on `finish`. We receive neither,
                // due to finish being a misnomer (additional writes could be expected) and it's intended
                // for the hasher to merely return it's current state. The issue with HighwayHash is that
                // there are several rounds of permutations when finalizing a value, and internal state is
                // modified during that process. We work around these constraints by cloning the hasher and
                // finalizing that one.
                $crate::HighwayHash::finalize64(self.clone())
            }
        }
    };
}
