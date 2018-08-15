#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use std::fmt;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, ShlAssign,
    ShrAssign, SubAssign,
};

#[derive(Clone)]
pub struct V2x64U(pub __m128i);

impl Default for V2x64U {
    fn default() -> Self {
        V2x64U(unsafe { _mm_setzero_si128() })
    }
}

impl fmt::Debug for V2x64U {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut arr: [u64; 2] = [0, 0];
        unsafe { _mm_storeu_si128((&mut arr[0] as *mut u64) as *mut __m128i, self.0) };
        write!(f, "V2x64U: {:?}", arr)
    }
}

impl Copy for V2x64U {}


impl V2x64U {
    pub fn new(hi: u64, low: u64) -> Self {
        V2x64U(unsafe { _mm_set_epi64x(hi as i64, low as i64) })
    }

    pub fn rotate_by_32(&self) -> Self {
        V2x64U(unsafe { _mm_shuffle_epi32(self.0, _mm_shuffle!(2, 3, 0, 1)) })
    }

    pub fn shuffle(&self, mask: &V2x64U) -> Self {
        V2x64U::from(unsafe { _mm_shuffle_epi8(self.0, mask.0) })
    }

    pub fn and_not(&self, neg_mask: &V2x64U) -> Self {
        V2x64U::from(unsafe { _mm_andnot_si128(neg_mask.0, self.0) })
    }
}

impl From<__m128i> for V2x64U {
    fn from(v: __m128i) -> Self {
        V2x64U(v)
    }
}

impl From<[u64; 2]> for V2x64U {
    fn from(v: [u64; 2]) -> Self {
        V2x64U(unsafe { _mm_loadu_si128((&v[0] as *const u64) as *const __m128i) })
    }
}

impl AddAssign for V2x64U {
    fn add_assign(&mut self, other: Self) {
        self.0 = unsafe { _mm_add_epi64(self.0, other.0) }
    }
}

impl SubAssign for V2x64U {
    fn sub_assign(&mut self, other: Self) {
        self.0 = unsafe { _mm_sub_epi64(self.0, other.0) }
    }
}

impl BitAndAssign for V2x64U {
    fn bitand_assign(&mut self, other: Self) {
        self.0 = unsafe { _mm_and_si128(self.0, other.0) }
    }
}

impl BitAnd for V2x64U {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        let mut new = V2x64U(self.0);
        new &= other;
        new
    }
}

impl BitOrAssign for V2x64U {
    fn bitor_assign(&mut self, other: Self) {
        self.0 = unsafe { _mm_or_si128(self.0, other.0) }
    }
}

impl BitOr for V2x64U {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        let mut new = V2x64U(self.0);
        new |= other;
        new
    }
}

impl BitXorAssign for V2x64U {
    fn bitxor_assign(&mut self, other: Self) {
        self.0 = unsafe { _mm_xor_si128(self.0, other.0) }
    }
}

impl Add for V2x64U {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut new = V2x64U(self.0);
        new += other;
        new
    }
}

impl BitXor for V2x64U {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self {
        let mut new = V2x64U(self.0);
        new ^= other;
        new
    }
}

impl ShlAssign<__m128i> for V2x64U {
    fn shl_assign(&mut self, count: __m128i) {
        self.0 = unsafe { _mm_sll_epi64(self.0, count) }
    }
}

impl ShrAssign<__m128i> for V2x64U {
    fn shr_assign(&mut self, count: __m128i) {
        self.0 = unsafe { _mm_srl_epi64(self.0, count) }
    }
}
