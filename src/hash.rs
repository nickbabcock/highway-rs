use crate::builder::HighwayBuilder;
use crate::key::Key;
use core::hash::BuildHasher;

pub type HighwayHasher = HighwayBuilder;

#[derive(Debug, Default)]
pub struct HighwayBuildHasher {
    key: Key,
}

impl HighwayBuildHasher {
    pub fn new(key: Key) -> Self {
        HighwayBuildHasher { key }
    }
}

impl BuildHasher for HighwayBuildHasher {
    type Hasher = HighwayHasher;

    fn build_hasher(&self) -> Self::Hasher {
        HighwayBuilder::new(self.key)
    }
}
