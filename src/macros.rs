macro_rules! impl_write {
    ($hasher_struct:ty) => {
        #[cfg(feature = "std")]
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
