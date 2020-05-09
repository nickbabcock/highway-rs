use highway::Key;
use libfuzzer_sys::arbitrary::{Arbitrary, Result, Unstructured};

#[derive(Debug)]
pub struct FuzzKey {
    pub key: Key,
    pub data: Vec<u8>,
}

impl Arbitrary for FuzzKey {
    fn arbitrary(u: &mut Unstructured<'_>) -> Result<Self> {
        let d = <[u64; 4]>::arbitrary(u)?;
        let key = Key(d);
        let data = <Vec<u8>>::arbitrary(u)?;
        Ok(FuzzKey { key, data })
    }
}
