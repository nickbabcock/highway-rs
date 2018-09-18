use std::arch::x86_64::*;

use std::fmt;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, SubAssign,
};

#[derive(Clone, Copy)]
pub struct V4x64U(pub __m256i);

impl Default for V4x64U {
    fn default() -> Self {
        V4x64U(unsafe { _mm256_setzero_si256() })
    }
}

impl fmt::Debug for V4x64U {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut arr: [u64; 4] = [0; 4];
        unsafe { _mm256_storeu_si256((&mut arr[0] as *mut u64) as *mut __m256i, self.0) };
        write!(f, "V4x64U: {:?}", arr)
    }
}

macro_rules! _mm_shuffle {
    ($z:expr, $y:expr, $x:expr, $w:expr) => {
        ($z << 6) | ($y << 4) | ($x << 2) | $w
    };
}

impl V4x64U {
    pub fn new(highest: u64, high: u64, low: u64, lowest: u64) -> Self {
        V4x64U(unsafe { _mm256_set_epi64x(highest as i64, high as i64, low as i64, lowest as i64) })
    }

    pub fn rotate_by_32(&self) -> Self {
        V4x64U(unsafe { _mm256_shuffle_epi32(self.0, _mm_shuffle!(2, 3, 0, 1)) })
    }

    pub fn shuffle(&self, mask: &V4x64U) -> Self {
        V4x64U::from(unsafe { _mm256_shuffle_epi8(self.0, mask.0) })
    }

    pub fn mul_low32(&self, x: &V4x64U) -> Self {
        V4x64U::from(unsafe { _mm256_mul_epu32(self.0, x.0) })
    }

    pub fn and_not(&self, neg_mask: &V4x64U) -> Self {
        V4x64U::from(unsafe { _mm256_andnot_si256(neg_mask.0, self.0) })
    }
}

impl From<__m256i> for V4x64U {
    fn from(v: __m256i) -> Self {
        V4x64U(v)
    }
}

/*
impl From<[u64; 2]> for V4x64U {
    fn from(v: [u64; 2]) -> Self {
        V4x64U(unsafe { _mm_loadu_si128((&v[0] as *const u64) as *const __m256i) })
    }
}*/

impl AddAssign for V4x64U {
    fn add_assign(&mut self, other: Self) {
        self.0 = unsafe { _mm256_add_epi64(self.0, other.0) }
    }
}

impl SubAssign for V4x64U {
    fn sub_assign(&mut self, other: Self) {
        self.0 = unsafe { _mm256_sub_epi64(self.0, other.0) }
    }
}

impl BitAndAssign for V4x64U {
    fn bitand_assign(&mut self, other: Self) {
        self.0 = unsafe { _mm256_and_si256(self.0, other.0) }
    }
}

impl BitAnd for V4x64U {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        let mut new = V4x64U(self.0);
        new &= other;
        new
    }
}

impl BitOrAssign for V4x64U {
    fn bitor_assign(&mut self, other: Self) {
        self.0 = unsafe { _mm256_or_si256(self.0, other.0) }
    }
}

impl BitOr for V4x64U {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        let mut new = V4x64U(self.0);
        new |= other;
        new
    }
}

impl BitXorAssign for V4x64U {
    fn bitxor_assign(&mut self, other: Self) {
        self.0 = unsafe { _mm256_xor_si256(self.0, other.0) }
    }
}

impl Add for V4x64U {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut new = V4x64U(self.0);
        new += other;
        new
    }
}

impl BitXor for V4x64U {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self {
        let mut new = V4x64U(self.0);
        new ^= other;
        new
    }
}
