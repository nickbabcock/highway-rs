use super::{v2x64u::V2x64U, v4x64u::V4x64U};
use crate::internal::unordered_load3;
use crate::internal::{HashPacket, PACKET_SIZE};
use crate::key::Key;
use crate::traits::HighwayHash;
use core::arch::x86_64::*;

/// AVX empowered implementation that will only work on `x86_64` with avx2 enabled at the CPU
/// level.
#[derive(Debug, Default, Clone)]
pub struct AvxHash {
    v0: V4x64U,
    v1: V4x64U,
    mul0: V4x64U,
    mul1: V4x64U,
    buffer: HashPacket,
}

impl HighwayHash for AvxHash {
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

impl AvxHash {
    /// Creates a new `AvxHash` while circumventing the runtime check for avx2.
    ///
    /// # Safety
    ///
    /// If called on a machine without avx2, a segfault will occur. Only use if you have
    /// control over the deployment environment and have either benchmarked that the runtime
    /// check is significant or are unable to check for avx2 capabilities
    #[must_use]
    #[target_feature(enable = "avx2")]
    pub unsafe fn force_new(key: Key) -> Self {
        let mul0 = V4x64U::new(
            0x243f_6a88_85a3_08d3,
            0x1319_8a2e_0370_7344,
            0xa409_3822_299f_31d0,
            0xdbe6_d5d5_fe4c_ce2f,
        );
        let mul1 = V4x64U::new(
            0x4528_21e6_38d0_1377,
            0xbe54_66cf_34e9_0c6c,
            0xc0ac_f169_b5f1_8a8c,
            0x3bd3_9e10_cb0e_f593,
        );

        let key = V4x64U::from(_mm256_load_si256(key.0.as_ptr().cast::<__m256i>()));

        AvxHash {
            v0: key ^ mul0,
            v1: key.rotate_by_32() ^ mul1,
            mul0,
            mul1,
            buffer: HashPacket::default(),
        }
    }

    /// Creates a new `AvxHash` if the avx2 feature is detected.
    #[must_use]
    pub fn new(key: Key) -> Option<Self> {
        #[cfg(feature = "std")]
        {
            if is_x86_feature_detected!("avx2") {
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

    #[target_feature(enable = "avx2")]
    unsafe fn finalize64(&mut self) -> u64 {
        if !self.buffer.is_empty() {
            self.update_remainder();
        }

        for _i in 0..4 {
            let permuted = AvxHash::permute(&self.v0);
            self.update(permuted);
        }

        let sum0 = V2x64U::from(_mm256_castsi256_si128((self.v0 + self.mul0).0));
        let sum1 = V2x64U::from(_mm256_castsi256_si128((self.v1 + self.mul1).0));
        let hash = sum0 + sum1;
        let mut result: u64 = 0;
        // Each lane is sufficiently mixed, so just truncate to 64 bits.
        _mm_storel_epi64(core::ptr::addr_of_mut!(result).cast::<__m128i>(), hash.0);
        result
    }

    #[target_feature(enable = "avx2")]
    unsafe fn finalize128(&mut self) -> [u64; 2] {
        if !self.buffer.is_empty() {
            self.update_remainder();
        }

        for _i in 0..6 {
            let permuted = AvxHash::permute(&self.v0);
            self.update(permuted);
        }

        let sum0 = V2x64U::from(_mm256_castsi256_si128((self.v0 + self.mul0).0));
        let sum1 = V2x64U::from(_mm256_extracti128_si256((self.v1 + self.mul1).0, 1));
        let hash = sum0 + sum1;
        let mut result: [u64; 2] = [0; 2];
        _mm_storeu_si128(result.as_mut_ptr().cast::<__m128i>(), hash.0);
        result
    }

    #[target_feature(enable = "avx2")]
    unsafe fn finalize256(&mut self) -> [u64; 4] {
        if !self.buffer.is_empty() {
            self.update_remainder();
        }

        for _i in 0..10 {
            let permuted = AvxHash::permute(&self.v0);
            self.update(permuted);
        }

        let sum0 = self.v0 + self.mul0;
        let sum1 = self.v1 + self.mul1;
        let hash = AvxHash::modular_reduction(&sum1, &sum0);
        let mut result: [u64; 4] = [0; 4];
        _mm256_storeu_si256(result.as_mut_ptr().cast::<__m256i>(), hash.0);
        result
    }

    #[inline]
    #[target_feature(enable = "avx2")]
    unsafe fn data_to_lanes(packet: &[u8]) -> V4x64U {
        V4x64U::from(_mm256_loadu_si256(packet.as_ptr().cast::<__m256i>()))
    }

    #[target_feature(enable = "avx2")]
    unsafe fn remainder(bytes: &[u8]) -> V4x64U {
        let size_mod32 = bytes.len();
        let size256 = _mm256_broadcastd_epi32(_mm_cvtsi64_si128(size_mod32 as i64));
        let size_mod4 = size_mod32 & 3;
        let size = _mm256_castsi256_si128(size256);
        if size_mod32 & 16 != 0 {
            let packetL = _mm_load_si128(bytes.as_ptr().cast::<__m128i>());
            let int_mask = _mm_cmpgt_epi32(size, _mm_set_epi32(31, 27, 23, 19));
            let int_lanes = _mm_maskload_epi32(bytes.as_ptr().offset(16).cast::<i32>(), int_mask);
            let remainder = &bytes[(size_mod32 & !3) + size_mod4 - 4..];
            let last4 =
                i32::from_le_bytes([remainder[0], remainder[1], remainder[2], remainder[3]]);
            let packetH = _mm_insert_epi32(int_lanes, last4, 3);
            let packetL256 = _mm256_castsi128_si256(packetL);
            let packet = _mm256_inserti128_si256(packetL256, packetH, 1);
            V4x64U::from(packet)
        } else {
            let int_mask = _mm_cmpgt_epi32(size, _mm_set_epi32(15, 11, 7, 3));
            let packetL = _mm_maskload_epi32(bytes.as_ptr().cast::<i32>(), int_mask);
            let remainder = &bytes[size_mod32 & !3..];
            let last3 = unordered_load3(remainder);
            let packetH = _mm_cvtsi64_si128(last3 as i64);
            let packetL256 = _mm256_castsi128_si256(packetL);
            let packet = _mm256_inserti128_si256(packetL256, packetH, 1);
            V4x64U::from(packet)
        }
    }

    #[target_feature(enable = "avx2")]
    unsafe fn update_remainder(&mut self) {
        let size = self.buffer.len();
        let size256 = _mm256_broadcastd_epi32(_mm_cvtsi64_si128(size as i64));
        self.v0 += V4x64U::from(size256);
        let shifted_left = V4x64U::from(_mm256_sllv_epi32(self.v1.0, size256));
        let tip = _mm256_broadcastd_epi32(_mm_cvtsi32_si128(32));
        let shifted_right =
            V4x64U::from(_mm256_srlv_epi32(self.v1.0, _mm256_sub_epi32(tip, size256)));
        self.v1 = shifted_left | shifted_right;

        let packet = AvxHash::remainder(self.buffer.as_slice());
        self.update(packet);
    }

    #[target_feature(enable = "avx2")]
    unsafe fn zipper_merge(v: &V4x64U) -> V4x64U {
        let hi = 0x0708_0609_0D0A_040B;
        let lo = 0x000F_010E_0502_0C03;
        v.shuffle(&V4x64U::new(hi, lo, hi, lo))
    }

    #[target_feature(enable = "avx2")]
    unsafe fn update(&mut self, packet: V4x64U) {
        self.v1 += packet;
        self.v1 += self.mul0;
        self.mul0 ^= self.v1.mul_low32(&self.v0.shr_by_32());
        self.v0 += self.mul1;
        self.mul1 ^= self.v0.mul_low32(&self.v1.shr_by_32());
        self.v0 += AvxHash::zipper_merge(&self.v1);
        self.v1 += AvxHash::zipper_merge(&self.v0);
    }

    #[target_feature(enable = "avx2")]
    unsafe fn permute(v: &V4x64U) -> V4x64U {
        let indices = V4x64U::new(
            0x0000_0002_0000_0003,
            0x0000_0000_0000_0001,
            0x0000_0006_0000_0007,
            0x0000_0004_0000_0005,
        );

        V4x64U::from(_mm256_permutevar8x32_epi32(v.0, indices.0))
    }

    #[target_feature(enable = "avx2")]
    unsafe fn modular_reduction(x: &V4x64U, init: &V4x64U) -> V4x64U {
        let top_bits2 = V4x64U::from(_mm256_srli_epi64(x.0, 62));
        let ones = V4x64U::from(_mm256_cmpeq_epi64(x.0, x.0));
        let shifted1_unmasked = *x + *x;
        let top_bits1 = V4x64U::from(_mm256_srli_epi64(x.0, 63));
        let upper_8bytes = V4x64U::from(_mm256_slli_si256(ones.0, 8));
        let shifted2 = shifted1_unmasked + shifted1_unmasked;
        let upper_bit_of_128 = V4x64U::from(_mm256_slli_epi64(upper_8bytes.0, 63));
        let zero = V4x64U::from(_mm256_setzero_si256());
        let new_low_bits2 = V4x64U::from(_mm256_unpacklo_epi64(zero.0, top_bits2.0));
        let shifted1 = shifted1_unmasked.and_not(&upper_bit_of_128);
        let new_low_bits1 = V4x64U::from(_mm256_unpacklo_epi64(zero.0, top_bits1.0));

        *init ^ shifted2 ^ new_low_bits2 ^ shifted1 ^ new_low_bits1
    }

    #[target_feature(enable = "avx2")]
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

impl_write!(AvxHash);
impl_hasher!(AvxHash);
