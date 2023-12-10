use core::arch::x86_64::*;
use core::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, ShlAssign,
    ShrAssign, SubAssign,
};

#[derive(Clone, Copy)]
pub struct V2x64U(pub __m128i);

impl Default for V2x64U {
    #[inline]
    fn default() -> Self {
        unsafe { V2x64U::zeroed() }
    }
}

impl core::fmt::Debug for V2x64U {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "V2x64U: {:?}", unsafe { self.as_arr() })
    }
}

impl V2x64U {
    #[inline]
    #[target_feature(enable = "sse4.1")]
    unsafe fn zeroed() -> Self {
        V2x64U(_mm_setzero_si128())
    }

    #[inline]
    #[target_feature(enable = "sse4.1")]
    pub unsafe fn new(hi: u64, low: u64) -> Self {
        V2x64U(_mm_set_epi64x(hi as i64, low as i64))
    }

    #[target_feature(enable = "sse4.1")]
    pub unsafe fn as_arr(&self) -> [u64; 2] {
        let mut arr: [u64; 2] = [0, 0];
        _mm_storeu_si128(arr.as_mut_ptr().cast::<__m128i>(), self.0);
        arr
    }

    #[inline]
    #[target_feature(enable = "sse4.1")]
    pub unsafe fn rotate_by_32(&self) -> Self {
        V2x64U(_mm_shuffle_epi32(self.0, _mm_shuffle!(2, 3, 0, 1)))
    }

    #[inline]
    #[target_feature(enable = "sse4.1")]
    pub unsafe fn shuffle(&self, mask: &V2x64U) -> Self {
        V2x64U::from(_mm_shuffle_epi8(self.0, mask.0))
    }

    #[inline]
    #[target_feature(enable = "sse4.1")]
    pub unsafe fn and_not(&self, neg_mask: &V2x64U) -> Self {
        V2x64U::from(_mm_andnot_si128(neg_mask.0, self.0))
    }

    #[inline]
    #[target_feature(enable = "sse4.1")]
    unsafe fn add_assign(&mut self, other: Self) {
        self.0 = _mm_add_epi64(self.0, other.0);
    }

    #[inline]
    #[target_feature(enable = "sse4.1")]
    unsafe fn sub_assign(&mut self, other: Self) {
        self.0 = _mm_sub_epi64(self.0, other.0);
    }

    #[inline]
    #[target_feature(enable = "sse4.1")]
    unsafe fn bitand_assign(&mut self, other: Self) {
        self.0 = _mm_and_si128(self.0, other.0);
    }

    #[inline]
    #[target_feature(enable = "sse4.1")]
    unsafe fn bitor_assign(&mut self, other: Self) {
        self.0 = _mm_or_si128(self.0, other.0);
    }

    #[inline]
    #[target_feature(enable = "sse4.1")]
    unsafe fn bitxor_assign(&mut self, other: Self) {
        self.0 = _mm_xor_si128(self.0, other.0);
    }

    #[inline]
    #[target_feature(enable = "sse4.1")]
    unsafe fn shl_assign(&mut self, count: __m128i) {
        self.0 = _mm_sll_epi64(self.0, count);
    }

    #[inline]
    #[target_feature(enable = "sse4.1")]
    unsafe fn shr_assign(&mut self, count: __m128i) {
        self.0 = _mm_srl_epi64(self.0, count);
    }
}

impl From<__m128i> for V2x64U {
    #[inline]
    fn from(v: __m128i) -> Self {
        V2x64U(v)
    }
}

impl AddAssign for V2x64U {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        unsafe { self.add_assign(other) }
    }
}

impl SubAssign for V2x64U {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        unsafe { self.sub_assign(other) }
    }
}

impl BitAndAssign for V2x64U {
    #[inline]
    fn bitand_assign(&mut self, other: Self) {
        unsafe { self.bitand_assign(other) }
    }
}

impl BitAnd for V2x64U {
    type Output = Self;
    #[inline]
    fn bitand(self, other: Self) -> Self {
        let mut new = V2x64U(self.0);
        new &= other;
        new
    }
}

impl BitOrAssign for V2x64U {
    #[inline]
    fn bitor_assign(&mut self, other: Self) {
        unsafe { self.bitor_assign(other) }
    }
}

impl BitOr for V2x64U {
    type Output = Self;
    #[inline]
    fn bitor(self, other: Self) -> Self {
        let mut new = V2x64U(self.0);
        new |= other;
        new
    }
}

impl BitXorAssign for V2x64U {
    #[inline]
    fn bitxor_assign(&mut self, other: Self) {
        unsafe { self.bitxor_assign(other) }
    }
}

impl Add for V2x64U {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self {
        let mut new = V2x64U(self.0);
        new += other;
        new
    }
}

impl BitXor for V2x64U {
    type Output = Self;

    #[inline]
    fn bitxor(self, other: Self) -> Self {
        let mut new = V2x64U(self.0);
        new ^= other;
        new
    }
}

impl ShlAssign<__m128i> for V2x64U {
    #[inline]
    fn shl_assign(&mut self, count: __m128i) {
        unsafe { self.shl_assign(count) }
    }
}

impl ShrAssign<__m128i> for V2x64U {
    #[inline]
    fn shr_assign(&mut self, count: __m128i) {
        unsafe { self.shr_assign(count) }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_as_arr() {
        unsafe {
            let x = V2x64U::new(55, 1);
            let res = x.as_arr();
            assert_eq!(res, [1, 55]);
        }
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_rotate_by_32() {
        unsafe {
            let x = V2x64U::new(0x0264_432C_CD8A_70E0, 0x0B28_E3EF_EBB3_172D);
            let y = x.rotate_by_32();
            let res = y.as_arr();
            assert_eq!(res, [0xEBB3_172D_0B28_E3EF, 0xCD8A_70E0_0264_432C]);
        }
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_add() {
        unsafe {
            let x = V2x64U::new(55, 1);
            let y = V2x64U::new(0x0264_432C_CD8A_70E0, 0x0B28E_3EFE_BB3_172D);
            let z = x + y;
            assert_eq!(z.as_arr(), [0x0B28_E3EF_EBB3_172E, 0x2644_32CC_D8A7_117]);
        }
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_mm_srli_epi64() {
        unsafe {
            let x = V2x64U::new(0x0264_432C_CD8A_70E0, 0x0B28E_3EFE_BB3_172D);
            let y = V2x64U::from(_mm_srli_epi64(x.0, 33));
            assert_eq!(y.as_arr(), [0x0000_0000_0594_71F7, 0x0000_0000_0132_2196]);
        }
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_mm_mul_epu32() {
        unsafe {
            let x = V2x64U::new(0x0264_432C_CD8A_70E0, 0x0B28_E3EF_EBB3_172D);
            let y = V2x64U::new(0x0B28_E3EF_EBB3_172D, 0x0264_432C_CD8A_70E0);
            let z = V2x64U::from(_mm_mul_epu32(x.0, y.0));
            assert_eq!(z.as_arr(), [0xBD3D_E006_1E19_F760, 0xBD3D_E006_1E19_F760]);
        }
    }

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_mm_slli_si128_8() {
        unsafe {
            let x = V2x64U::new(0, 0xFFFF_FFFF);
            let y = V2x64U::from(_mm_slli_si128(x.0, 8));
            assert_eq!(y.as_arr(), [0, 0xFFFF_FFFF]);
        }
    }
}
