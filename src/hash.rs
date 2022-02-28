use crate::builder::HighwayHasher;
use crate::key::Key;
use core::hash::BuildHasher;

/// Constructs a hasher used in rust collections
#[derive(Debug, Default)]
pub struct HighwayBuildHasher {
    key: Key,
}

impl HighwayBuildHasher {
    /// Creates a new hash builder with a given key
    pub fn new(key: Key) -> Self {
        HighwayBuildHasher { key }
    }
}

impl BuildHasher for HighwayBuildHasher {
    type Hasher = HighwayHasher;

    fn build_hasher(&self) -> Self::Hasher {
        HighwayHasher::new(self.key)
    }
}
