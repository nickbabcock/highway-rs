use core::ops::Index;

/// Key used in HighwayHash that will drastically change the hash outputs.
#[derive(Debug, Default, Clone, Copy)]
#[repr(align(32))]
pub struct Key(pub [u64; 4]);

impl Index<usize> for Key {
    type Output = u64;
    fn index(&self, index: usize) -> &u64 {
        &self.0[index]
    }
}
