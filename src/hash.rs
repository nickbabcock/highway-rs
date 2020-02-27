use crate::builder::HighwayBuilder;
use crate::key::Key;
use crate::traits::HighwayHash;
use std::hash::{BuildHasher, Hasher};

#[derive(Debug, Default)]
pub struct HighwayHasher {
    builder: HighwayBuilder,
}

impl HighwayHasher {
    pub fn new(key: &Key) -> Self {
        HighwayHasher {
            builder: HighwayBuilder::new(key),
        }
    }
}

impl Hasher for HighwayHasher {
    fn write(&mut self, bytes: &[u8]) {
        self.builder.append(bytes);
    }

    fn finish(&self) -> u64 {
        // Reasons why we need to clone. finalize64` mutates internal state so either we need our
        // Hasher to consume itself or receive a mutable reference on `finish`. We receive neither,
        // due to finish being a misnomer (additional writes could be expected) and it's intended
        // for the hasher to merely return it's current state. The issue with HighwayHash is that
        // there are several rounds of permutations when finalizing a value, and internal state is
        // modified during that process. We work around these constraints by cloning the hasher and
        // finalizing that one.
        self.builder.clone().finalize64()
    }
}

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
        HighwayHasher::new(&self.key)
    }
}
