use highway::Key;

#[derive(Debug, arbitrary::Arbitrary)]
pub struct FuzzKey {
    pub key: Key,
    pub data: Vec<u8>,
}
