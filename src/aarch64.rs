use crate::internal::{unordered_load3, HashPacket, PACKET_SIZE};
use crate::{HighwayHash, Key};
use core::arch::aarch64::*;
use core::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, SubAssign,
};

/// HighwayHash powered by Neon instructions
#[derive(Debug, Default, Clone)]
pub struct NeonHash {
    buffer: HashPacket,
    v0L: V2x64U,
    v0H: V2x64U,
    v1L: V2x64U,
    v1H: V2x64U,
    mul0L: V2x64U,
    mul0H: V2x64U,
    mul1L: V2x64U,
    mul1H: V2x64U,
}

impl HighwayHash for NeonHash {
    #[inline]
    fn append(&mut self, data: &[u8]) {
        unsafe {
            self.append(data);
        }
    }

    #[inline]
    fn finalize64(mut self) -> u64 {
        unsafe { Self::finalize64(&mut self) }
    }

    #[inline]
    fn finalize128(mut self) -> [u64; 2] {
        unsafe { Self::finalize128(&mut self) }
    }

    #[inline]
    fn finalize256(mut self) -> [u64; 4] {
        unsafe { Self::finalize256(&mut self) }
    }
}

impl NeonHash {
    /// Creates a new `NeonHash` while circumventing any runtime checks.
    pub unsafe fn force_new(key: Key) -> Self {
        let init0L = V2x64U::new(0xa409_3822_299f_31d0, 0xdbe6_d5d5_fe4c_ce2f);
        let init0H = V2x64U::new(0x243f_6a88_85a3_08d3, 0x1319_8a2e_0370_7344);
        let init1L = V2x64U::new(0xc0ac_f169_b5f1_8a8c, 0x3bd3_9e10_cb0e_f593);
        let init1H = V2x64U::new(0x4528_21e6_38d0_1377, 0xbe54_66cf_34e9_0c6c);
        let keyL = V2x64U::new(key[1], key[0]);
        let keyH = V2x64U::new(key[3], key[2]);

        NeonHash {
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

    unsafe fn zipper_merge(v: &V2x64U) -> V2x64U {
        let pos = [3, 12, 2, 5, 14, 1, 15, 0, 11, 4, 10, 13, 9, 6, 8, 7];
        let tbl = vld1q_u8(pos.as_ptr());
        let lookup = vqtbl1q_u8(vreinterpretq_u8_u64(v.0), tbl);
        V2x64U::from(lookup)
    }

    unsafe fn update(&mut self, (packetH, packetL): (V2x64U, V2x64U)) {
        self.v1L += packetL;
        self.v1H += packetH;
        self.v1L += self.mul0L;
        self.v1H += self.mul0H;
        self.mul0L ^= V2x64U(vmull_u32(
            vmovn_u64(self.v1L.0),
            vshrn_n_u64(self.v0L.0, 32),
        ));
        self.mul0H ^= V2x64U(vmull_u32(
            vmovn_u64(self.v1H.0),
            vshrn_n_u64(self.v0H.0, 32),
        ));
        self.v0L += self.mul1L;
        self.v0H += self.mul1H;
        self.mul1L ^= V2x64U(vmull_u32(
            vmovn_u64(self.v0L.0),
            vshrn_n_u64(self.v1L.0, 32),
        ));
        self.mul1H ^= V2x64U(vmull_u32(
            vmovn_u64(self.v0H.0),
            vshrn_n_u64(self.v1H.0, 32),
        ));
        self.v0L += NeonHash::zipper_merge(&self.v1L);
        self.v0H += NeonHash::zipper_merge(&self.v1H);
        self.v1L += NeonHash::zipper_merge(&self.v0L);
        self.v1H += NeonHash::zipper_merge(&self.v0H);
    }

    unsafe fn permute_and_update(&mut self) {
        let low = self.v0L.rotate_by_32();
        let high = self.v0H.rotate_by_32();
        self.update((low, high));
    }

    pub(crate) unsafe fn finalize64(&mut self) -> u64 {
        if !self.buffer.is_empty() {
            self.update_remainder();
        }

        for _i in 0..4 {
            self.permute_and_update();
        }

        let sum0 = self.v0L + self.mul0L;
        let sum1 = self.v1L + self.mul1L;
        let hash = sum0 + sum1;
        hash.as_arr()[0]
    }

    pub(crate) unsafe fn finalize128(&mut self) -> [u64; 2] {
        if !self.buffer.is_empty() {
            self.update_remainder();
        }

        for _i in 0..6 {
            self.permute_and_update();
        }

        let sum0 = self.v0L + self.mul0L;
        let sum1 = self.v1H + self.mul1H;
        let hash = sum0 + sum1;
        hash.as_arr()
    }

    pub(crate) unsafe fn finalize256(&mut self) -> [u64; 4] {
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

        let hashL = NeonHash::modular_reduction(&sum1L, &sum0L).as_arr();
        let hashH = NeonHash::modular_reduction(&sum1H, &sum0H).as_arr();

        [hashL[0], hashL[1], hashH[0], hashH[1]]
    }

    unsafe fn modular_reduction(x: &V2x64U, init: &V2x64U) -> V2x64U {
        let zero = vdupq_n_u32(0);
        let sign_bit128 = V2x64U::from(vsetq_lane_u32(0x8000_0000_u32, zero, 3));
        let top_bits2 = V2x64U::from(vshrq_n_u64(x.0, 62));
        let shifted1_unmasked = *x + *x;
        let top_bits1 = V2x64U::from(vshrq_n_u64(x.0, 63));
        let shifted2 = shifted1_unmasked + shifted1_unmasked;
        let new_low_bits2 = V2x64U::from(_mm_slli_si128_8(top_bits2.0));
        let shifted1 = shifted1_unmasked.and_not(&sign_bit128);
        let new_low_bits1 = V2x64U::from(_mm_slli_si128_8(top_bits1.0));
        *init ^ shifted2 ^ new_low_bits2 ^ shifted1 ^ new_low_bits1
    }

    unsafe fn load_multiple_of_four(bytes: &[u8], size: u64) -> V2x64U {
        let mut data = bytes;
        let mut mask4 = V2x64U::new(0, 0xFFFF_FFFF);
        let mut ret = if bytes.len() >= 8 {
            mask4 = V2x64U::from(_mm_slli_si128_8(mask4.0));
            data = &bytes[8..];
            let lo = u64::from_le_bytes(take::<8>(bytes));
            V2x64U::new(0, lo)
        } else {
            V2x64U::new(0, 0)
        };

        if size & 4 != 0 {
            let last4 = u32::from_le_bytes(take::<4>(data));
            let broadcast = V2x64U::from(vdupq_n_u32(last4));
            ret |= broadcast & mask4;
        }

        ret
    }

    unsafe fn remainder(bytes: &[u8]) -> (V2x64U, V2x64U) {
        let size_mod32 = bytes.len();
        let size_mod4 = size_mod32 & 3;
        if size_mod32 & 16 != 0 {
            let packetL = V2x64U::from(vld1q_u8(bytes.as_ptr()));
            let packett = NeonHash::load_multiple_of_four(&bytes[16..], size_mod32 as u64);
            let remainder = &bytes[(size_mod32 & !3) + size_mod4 - 4..];
            let last4 =
                u32::from_le_bytes([remainder[0], remainder[1], remainder[2], remainder[3]]);
            let packetH = V2x64U::from(vsetq_lane_u32(last4, vreinterpretq_u32_u64(packett.0), 3));
            (packetH, packetL)
        } else {
            let remainder = &bytes[size_mod32 & !3..];
            let packetL = NeonHash::load_multiple_of_four(bytes, size_mod32 as u64);
            let last4 = unordered_load3(remainder);
            let packetH = V2x64U::new(0, last4);
            (packetH, packetL)
        }
    }

    unsafe fn update_remainder(&mut self) {
        let size = self.buffer.len() as i32;
        let vsize_mod32 = V2x64U::from(vdupq_n_s32(size));
        self.v0L += vsize_mod32;
        self.v0H += vsize_mod32;
        self.rotate_32_by(size);
        let packet = NeonHash::remainder(self.buffer.as_slice());
        self.update(packet);
    }

    unsafe fn rotate_32_by(&mut self, count: i32) {
        let vL = &mut self.v1L;
        let vH = &mut self.v1H;
        let count_left = vdupq_n_s32(count);
        let count_right = vdupq_n_s32(count + (!32 + 1));

        let shifted_leftL = V2x64U::from(vshlq_u32(vreinterpretq_u32_u64(vL.0), count_left));
        let shifted_leftH = V2x64U::from(vshlq_u32(vreinterpretq_u32_u64(vH.0), count_left));
        let shifted_rightL = V2x64U::from(vshlq_u32(vreinterpretq_u32_u64(vL.0), count_right));
        let shifted_rightH = V2x64U::from(vshlq_u32(vreinterpretq_u32_u64(vH.0), count_right));
        *vL = shifted_leftL | shifted_rightL;
        *vH = shifted_leftH | shifted_rightH;
    }

    #[inline]
    unsafe fn data_to_lanes(packet: &[u8]) -> (V2x64U, V2x64U) {
        let ptr = packet.as_ptr();
        let packetL = V2x64U::from(vld1q_u8(ptr));
        let packetH = V2x64U::from(vld1q_u8(ptr.offset(16)));

        (packetH, packetL)
    }

    unsafe fn append(&mut self, data: &[u8]) {
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

#[inline]
fn take<const N: usize>(data: &[u8]) -> [u8; N] {
    debug_assert!(data.len() >= N);
    unsafe { *(data.as_ptr() as *const [u8; N]) }
}

#[inline]
unsafe fn _mm_slli_si128_8(a: uint64x2_t) -> uint64x2_t {
    // aka _mm_bslli_si128_8
    let tmp = vreinterpretq_u8_u64(a);
    let rotated = vextq_u8(vdupq_n_u8(0), tmp, 8);
    vreinterpretq_u64_u8(rotated)
}

#[derive(Clone, Copy)]
pub struct V2x64U(pub uint64x2_t);

impl Default for V2x64U {
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
    unsafe fn zeroed() -> Self {
        V2x64U(vdupq_n_u64(0))
    }

    #[inline]
    pub unsafe fn new(hi: u64, low: u64) -> Self {
        V2x64U(vld1q_u64([low, hi].as_ptr()))
    }

    pub unsafe fn as_arr(&self) -> [u64; 2] {
        let mut arr: [u64; 2] = [0, 0];
        vst1q_u64(arr.as_mut_ptr(), self.0);
        arr
    }

    #[inline]
    pub unsafe fn rotate_by_32(&self) -> Self {
        let tmp = vreinterpretq_u32_u64(self.0);
        let rotated = vrev64q_u32(tmp);
        V2x64U(vreinterpretq_u64_u32(rotated))
    }

    #[inline]
    pub unsafe fn and_not(&self, neg_mask: &V2x64U) -> Self {
        V2x64U::from(vbicq_u64(self.0, neg_mask.0))
    }

    #[inline]
    unsafe fn add_assign(&mut self, other: Self) {
        self.0 = vaddq_u64(self.0, other.0)
    }

    #[inline]
    unsafe fn sub_assign(&mut self, other: Self) {
        self.0 = vsubq_u64(self.0, other.0)
    }

    #[inline]
    unsafe fn bitand_assign(&mut self, other: Self) {
        self.0 = vandq_u64(self.0, other.0)
    }

    #[inline]
    unsafe fn bitor_assign(&mut self, other: Self) {
        self.0 = vorrq_u64(self.0, other.0)
    }

    #[inline]
    unsafe fn bitxor_assign(&mut self, other: Self) {
        self.0 = veorq_u64(self.0, other.0)
    }
}

impl From<uint64x2_t> for V2x64U {
    #[inline]
    fn from(v: uint64x2_t) -> Self {
        V2x64U(v)
    }
}

impl From<uint32x4_t> for V2x64U {
    #[inline]
    fn from(v: uint32x4_t) -> Self {
        V2x64U(unsafe { vreinterpretq_u64_u32(v) })
    }
}

impl From<int32x4_t> for V2x64U {
    #[inline]
    fn from(v: int32x4_t) -> Self {
        V2x64U(unsafe { vreinterpretq_u64_s32(v) })
    }
}

impl From<uint16x8_t> for V2x64U {
    #[inline]
    fn from(v: uint16x8_t) -> Self {
        V2x64U(unsafe { vreinterpretq_u64_u16(v) })
    }
}

impl From<uint8x16_t> for V2x64U {
    #[inline]
    fn from(v: uint8x16_t) -> Self {
        V2x64U(unsafe { vreinterpretq_u64_u8(v) })
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

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_as_arr() {
        unsafe {
            let x = V2x64U::new(55, 1);
            let res = x.as_arr();
            assert_eq!(res, [1, 55]);
        }
    }

    #[test]
    fn test_rotate_by_32() {
        unsafe {
            let x = V2x64U::new(0x0264_432C_CD8A_70E0, 0x0B28_E3EF_EBB3_172D);
            let y = x.rotate_by_32();
            let res = y.as_arr();
            assert_eq!(res, [0xEBB3_172D_0B28_E3EF, 0xCD8A_70E0_0264_432C]);
        }
    }

    #[test]
    fn test_add() {
        unsafe {
            let x = V2x64U::new(55, 1);
            let y = V2x64U::new(0x0264_432C_CD8A_70E0, 0x0B28_E3EF_EBB3_172D);
            let z = x + y;
            assert_eq!(z.as_arr(), [0x0B28_E3EF_EBB3_172E, 0x0264_432C_CD8A_7117]);
        }
    }

    #[test]
    fn test_mm_slli_si128_8() {
        unsafe {
            let x = V2x64U::new(0, 0xFFFF_FFFF);
            let y = V2x64U::from(_mm_slli_si128_8(x.0));
            assert_eq!(y.as_arr(), [0, 0xFFFF_FFFF]);
        }
    }
}
