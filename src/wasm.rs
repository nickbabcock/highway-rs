use crate::internal::{unordered_load3, HashPacket, PACKET_SIZE};
use crate::{HighwayHash, Key};
use core::arch::wasm32::{self, v128};
use core::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, ShlAssign,
    ShrAssign, SubAssign,
};

/// HighwayHash powered by Wasm SIMD instructions
#[derive(Debug, Default, Clone)]
pub struct WasmHash {
    v0L: V2x64U,
    v0H: V2x64U,
    v1L: V2x64U,
    v1H: V2x64U,
    mul0L: V2x64U,
    mul0H: V2x64U,
    mul1L: V2x64U,
    mul1H: V2x64U,
    buffer: HashPacket,
}

impl HighwayHash for WasmHash {
    #[inline]
    fn append(&mut self, data: &[u8]) {
        self.append(data);
    }

    #[inline]
    fn finalize64(mut self) -> u64 {
        Self::finalize64(&mut self)
    }

    #[inline]
    fn finalize128(mut self) -> [u64; 2] {
        Self::finalize128(&mut self)
    }

    #[inline]
    fn finalize256(mut self) -> [u64; 4] {
        Self::finalize256(&mut self)
    }
}

impl WasmHash {
    /// Creates a new `WasmHash` based on Wasm SIMD extension
    pub fn new(key: Key) -> Self {
        let init0L = V2x64U::new(0xa409_3822_299f_31d0, 0xdbe6_d5d5_fe4c_ce2f);
        let init0H = V2x64U::new(0x243f_6a88_85a3_08d3, 0x1319_8a2e_0370_7344);
        let init1L = V2x64U::new(0xc0ac_f169_b5f1_8a8c, 0x3bd3_9e10_cb0e_f593);
        let init1H = V2x64U::new(0x4528_21e6_38d0_1377, 0xbe54_66cf_34e9_0c6c);
        let keyL = V2x64U::new(key[1], key[0]);
        let keyH = V2x64U::new(key[3], key[2]);

        WasmHash {
            v0L: keyL ^ init0L,
            v0H: keyH ^ init0H,
            v1L: keyL.rotate_by_32() ^ init1L,
            v1H: keyH.rotate_by_32() ^ init1H,
            mul0L: init0L,
            mul0H: init0H,
            mul1L: init1L,
            mul1H: init1H,
            buffer: HashPacket::default(),
        }
    }

    fn zipper_merge(v: &V2x64U) -> V2x64U {
        let ignored = v.0;

        let res = wasm32::u8x16_shuffle::<3, 12, 2, 5, 1, 14, 0, 15, 11, 4, 10, 13, 6, 9, 7, 8>(
            v.0, ignored,
        );
        V2x64U::from(res)
    }

    fn update(&mut self, (packetH, packetL): (V2x64U, V2x64U)) {
        self.v1L += packetL;
        self.v1H += packetH;
        self.v1L += self.mul0L;
        self.v1H += self.mul0H;
        self.mul0L ^= V2x64U(_mm_mul_epu32(self.v1L.0, self.v0L.rotate_by_32().0));
        self.mul0H ^= V2x64U(_mm_mul_epu32(self.v1H.0, _mm_srli_epi64(self.v0H.0, 32)));
        self.v0L += self.mul1L;
        self.v0H += self.mul1H;
        self.mul1L ^= V2x64U(_mm_mul_epu32(self.v0L.0, self.v1L.rotate_by_32().0));
        self.mul1H ^= V2x64U(_mm_mul_epu32(self.v0H.0, _mm_srli_epi64(self.v1H.0, 32)));
        self.v0L += WasmHash::zipper_merge(&self.v1L);
        self.v0H += WasmHash::zipper_merge(&self.v1H);
        self.v1L += WasmHash::zipper_merge(&self.v0L);
        self.v1H += WasmHash::zipper_merge(&self.v0H);
    }

    fn permute_and_update(&mut self) {
        let low = self.v0L.rotate_by_32();
        let high = self.v0H.rotate_by_32();
        self.update((low, high));
    }

    pub(crate) fn finalize64(&mut self) -> u64 {
        if !self.buffer.is_empty() {
            self.update_remainder();
        }

        for _i in 0..4 {
            self.permute_and_update();
        }

        let sum0 = self.v0L + self.mul0L;
        let sum1 = self.v1L + self.mul1L;
        let hash = sum0 + sum1;

        wasm32::u64x2_extract_lane::<1>(hash.0)
    }

    pub(crate) fn finalize128(&mut self) -> [u64; 2] {
        if !self.buffer.is_empty() {
            self.update_remainder();
        }

        for _i in 0..6 {
            self.permute_and_update();
        }

        let sum0 = self.v0L + self.mul0L;
        let sum1 = self.v1H + self.mul1H;
        let hash = sum0 + sum1;
        [
            wasm32::u64x2_extract_lane::<1>(hash.0),
            wasm32::u64x2_extract_lane::<0>(hash.0),
        ]
    }

    pub(crate) fn finalize256(&mut self) -> [u64; 4] {
        if !self.buffer.is_empty() {
            self.update_remainder();
        }

        for _i in 0..10 {
            self.permute_and_update();
        }

        let sum0L = self.v0L + self.mul0L;
        let sum1L = self.v1L + self.mul1L;
        let sum0H = self.v0H + self.mul0H;
        let sum1H = self.v1H + self.mul1H;
        let hashL = WasmHash::modular_reduction(&sum1L, &sum0L);
        let hashH = WasmHash::modular_reduction(&sum1H, &sum0H);

        [
            wasm32::u64x2_extract_lane::<1>(hashL.0),
            wasm32::u64x2_extract_lane::<0>(hashL.0),
            wasm32::u64x2_extract_lane::<1>(hashH.0),
            wasm32::u64x2_extract_lane::<0>(hashH.0),
        ]
    }

    fn modular_reduction(x: &V2x64U, init: &V2x64U) -> V2x64U {
        let zero = V2x64U::default();
        let repl = wasm32::i32x4_replace_lane::<1>(zero.0, 0x8000_0000_u32 as i32);
        let sign_bit128 = V2x64U::from(repl);
        let top_bits2 = V2x64U::from(_mm_srli_epi64(x.0, 62));
        let shifted1_unmasked = *x + *x;
        let top_bits1 = V2x64U::from(_mm_srli_epi64(x.0, 63));
        let shifted2 = shifted1_unmasked + shifted1_unmasked;
        let new_low_bits2 = V2x64U::from(_mm_slli_si128_8(top_bits2.0));
        let shifted1 = shifted1_unmasked.and_not(&sign_bit128);
        let new_low_bits1 = V2x64U::from(_mm_slli_si128_8(top_bits1.0));
        *init ^ shifted2 ^ new_low_bits2 ^ shifted1 ^ new_low_bits1
    }

    fn load_multiple_of_four(bytes: &[u8]) -> V2x64U {
        let mut data = bytes;
        let mut mask4 = V2x64U::new(0, 0xFFFF_FFFF);
        let mut ret = if bytes.len() >= 8 {
            let lo = le_u64(bytes);
            mask4 = V2x64U::from(_mm_slli_si128_8(mask4.0));
            data = &bytes[8..];
            V2x64U::new(0, lo)
        } else {
            V2x64U::new(0, 0)
        };

        if let Some(d) = data.get(..4) {
            let last4 = u32::from_le_bytes([d[0], d[1], d[2], d[3]]);
            let broadcast = V2x64U::from(wasm32::u32x4(last4, last4, last4, last4));
            ret |= broadcast & mask4;
        }

        ret
    }

    fn remainder(bytes: &[u8]) -> (V2x64U, V2x64U) {
        let size_mod32 = bytes.len();
        let size_mod4 = size_mod32 & 3;
        if bytes.len() > 32 {
            debug_assert!(false, "remainder bytes must be less than 32");
            return (V2x64U::zeroed(), V2x64U::zeroed());
        }

        if bytes.len() >= 16 {
            let packetLL = le_u64(bytes);
            let packetLH = le_u64(&bytes[8..]);
            let packetL = V2x64U::new(packetLH, packetLL);
            let packett = WasmHash::load_multiple_of_four(&bytes[16..]);
            let remainder = &bytes[(size_mod32 & !3) + size_mod4 - 4..];
            let last4 =
                i32::from_le_bytes([remainder[0], remainder[1], remainder[2], remainder[3]]);

            let packetH = V2x64U::from(wasm32::i32x4_replace_lane::<1>(packett.0, last4));
            (packetH, packetL)
        } else {
            let remainder = &bytes[size_mod32 & !3..];
            let packetL = WasmHash::load_multiple_of_four(bytes);

            let last4 = unordered_load3(remainder);
            let packetH = V2x64U::new(0, last4);
            (packetH, packetL)
        }
    }

    fn update_remainder(&mut self) {
        let size = self.buffer.len() as i32;
        let vsize_mod32 = wasm32::i32x4(size, size, size, size);
        self.v0L += V2x64U::from(vsize_mod32);
        self.v0H += V2x64U::from(vsize_mod32);
        self.rotate_32_by(size as u32);
        let packet = WasmHash::remainder(self.buffer.as_slice());
        self.update(packet);
    }

    fn rotate_32_by(&mut self, count: u32) {
        let vL = &mut self.v1L;
        let vH = &mut self.v1H;
        let count_left = count;
        let count_right = 32 - count;

        let shifted_leftL = V2x64U::from(_mm_sll_epi32(vL.0, count_left));
        let shifted_leftH = V2x64U::from(_mm_sll_epi32(vH.0, count_left));
        let shifted_rightL = V2x64U::from(_mm_srl_epi32(vL.0, count_right));
        let shifted_rightH = V2x64U::from(_mm_srl_epi32(vH.0, count_right));
        *vL = shifted_leftL | shifted_rightL;
        *vH = shifted_leftH | shifted_rightH;
    }

    #[inline]
    fn data_to_lanes(packet: &[u8]) -> (V2x64U, V2x64U) {
        let mut lanes = [0u64; 4];
        for (x, dest) in packet.chunks_exact(8).zip(lanes.iter_mut()) {
            *dest = le_u64(x);
        }

        let hi = V2x64U::new(lanes[3], lanes[2]);
        let lo = V2x64U::new(lanes[1], lanes[0]);
        (hi, lo)
    }

    fn append(&mut self, data: &[u8]) {
        if self.buffer.is_empty() {
            let mut chunks = data.chunks_exact(PACKET_SIZE);
            for chunk in chunks.by_ref() {
                self.update(Self::data_to_lanes(chunk));
            }
            self.buffer.set_to(chunks.remainder());
        } else if let Some(tail) = self.buffer.fill(data) {
            self.update(Self::data_to_lanes(self.buffer.inner()));
            let mut chunks = tail.chunks_exact(PACKET_SIZE);
            for chunk in chunks.by_ref() {
                self.update(Self::data_to_lanes(chunk));
            }

            self.buffer.set_to(chunks.remainder());
        }
    }
}

impl_write!(WasmHash);
impl_hasher!(WasmHash);

#[inline]
fn le_u64(x: &[u8]) -> u64 {
    u64::from_le_bytes([x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7]])
}

#[inline]
fn _mm_mul_epu32(a: wasm32::v128, b: wasm32::v128) -> wasm32::v128 {
    let mask = wasm32::u32x4(0xFFFF_FFFF, 0, 0xFFFF_FFFF, 0);
    let lo_a_0 = wasm32::v128_and(a, mask);
    let lo_b_0 = wasm32::v128_and(b, mask);
    wasm32::u64x2_mul(lo_a_0, lo_b_0)
}

#[inline]
fn _mm_srli_epi64(a: wasm32::v128, amt: u32) -> wasm32::v128 {
    wasm32::u64x2_shr(a, amt)
}

#[inline]
fn _mm_srl_epi32(a: wasm32::v128, amt: u32) -> wasm32::v128 {
    wasm32::u32x4_shr(a, amt)
}

#[inline]
fn _mm_sll_epi32(a: wasm32::v128, amt: u32) -> wasm32::v128 {
    wasm32::u32x4_shl(a, amt)
}

#[inline]
fn _mm_slli_si128_8(a: wasm32::v128) -> wasm32::v128 {
    // aka _mm_bslli_si128_8
    let zero = wasm32::u64x2(0, 0);
    wasm32::u64x2_shuffle::<1, 2>(a, zero)
}

#[derive(Clone, Copy)]
pub struct V2x64U(pub v128);

impl Default for V2x64U {
    fn default() -> Self {
        V2x64U::zeroed()
    }
}

impl core::fmt::Debug for V2x64U {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "V2x64U: {:?}", self.as_arr())
    }
}

impl V2x64U {
    #[inline]
    fn zeroed() -> Self {
        Self::new(0, 0)
    }

    #[inline]
    pub fn new(hi: u64, low: u64) -> Self {
        V2x64U(wasm32::u64x2(hi, low))
    }

    fn as_arr(&self) -> [u64; 2] {
        let hi = wasm32::u64x2_extract_lane::<0>(self.0);
        let lo = wasm32::u64x2_extract_lane::<1>(self.0);
        [lo, hi]
    }

    #[inline]
    pub fn rotate_by_32(&self) -> Self {
        let ignored = self.0;
        let res = wasm32::u32x4_shuffle::<1, 0, 3, 2>(self.0, ignored);
        V2x64U::from(res)
    }

    #[inline]
    pub fn and_not(&self, neg_mask: &V2x64U) -> Self {
        V2x64U::from(wasm32::v128_andnot(self.0, neg_mask.0))
    }

    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.0 = wasm32::u64x2_add(self.0, other.0)
    }

    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.0 = wasm32::u64x2_sub(self.0, other.0)
    }

    #[inline]
    fn bitand_assign(&mut self, other: Self) {
        self.0 = wasm32::v128_and(self.0, other.0)
    }

    #[inline]
    fn bitor_assign(&mut self, other: Self) {
        self.0 = wasm32::v128_or(self.0, other.0)
    }

    #[inline]
    fn bitxor_assign(&mut self, other: Self) {
        self.0 = wasm32::v128_xor(self.0, other.0)
    }

    #[inline]
    fn shl_assign(&mut self, count: u32) {
        self.0 = wasm32::u64x2_shl(self.0, count)
    }

    #[inline]
    fn shr_assign(&mut self, count: u32) {
        self.0 = wasm32::u64x2_shr(self.0, count)
    }
}

impl From<v128> for V2x64U {
    #[inline]
    fn from(v: v128) -> Self {
        V2x64U(v)
    }
}

impl AddAssign for V2x64U {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        self.add_assign(other)
    }
}

impl SubAssign for V2x64U {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        self.sub_assign(other)
    }
}

impl BitAndAssign for V2x64U {
    #[inline]
    fn bitand_assign(&mut self, other: Self) {
        self.bitand_assign(other)
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
        self.bitor_assign(other)
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
        self.bitxor_assign(other)
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

impl ShlAssign<u32> for V2x64U {
    #[inline]
    fn shl_assign(&mut self, count: u32) {
        self.shl_assign(count)
    }
}

impl ShrAssign<u32> for V2x64U {
    #[inline]
    fn shr_assign(&mut self, count: u32) {
        self.shr_assign(count)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_as_arr() {
        let x = V2x64U::new(55, 1);
        let res = x.as_arr();
        assert_eq!(res, [1, 55]);
    }

    #[wasm_bindgen_test]
    fn test_rotate_by_32() {
        let x = V2x64U::new(0x0264_432C_CD8A_70E0, 0x0B28_E3EF_EBB3_172D);
        let y = x.rotate_by_32();
        let res = y.as_arr();
        assert_eq!(res, [0xEBB3_172D_0B28_E3EF, 0xCD8A_70E0_0264_432C]);
    }

    #[wasm_bindgen_test]
    fn test_add() {
        let x = V2x64U::new(55, 1);
        let y = V2x64U::new(0x0264_432C_CD8A_70E0, 0x0B28E_3EFE_BB3_172D);
        let z = x + y;
        assert_eq!(z.as_arr(), [0x0B28_E3EF_EBB3_172E, 0x2644_32CC_D8A7_117]);
    }

    #[wasm_bindgen_test]
    fn test_mm_srli_epi64() {
        let x = V2x64U::new(0x0264_432C_CD8A_70E0, 0x0B28E_3EFE_BB3_172D);
        let y = V2x64U::from(_mm_srli_epi64(x.0, 33));
        assert_eq!(y.as_arr(), [0x0000_0000_0594_71F7, 0x0000_0000_0132_2196]);
    }

    #[wasm_bindgen_test]
    fn test_zipper_merge() {
        let x = V2x64U::new(0x0264_432C_CD8A_70E0, 0x0B28_E3EF_EBB3_172D);
        let y = WasmHash::zipper_merge(&x);
        assert_eq!(y.as_arr(), [0x2D02_1764_E3B3_2CEB, 0x0BE0_2870_438A_EFCD]);
    }

    #[wasm_bindgen_test]
    fn test_mm_mul_epu32() {
        let x = V2x64U::new(0x0264_432C_CD8A_70E0, 0x0B28_E3EF_EBB3_172D);
        let y = V2x64U::new(0x0B28_E3EF_EBB3_172D, 0x0264_432C_CD8A_70E0);
        let z = V2x64U::from(_mm_mul_epu32(x.0, y.0));
        assert_eq!(z.as_arr(), [0xBD3D_E006_1E19_F760, 0xBD3D_E006_1E19_F760]);
    }

    #[wasm_bindgen_test]
    fn test_mm_slli_si128_8() {
        let x = V2x64U::new(0, 0xFFFF_FFFF);
        let y = V2x64U::from(_mm_slli_si128_8(x.0));
        assert_eq!(y.as_arr(), [0, 0xFFFF_FFFF]);
    }
}
