use crate::builder::HighwayBuilder;
use crate::key::Key;
use core::hash::BuildHasher;

/// HighwayHash implementation that selects best hash implementation at runtime.
pub type HighwayHasher = HighwayBuilder;

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
        HighwayBuilder::new(self.key)
    }
}
