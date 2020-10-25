use core::arch::x86_64::*;
use core::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, ShlAssign,
    ShrAssign, SubAssign,
};

#[derive(Clone, Copy)]
pub struct V2x64U(pub __m128i);

impl Default for V2x64U {
    fn default() -> Self {
        unsafe { V2x64U::zeroed() }
    }
}

#[cfg(feature = "std")]
impl std::fmt::Debug for V2x64U {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "V2x64U: {:?}", unsafe { self.to_arr() })
    }
}

impl V2x64U {
    #[target_feature(enable = "sse4.1")]
    unsafe fn zeroed() -> Self {
        V2x64U(_mm_setzero_si128())
    }

    #[target_feature(enable = "sse4.1")]
    pub unsafe fn new(hi: u64, low: u64) -> Self {
        V2x64U(_mm_set_epi64x(hi as i64, low as i64))
    }

    #[target_feature(enable = "sse4.1")]
    #[cfg(feature = "std")]
    unsafe fn to_arr(&self) -> [u64; 2] {
        let mut arr: [u64; 2] = [0, 0];
        _mm_storeu_si128(arr.as_mut_ptr() as *mut __m128i, self.0);
        arr
    }

    #[target_feature(enable = "sse4.1")]
    pub unsafe fn rotate_by_32(&self) -> Self {
        V2x64U(_mm_shuffle_epi32(self.0, _mm_shuffle!(2, 3, 0, 1)))
    }

    #[target_feature(enable = "sse4.1")]
    pub unsafe fn shuffle(&self, mask: &V2x64U) -> Self {
        V2x64U::from(_mm_shuffle_epi8(self.0, mask.0))
    }

    #[target_feature(enable = "sse4.1")]
    pub unsafe fn and_not(&self, neg_mask: &V2x64U) -> Self {
        V2x64U::from(_mm_andnot_si128(neg_mask.0, self.0))
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn add_assign(&mut self, other: Self) {
        self.0 = _mm_add_epi64(self.0, other.0)
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn sub_assign(&mut self, other: Self) {
        self.0 = _mm_sub_epi64(self.0, other.0)
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn bitand_assign(&mut self, other: Self) {
        self.0 = _mm_and_si128(self.0, other.0)
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn bitor_assign(&mut self, other: Self) {
        self.0 = _mm_or_si128(self.0, other.0)
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn bitxor_assign(&mut self, other: Self) {
        self.0 = _mm_xor_si128(self.0, other.0)
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn shl_assign(&mut self, count: __m128i) {
        self.0 = _mm_sll_epi64(self.0, count)
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn shr_assign(&mut self, count: __m128i) {
        self.0 = _mm_srl_epi64(self.0, count)
    }
}

impl From<__m128i> for V2x64U {
    fn from(v: __m128i) -> Self {
        V2x64U(v)
    }
}

impl AddAssign for V2x64U {
    fn add_assign(&mut self, other: Self) {
        unsafe { self.add_assign(other) }
    }
}

impl SubAssign for V2x64U {
    fn sub_assign(&mut self, other: Self) {
        unsafe { self.sub_assign(other) }
    }
}

impl BitAndAssign for V2x64U {
    fn bitand_assign(&mut self, other: Self) {
        unsafe { self.bitand_assign(other) }
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
        unsafe { self.bitor_assign(other) }
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
        unsafe { self.bitxor_assign(other) }
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
        unsafe { self.shl_assign(count) }
    }
}

impl ShrAssign<__m128i> for V2x64U {
    fn shr_assign(&mut self, count: __m128i) {
        unsafe { self.shr_assign(count) }
    }
}
