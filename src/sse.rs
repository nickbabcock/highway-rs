use byteorder::{ByteOrder, LE};
use internal::unordered_load3;
use key::Key;
use std::arch::x86_64::*;
use traits::HighwayHash;
use v2x64u::V2x64U;

#[derive(Default)]
pub struct SseHash {
    key: Key,
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
        self.process_all(data);
        self.finalize64()
    }

    fn hash128(mut self, data: &[u8]) -> u128 {
        self.process_all(data);
        self.finalize128()
    }

    fn hash256(mut self, data: &[u8]) -> (u128, u128) {
        self.process_all(data);
        self.finalize256()
    }
}

impl SseHash {
    pub unsafe fn force_new(key: &Key) -> Self {
        SseHash {
            key: key.clone(),
            ..Default::default()
        }
    }

    pub fn new(key: &Key) -> Option<Self> {
        if is_x86_feature_detected!("sse4.1") {
            Some(unsafe { Self::force_new(key) })
        } else {
            None
        }
    }

    fn reset(&mut self) {
        let init0L = V2x64U::new(0xa4093822299f31d0, 0xdbe6d5d5fe4cce2f);
        let init0H = V2x64U::new(0x243f6a8885a308d3, 0x13198a2e03707344);
        let init1L = V2x64U::new(0xc0acf169b5f18a8c, 0x3bd39e10cb0ef593);
        let init1H = V2x64U::new(0x452821e638d01377, 0xbe5466cf34e90c6c);
        let keyL = V2x64U::from(unsafe {
            _mm_loadu_si128((&self.key[0] as *const u64) as *const __m128i)
        });
        let keyH = V2x64U::from(unsafe {
            _mm_loadu_si128((&self.key[2] as *const u64) as *const __m128i)
        });
        self.v0L = keyL ^ init0L;
        self.v0H = keyH ^ init0H;
        self.v1L = keyL.rotate_by_32() ^ init1L;
        self.v1H = keyH.rotate_by_32() ^ init1H;
        self.mul0L = init0L;
        self.mul0H = init0H;
        self.mul1L = init1L;
        self.mul1H = init1H;
    }

    fn zipper_merge(v: &V2x64U) -> V2x64U {
        v.shuffle(&V2x64U::new(0x070806090D0A040B, 0x000F010E05020C03))
    }

    fn update(&mut self, packetH: V2x64U, packetL: V2x64U) {
        unsafe {
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
    }

    fn permute_and_update(&mut self) {
        let low = self.v0L.rotate_by_32();
        let high = self.v0H.rotate_by_32();
        self.update(low, high);
    }

    fn finalize64(&mut self) -> u64 {
        for _i in 0..4 {
            self.permute_and_update();
        }

        let sum0 = self.v0L + self.mul0L;
        let sum1 = self.v1L + self.mul1L;
        let hash = sum0 + sum1;
        let mut result: u64 = 0;
        unsafe { _mm_storel_epi64((&mut result as *mut u64) as *mut __m128i, hash.0) };
        result
    }

    fn finalize128(&mut self) -> u128 {
        for _i in 0..6 {
            self.permute_and_update();
        }

        let sum0 = self.v0L + self.mul0L;
        let sum1 = self.v1H + self.mul1H;
        let hash = sum0 + sum1;
        let mut result: u128 = 0;
        unsafe { _mm_storeu_si128((&mut result as *mut u128) as *mut __m128i, hash.0) };
        result
    }

    fn finalize256(&mut self) -> (u128, u128) {
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
        unsafe { _mm_storeu_si128((&mut resultL as *mut u128) as *mut __m128i, hashL.0) };
        unsafe { _mm_storeu_si128((&mut resultH as *mut u128) as *mut __m128i, hashH.0) };
        (resultL, resultH)
    }

    fn modular_reduction(x: &V2x64U, init: &V2x64U) -> V2x64U {
        let zero = V2x64U::default();
        let sign_bit128 =
            V2x64U::from(unsafe { _mm_insert_epi32(zero.0, 0x80000000u32 as i32, 3) });
        let top_bits2 = V2x64U::from(unsafe { _mm_srli_epi64(x.0, 62) });
        let shifted1_unmasked = *x + *x;
        let top_bits1 = V2x64U::from(unsafe { _mm_srli_epi64(x.0, 63) });
        let shifted2 = shifted1_unmasked + shifted1_unmasked;
        let new_low_bits2 = V2x64U::from(unsafe { _mm_slli_si128(top_bits2.0, 8) });
        let shifted1 = shifted1_unmasked.and_not(&sign_bit128);
        let new_low_bits1 = V2x64U::from(unsafe { _mm_slli_si128(top_bits1.0, 8) });
        *init ^ shifted2 ^ new_low_bits2 ^ shifted1 ^ new_low_bits1
    }

    fn process_all(&mut self, data: &[u8]) {
        self.reset();
        let mut slice = &data[..];
        while slice.len() >= 32 {
            let packetL = V2x64U::from(unsafe {
                _mm_loadu_si128((&slice[0] as *const u8) as *const __m128i)
            });
            let packetH = V2x64U::from(unsafe {
                _mm_loadu_si128((&slice[16] as *const u8) as *const __m128i)
            });
            self.update(packetH, packetL);
            slice = &slice[32..];
        }

        if !slice.is_empty() {
            self.update_remainder(&slice);
        }
    }

    fn load_multiple_of_four(bytes: &[u8], size: u64) -> V2x64U {
        let mut data = &bytes[..];
        let mut mask4 = V2x64U::from(unsafe { _mm_cvtsi64_si128(0xFFFFFFFF) });
        let mut ret = if size & 8 != 0 {
            mask4 = V2x64U::from(unsafe { _mm_slli_si128(mask4.0, 8) });
            data = &bytes[8..];
            V2x64U::from(unsafe { _mm_loadl_epi64((&bytes[0] as *const u8) as *const __m128i) })
        } else {
            V2x64U::new(0, 0)
        };

        if size & 4 != 0 {
            let word2 = unsafe { _mm_cvtsi32_si128(LE::read_i32(data)) };
            let broadcast = V2x64U::from(unsafe { _mm_shuffle_epi32(word2, 0) });
            ret |= broadcast & mask4;
        }

        ret
    }

    fn update_remainder(&mut self, bytes: &[u8]) {
        let vsize_mod32 = unsafe { _mm_set1_epi32(bytes.len() as i32) };
        self.v0L += V2x64U::from(vsize_mod32);
        self.v0H += V2x64U::from(vsize_mod32);
        self.rotate_32_by(bytes.len() as i64);
        let size_mod32 = bytes.len();
        let size_mod4 = size_mod32 & 3;

        if size_mod32 & 16 != 0 {
            let packetL = V2x64U::from(unsafe {
                _mm_loadu_si128((&bytes[0] as *const u8) as *const __m128i)
            });
            let packett = SseHash::load_multiple_of_four(&bytes[16..], size_mod32 as u64);
            let remainder = &bytes[(size_mod32 & !3) + size_mod4 - 4..];
            let last4 = LE::read_i32(remainder);
            let packetH = V2x64U::from(unsafe { _mm_insert_epi32(packett.0, last4, 3) });
            self.update(packetH, packetL);
        } else {
            let remainder = &bytes[size_mod32 & !3..];
            let packetL = SseHash::load_multiple_of_four(bytes, size_mod32 as u64);
            let last4 = unordered_load3(remainder);
            let packetH = V2x64U::from(unsafe { _mm_cvtsi64_si128(last4 as i64) });
            self.update(packetH, packetL);
        }
    }

    fn rotate_32_by(&mut self, count: i64) {
        let vL = &mut self.v1L;
        let vH = &mut self.v1H;
        unsafe {
            let count_left = _mm_cvtsi64_si128(count);
            let count_right = _mm_cvtsi64_si128(32 - count);
            let shifted_leftL = V2x64U::from(_mm_sll_epi32(vL.0, count_left));
            let shifted_leftH = V2x64U::from(_mm_sll_epi32(vH.0, count_left));
            let shifted_rightL = V2x64U::from(_mm_srl_epi32(vL.0, count_right));
            let shifted_rightH = V2x64U::from(_mm_srl_epi32(vH.0, count_right));
            *vL = shifted_leftL | shifted_rightL;
            *vH = shifted_leftH | shifted_rightH;
        }
    }
}
