use key::Key;

pub trait HighwayHash {
    fn hash64(data: &[u8], key: &Key) -> u64;
    fn hash128(data: &[u8], key: &Key) -> u128;
    fn hash256(data: &[u8], key: &Key) -> (u128, u128);
}
