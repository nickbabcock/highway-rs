#![allow(unsafe_code)]
use super::v2x64u::V2x64U;
use crate::internal::unordered_load3;
use crate::internal::{HashPacket, PACKET_SIZE};
use crate::key::Key;
use crate::traits::HighwayHash;
use crate::PortableHash;
use core::arch::x86_64::*;

/// SSE empowered implementation that will only work on `x86_64` with sse 4.1 enabled at the CPU
/// level.
#[derive(Debug, Default, Clone)]
pub struct SseHash {
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

impl HighwayHash for SseHash {
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

    #[inline]
    fn checkpoint(&self) -> [u8; 164] {
        let mut v0 = [0u64; 4];
        v0[..2].copy_from_slice(unsafe { &self.v0L.as_arr() });
        v0[2..].copy_from_slice(unsafe { &self.v0H.as_arr() });

        let mut v1 = [0u64; 4];
        v1[..2].copy_from_slice(unsafe { &self.v1L.as_arr() });
        v1[2..].copy_from_slice(unsafe { &self.v1H.as_arr() });

        let mut mul0 = [0u64; 4];
        mul0[..2].copy_from_slice(unsafe { &self.mul0L.as_arr() });
        mul0[2..].copy_from_slice(unsafe { &self.mul0H.as_arr() });

        let mut mul1 = [0u64; 4];
        mul1[..2].copy_from_slice(unsafe { &self.mul1L.as_arr() });
        mul1[2..].copy_from_slice(unsafe { &self.mul1H.as_arr() });

        PortableHash {
            v0,
            v1,
            mul0,
            mul1,
            buffer: self.buffer,
        }
        .checkpoint()
    }
}

impl SseHash {
    /// Creates a new `SseHash` while circumventing the runtime check for sse4.1.
    ///
    /// # Safety
    ///
    /// If called on a machine without sse4.1, a segfault will occur. Only use if you have
    /// control over the deployment environment and have either benchmarked that the runtime
    /// check is significant or are unable to check for sse4.1 capabilities
    #[must_use]
    #[target_feature(enable = "sse4.1")]
    pub unsafe fn force_new(key: Key) -> Self {
        let init0L = V2x64U::new(0xa409_3822_299f_31d0, 0xdbe6_d5d5_fe4c_ce2f);
        let init0H = V2x64U::new(0x243f_6a88_85a3_08d3, 0x1319_8a2e_0370_7344);
        let init1L = V2x64U::new(0xc0ac_f169_b5f1_8a8c, 0x3bd3_9e10_cb0e_f593);
        let init1H = V2x64U::new(0x4528_21e6_38d0_1377, 0xbe54_66cf_34e9_0c6c);
        let key_ptr = key.0.as_ptr().cast::<__m128i>();
        let keyL = V2x64U::from(_mm_loadu_si128(key_ptr));
        let keyH = V2x64U::from(_mm_loadu_si128(key_ptr.add(1)));

        SseHash {
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

    /// Create a new `SseHash` if the sse4.1 feature is detected
    #[must_use]
    pub fn new(key: Key) -> Option<Self> {
        #[cfg(feature = "std")]
        {
            if is_x86_feature_detected!("sse4.1") {
                Some(unsafe { Self::force_new(key) })
            } else {
                None
            }
        }

        #[cfg(not(feature = "std"))]
        {
            let _key = key;
            None
        }
    }

    /// Creates a new `SseHash` from a checkpoint while circumventing the runtime check for sse4.1.
    ///
    /// # Safety
    ///
    /// See [`Self::force_new`] for safety concerns.
    #[must_use]
    #[target_feature(enable = "sse4.1")]
    pub unsafe fn force_from_checkpoint(data: [u8; 164]) -> Self {
        let portable = PortableHash::from_checkpoint(data);
        SseHash {
            v0L: V2x64U::new(portable.v0[1], portable.v0[0]),
            v0H: V2x64U::new(portable.v0[3], portable.v0[2]),
            v1L: V2x64U::new(portable.v1[1], portable.v1[0]),
            v1H: V2x64U::new(portable.v1[3], portable.v1[2]),
            mul0L: V2x64U::new(portable.mul0[1], portable.mul0[0]),
            mul0H: V2x64U::new(portable.mul0[3], portable.mul0[2]),
            mul1L: V2x64U::new(portable.mul1[1], portable.mul1[0]),
            mul1H: V2x64U::new(portable.mul1[3], portable.mul1[2]),
            buffer: portable.buffer,
        }
    }

    /// Create a new `SseHash` from a checkpoint if the sse4.1 feature is detected
    #[must_use]
    pub fn from_checkpoint(data: [u8; 164]) -> Option<Self> {
        #[cfg(feature = "std")]
        {
            if is_x86_feature_detected!("sse4.1") {
                Some(unsafe { Self::force_from_checkpoint(data) })
            } else {
                None
            }
        }

        #[cfg(not(feature = "std"))]
        {
            let _ = data;
            None
        }
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn zipper_merge(v: &V2x64U) -> V2x64U {
        v.shuffle(&V2x64U::new(0x0708_0609_0D0A_040B, 0x000F_010E_0502_0C03))
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn update(&mut self, (packetH, packetL): (V2x64U, V2x64U)) {
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
        self.v0L += SseHash::zipper_merge(&self.v1L);
        self.v0H += SseHash::zipper_merge(&self.v1H);
        self.v1L += SseHash::zipper_merge(&self.v0L);
        self.v1H += SseHash::zipper_merge(&self.v0H);
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn permute_and_update(&mut self) {
        let low = self.v0L.rotate_by_32();
        let high = self.v0H.rotate_by_32();
        self.update((low, high));
    }

    #[target_feature(enable = "sse4.1")]
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
        let mut result: u64 = 0;
        _mm_storel_epi64(core::ptr::addr_of_mut!(result).cast::<__m128i>(), hash.0);
        result
    }

    #[target_feature(enable = "sse4.1")]
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
        let mut result: [u64; 2] = [0; 2];
        _mm_storeu_si128(result.as_mut_ptr().cast::<__m128i>(), hash.0);
        result
    }

    #[target_feature(enable = "sse4.1")]
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
        let hashL = SseHash::modular_reduction(&sum1L, &sum0L);
        let hashH = SseHash::modular_reduction(&sum1H, &sum0H);
        let mut result: [u64; 4] = [0; 4];
        let ptr = result.as_mut_ptr().cast::<__m128i>();
        _mm_storeu_si128(ptr, hashL.0);
        _mm_storeu_si128(ptr.add(1), hashH.0);
        result
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn modular_reduction(x: &V2x64U, init: &V2x64U) -> V2x64U {
        let zero = V2x64U::default();
        let sign_bit128 = V2x64U::from(_mm_insert_epi32(zero.0, 0x8000_0000_u32 as i32, 3));
        let top_bits2 = V2x64U::from(_mm_srli_epi64(x.0, 62));
        let shifted1_unmasked = *x + *x;
        let top_bits1 = V2x64U::from(_mm_srli_epi64(x.0, 63));
        let shifted2 = shifted1_unmasked + shifted1_unmasked;
        let new_low_bits2 = V2x64U::from(_mm_slli_si128(top_bits2.0, 8));
        let shifted1 = shifted1_unmasked.and_not(&sign_bit128);
        let new_low_bits1 = V2x64U::from(_mm_slli_si128(top_bits1.0, 8));
        *init ^ shifted2 ^ new_low_bits2 ^ shifted1 ^ new_low_bits1
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn load_multiple_of_four(bytes: &[u8]) -> V2x64U {
        let mut data = bytes;
        let mut mask4 = V2x64U::from(_mm_cvtsi64_si128(0xFFFF_FFFF));
        let mut ret = if bytes.len() >= 8 {
            mask4 = V2x64U::from(_mm_slli_si128(mask4.0, 8));
            data = &bytes[8..];
            V2x64U::from(_mm_loadl_epi64(bytes.as_ptr().cast::<__m128i>()))
        } else {
            V2x64U::new(0, 0)
        };

        if let Some(d) = data.get(..4) {
            let last4 = i32::from_le_bytes([d[0], d[1], d[2], d[3]]);
            let broadcast = V2x64U::from(_mm_set1_epi32(last4));
            ret |= broadcast & mask4;
        }

        ret
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn remainder(bytes: &[u8]) -> (V2x64U, V2x64U) {
        let size_mod32 = bytes.len();
        let size_mod4 = size_mod32 & 3;
        if size_mod32 & 16 != 0 {
            let packetL = V2x64U::from(_mm_loadu_si128(bytes.as_ptr().cast::<__m128i>()));
            let packett = SseHash::load_multiple_of_four(&bytes[16..]);
            let remainder = &bytes[(size_mod32 & !3) + size_mod4 - 4..];
            let last4 =
                i32::from_le_bytes([remainder[0], remainder[1], remainder[2], remainder[3]]);
            let packetH = V2x64U::from(_mm_insert_epi32(packett.0, last4, 3));
            (packetH, packetL)
        } else {
            let remainder = &bytes[size_mod32 & !3..];
            let packetL = SseHash::load_multiple_of_four(bytes);
            let last4 = unordered_load3(remainder);
            let packetH = V2x64U::from(_mm_cvtsi64_si128(last4 as i64));
            (packetH, packetL)
        }
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn update_remainder(&mut self) {
        let size = self.buffer.len();
        let vsize_mod32 = _mm_set1_epi32(size as i32);
        self.v0L += V2x64U::from(vsize_mod32);
        self.v0H += V2x64U::from(vsize_mod32);
        self.rotate_32_by(size as i64);
        let packet = SseHash::remainder(self.buffer.as_slice());
        self.update(packet);
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn rotate_32_by(&mut self, count: i64) {
        let vL = &mut self.v1L;
        let vH = &mut self.v1H;
        let count_left = _mm_cvtsi64_si128(count);
        let count_right = _mm_cvtsi64_si128(32 - count);
        let shifted_leftL = V2x64U::from(_mm_sll_epi32(vL.0, count_left));
        let shifted_leftH = V2x64U::from(_mm_sll_epi32(vH.0, count_left));
        let shifted_rightL = V2x64U::from(_mm_srl_epi32(vL.0, count_right));
        let shifted_rightH = V2x64U::from(_mm_srl_epi32(vH.0, count_right));
        *vL = shifted_leftL | shifted_rightL;
        *vH = shifted_leftH | shifted_rightH;
    }

    #[inline]
    #[target_feature(enable = "sse4.1")]
    unsafe fn data_to_lanes(packet: &[u8]) -> (V2x64U, V2x64U) {
        let ptr = packet.as_ptr().cast::<__m128i>();
        let packetL = V2x64U::from(_mm_loadu_si128(ptr));
        let packetH = V2x64U::from(_mm_loadu_si128(ptr.add(1)));

        (packetH, packetL)
    }

    #[target_feature(enable = "sse4.1")]
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

impl_write!(SseHash);
impl_hasher!(SseHash);

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(miri, ignore)]
    #[test]
    fn test_zipper_merge() {
        unsafe {
            let x = V2x64U::new(0x0264_432C_CD8A_70E0, 0x0B28_E3EF_EBB3_172D);
            let y = SseHash::zipper_merge(&x);
            assert_eq!(y.as_arr(), [0x2D02_1764_E3B3_2CEB, 0x0BE0_2870_438A_EFCD]);
        }
    }
}
