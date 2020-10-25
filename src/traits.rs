/// The common set of methods for hashing data.
pub trait HighwayHash: Sized {
    /// Convenience function for hashing all data in a single call and receiving a 64bit hash.
    /// Results are equivalent to appending the data manually.
    fn hash64(mut self, data: &[u8]) -> u64 {
        self.append(data);
        self.finalize64()
    }

    /// Convenience function for hashing all data in a single call and receiving a 128bit hash.
    /// Results are equivalent to appending the data manually.
    fn hash128(mut self, data: &[u8]) -> [u64; 2] {
        self.append(data);
        self.finalize128()
    }

    /// Convenience function for hashing all data in a single call and receiving a 256bit hash.
    /// Results are equivalent to appending the data manually.
    fn hash256(mut self, data: &[u8]) -> [u64; 4] {
        self.append(data);
        self.finalize256()
    }

    /// Adds data to be hashed. If it is important, the performance characteristics of this
    /// function differs depending on the amount of data previously hashed and the amount of
    /// data to be hashed. For instance, if one appends 50, 1 byte slices then appending the 32nd
    /// byte will have a performance outlier as the internal 32 byte block is complete and internally processed.
    fn append(&mut self, data: &[u8]);

    /// Consumes the hasher to return the 64bit hash
    fn finalize64(self) -> u64;

    /// Consumes the hasher to return the 128bit hash
    fn finalize128(self) -> [u64; 2];

    /// Consumes the hasher to return the 256bit hash
    fn finalize256(self) -> [u64; 4];
}
