use std::arch::x86_64::*;

use std::fmt;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, SubAssign,
};

#[derive(Clone, Copy)]
pub struct V4x64U(pub __m256i);

impl Default for V4x64U {
    fn default() -> Self {
        unsafe { V4x64U::zeroed() }
    }
}

impl fmt::Debug for V4x64U {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V4x64U: {:?}", unsafe { self.to_arr() })
    }
}

macro_rules! _mm_shuffle {
    ($z:expr, $y:expr, $x:expr, $w:expr) => {
        ($z << 6) | ($y << 4) | ($x << 2) | $w
    };
}

impl V4x64U {
    #[target_feature(enable = "avx2")]
    unsafe fn zeroed() -> Self {
        V4x64U(_mm256_setzero_si256())
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn new(highest: u64, high: u64, low: u64, lowest: u64) -> Self {
        V4x64U(_mm256_set_epi64x(
            highest as i64,
            high as i64,
            low as i64,
            lowest as i64,
        ))
    }

    #[target_feature(enable = "avx2")]
    unsafe fn to_arr(&self) -> [u64; 4] {
        let mut arr: [u64; 4] = [0; 4];
        _mm256_storeu_si256(arr.as_mut_ptr() as *mut __m256i, self.0);
        arr
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn rotate_by_32(&self) -> Self {
        V4x64U(_mm256_shuffle_epi32(self.0, _mm_shuffle!(2, 3, 0, 1)))
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn shuffle(&self, mask: &V4x64U) -> Self {
        V4x64U::from(_mm256_shuffle_epi8(self.0, mask.0))
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn mul_low32(&self, x: &V4x64U) -> Self {
        V4x64U::from(_mm256_mul_epu32(self.0, x.0))
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn and_not(&self, neg_mask: &V4x64U) -> Self {
        V4x64U::from(_mm256_andnot_si256(neg_mask.0, self.0))
    }

    #[target_feature(enable = "avx2")]
    unsafe fn add_assign(&mut self, other: Self) {
        self.0 = _mm256_add_epi64(self.0, other.0)
    }

    #[target_feature(enable = "avx2")]
    unsafe fn sub_assign(&mut self, other: Self) {
        self.0 = _mm256_sub_epi64(self.0, other.0)
    }

    #[target_feature(enable = "avx2")]
    unsafe fn bitand_assign(&mut self, other: Self) {
        self.0 = _mm256_and_si256(self.0, other.0)
    }

    #[target_feature(enable = "avx2")]
    unsafe fn bitor_assign(&mut self, other: Self) {
        self.0 = _mm256_or_si256(self.0, other.0)
    }

    #[target_feature(enable = "avx2")]
    unsafe fn bitxor_assign(&mut self, other: Self) {
        self.0 = _mm256_xor_si256(self.0, other.0)
    }
}

impl From<__m256i> for V4x64U {
    fn from(v: __m256i) -> Self {
        V4x64U(v)
    }
}

impl AddAssign for V4x64U {
    fn add_assign(&mut self, other: Self) {
        unsafe { self.add_assign(other) }
    }
}

impl SubAssign for V4x64U {
    fn sub_assign(&mut self, other: Self) {
        unsafe { self.sub_assign(other) }
    }
}

impl BitAndAssign for V4x64U {
    fn bitand_assign(&mut self, other: Self) {
        unsafe { self.bitand_assign(other) }
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
        unsafe { self.bitor_assign(other) }
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
        unsafe { self.bitxor_assign(other) }
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
