pub trait HighwayHash {
    fn hash64(self, data: &[u8]) -> u64;
    fn hash128(self, data: &[u8]) -> u128;
    fn hash256(self, data: &[u8]) -> (u128, u128);
    fn append(&mut self, data: &[u8]);
    fn finalize64(&mut self) -> u64;
    fn finalize128(&mut self) -> u128;
    fn finalize256(&mut self) -> (u128, u128);
}
