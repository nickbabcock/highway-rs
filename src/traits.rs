pub trait HighwayHash {
    fn hash64(self, data: &[u8]) -> u64;
    fn hash128(self, data: &[u8]) -> u128;
    fn hash256(self, data: &[u8]) -> (u128, u128);
}
