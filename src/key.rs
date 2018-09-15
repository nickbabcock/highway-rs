use std::ops::Index;

#[derive(Default, Clone)]
pub struct Key(pub [u64; 4]);

impl Index<usize> for Key {
    type Output = u64;
    fn index(&self, index: usize) -> &u64 {
        &self.0[index]
    }
}
