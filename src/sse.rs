use byteorder::{ByteOrder, LE};
use internal::unordered_load3;
use internal::{Filled, HashPacket, PACKET_SIZE};
use key::Key;
use std::arch::x86_64::*;
use traits::HighwayHash;
use v2x64u::V2x64U;

/// SSE empowered implementation that will only work on `x86_64` with sse 4.1 enabled at the CPU
/// level.
#[derive(Debug, Default)]
pub struct SseHash {
    key: Key,
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

impl HighwayHash for SseHash {
    fn hash64(mut self, data: &[u8]) -> u64 {
        unsafe {
            self.append(data);
            self.finalize64()
        }
    }

    fn hash128(mut self, data: &[u8]) -> u128 {
        unsafe {
            self.append(data);
            self.finalize128()
        }
    }

    fn hash256(mut self, data: &[u8]) -> (u128, u128) {
        unsafe {
            self.append(data);
            self.finalize256()
        }
    }

    fn append(&mut self, data: &[u8]) {
        unsafe {
            self.append(data);
        }
    }

    fn finalize64(mut self) -> u64 {
        unsafe { Self::finalize64(&mut self) }
    }

    fn finalize128(mut self) -> u128 {
        unsafe { Self::finalize128(&mut self) }
    }

    fn finalize256(mut self) -> (u128, u128) {
        unsafe { Self::finalize256(&mut self) }
    }
}

impl SseHash {
    /// Creates a new `SseHash` while circumventing the runtime check for sse4.1. This function is
    /// unsafe! If will cause a segfault if sse4.1 is not enabled. Only use this function if you have
    /// benchmarked that the runtime check is significant and you know sse4.1 is already enabled.
    pub unsafe fn force_new(key: &Key) -> Self {
        let mut h = SseHash {
            key: key.clone(),
            ..Default::default()
        };
        h.reset();
        h
    }

    /// Create a new `SseHash` if the sse4.1 feature is detected
    pub fn new(key: &Key) -> Option<Self> {
        if is_x86_feature_detected!("sse4.1") {
            Some(unsafe { Self::force_new(key) })
        } else {
            None
        }
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn reset(&mut self) {
        let init0L = V2x64U::new(0xa4093822299f31d0, 0xdbe6d5d5fe4cce2f);
        let init0H = V2x64U::new(0x243f6a8885a308d3, 0x13198a2e03707344);
        let init1L = V2x64U::new(0xc0acf169b5f18a8c, 0x3bd39e10cb0ef593);
        let init1H = V2x64U::new(0x452821e638d01377, 0xbe5466cf34e90c6c);
        let keyL = V2x64U::from(_mm_loadu_si128(self.key.0.as_ptr() as *const __m128i));
        let keyH = V2x64U::from(_mm_loadu_si128(
            self.key.0.as_ptr().offset(2) as *const __m128i
        ));
        self.v0L = keyL ^ init0L;
        self.v0H = keyH ^ init0H;
        self.v1L = keyL.rotate_by_32() ^ init1L;
        self.v1H = keyH.rotate_by_32() ^ init1H;
        self.mul0L = init0L;
        self.mul0H = init0H;
        self.mul1L = init1L;
        self.mul1H = init1H;
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn zipper_merge(v: &V2x64U) -> V2x64U {
        v.shuffle(&V2x64U::new(0x070806090D0A040B, 0x000F010E05020C03))
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn update(&mut self, packetH: V2x64U, packetL: V2x64U) {
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
        self.update(low, high);
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn finalize64(&mut self) -> u64 {
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
        _mm_storel_epi64((&mut result as *mut u64) as *mut __m128i, hash.0);
        result
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn finalize128(&mut self) -> u128 {
        if !self.buffer.is_empty() {
            self.update_remainder();
        }

        for _i in 0..6 {
            self.permute_and_update();
        }

        let sum0 = self.v0L + self.mul0L;
        let sum1 = self.v1H + self.mul1H;
        let hash = sum0 + sum1;
        let mut result: u128 = 0;
        _mm_storeu_si128((&mut result as *mut u128) as *mut __m128i, hash.0);
        result
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn finalize256(&mut self) -> (u128, u128) {
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
        let mut resultL: u128 = 0;
        let mut resultH: u128 = 0;
        _mm_storeu_si128((&mut resultL as *mut u128) as *mut __m128i, hashL.0);
        _mm_storeu_si128((&mut resultH as *mut u128) as *mut __m128i, hashH.0);
        (resultL, resultH)
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn modular_reduction(x: &V2x64U, init: &V2x64U) -> V2x64U {
        let zero = V2x64U::default();
        let sign_bit128 = V2x64U::from(_mm_insert_epi32(zero.0, 0x80000000u32 as i32, 3));
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
    unsafe fn load_multiple_of_four(bytes: &[u8], size: u64) -> V2x64U {
        let mut data = &bytes[..];
        let mut mask4 = V2x64U::from(_mm_cvtsi64_si128(0xFFFFFFFF));
        let mut ret = if size & 8 != 0 {
            mask4 = V2x64U::from(_mm_slli_si128(mask4.0, 8));
            data = &bytes[8..];
            V2x64U::from(_mm_loadl_epi64(bytes.as_ptr() as *const __m128i))
        } else {
            V2x64U::new(0, 0)
        };

        if size & 4 != 0 {
            let word2 = _mm_cvtsi32_si128(LE::read_i32(data));
            let broadcast = V2x64U::from(_mm_shuffle_epi32(word2, 0));
            ret |= broadcast & mask4;
        }

        ret
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn remainder(bytes: &[u8]) -> (V2x64U, V2x64U) {
        let size_mod32 = bytes.len();
        let size_mod4 = size_mod32 & 3;
        if size_mod32 & 16 != 0 {
            let packetL = V2x64U::from(_mm_loadu_si128(bytes.as_ptr() as *const __m128i));
            let packett = SseHash::load_multiple_of_four(&bytes[16..], size_mod32 as u64);
            let remainder = &bytes[(size_mod32 & !3) + size_mod4 - 4..];
            let last4 = LE::read_i32(remainder);
            let packetH = V2x64U::from(_mm_insert_epi32(packett.0, last4, 3));
            (packetH, packetL)
        } else {
            let remainder = &bytes[size_mod32 & !3..];
            let packetL = SseHash::load_multiple_of_four(bytes, size_mod32 as u64);
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
        let (packetH, packetL) = SseHash::remainder(self.buffer.as_slice());
        self.update(packetH, packetL);
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
    unsafe fn to_lanes(packet: &[u8]) -> (V2x64U, V2x64U) {
        let packetL = V2x64U::from(_mm_loadu_si128(packet.as_ptr() as *const __m128i));
        let packetH = V2x64U::from(_mm_loadu_si128(packet.as_ptr().offset(16) as *const __m128i));

        (packetH, packetL)
    }

    #[target_feature(enable = "sse4.1")]
    unsafe fn append(&mut self, data: &[u8]) {
        match self.buffer.fill(data) {
            Filled::Consumed => {}
            Filled::Full(new_data) => {
                let (packetH, packetL) = SseHash::to_lanes(self.buffer.as_slice());
                self.update(packetH, packetL);

                let mut rest = &new_data[..];
                while rest.len() >= PACKET_SIZE {
                    let (packetH, packetL) = SseHash::to_lanes(&rest);
                    self.update(packetH, packetL);
                    rest = &rest[PACKET_SIZE..];
                }

                self.buffer.set_to(rest);
            }
        }
    }
}
