use byteorder::{ByteOrder, LE};
use std::fmt;
use std::ops::Index;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, ShlAssign,
    ShrAssign, SubAssign,
};

use v2x64u::V2x64U;
use v4x64u::V4x64U;
use internal::unordered_load3;
use key::Key;

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[derive(Default)]
pub struct AvxHash {
    key: Key,
    v0: V4x64U,
    v1: V4x64U,
    mul0: V4x64U,
    mul1: V4x64U,
}

impl AvxHash {
    pub fn new(key: &Key) -> Self {
        AvxHash {
            key: key.clone(),
            ..Default::default()
        }
    }

    pub fn hash64(data: &[u8], key: &Key) -> u64 {
        let mut hash = AvxHash::new(key);
        hash.process_all(data);
        hash.finalize64()
    }

    pub fn hash128(data: &[u8], key: &Key) -> u128 {
        let mut hash = AvxHash::new(key);
        hash.process_all(data);
        hash.finalize128()
    }

    pub fn hash256(data: &[u8], key: &Key) -> (u128, u128) {
        let mut hash = AvxHash::new(key);
        hash.process_all(data);
        hash.finalize256()
    }

    fn finalize64(&mut self) -> u64 {
        for i in 0..4 {
            let permuted = AvxHash::permute(&self.v0);
            self.update(permuted);
        }

        let sum0 = V2x64U::from(unsafe { _mm256_castsi256_si128((self.v0 + self.mul0).0) });
        let sum1 = V2x64U::from(unsafe { _mm256_castsi256_si128((self.v1 + self.mul1).0) });
        let hash = sum0 + sum1;
        let mut result: u64 = 0;
        // Each lane is sufficiently mixed, so just truncate to 64 bits.
        unsafe { _mm_storel_epi64((&mut result as *mut u64 as *mut __m128i), hash.0) };
        result
    }

    fn finalize128(&mut self) -> u128 {
        for i in 0..6 {
            let permuted = AvxHash::permute(&self.v0);
            self.update(permuted);
        }

        let sum0 = V2x64U::from(unsafe { _mm256_castsi256_si128((self.v0 + self.mul0).0) });
        let sum1 = V2x64U::from(unsafe { _mm256_extracti128_si256((self.v1 + self.mul1).0, 1) });
        let hash = sum0 + sum1;
        let mut result: u128 = 0;
        unsafe { _mm_storeu_si128((&mut result as *mut u128 as *mut __m128i), hash.0) };
        result
    }

    fn finalize256(&mut self) -> (u128, u128) {
        for i in 0..10 {
            let permuted = AvxHash::permute(&self.v0);
            self.update(permuted);
        }

        let sum0 = self.v0 + self.mul0;
        let sum1 = self.v1 + self.mul1;
        let hash = AvxHash::modular_reduction(&sum1, &sum0);
        let mut result: [u128; 2] = [0, 0];
        unsafe { _mm256_storeu_si256((&mut result[0] as *mut u128 as *mut __m256i), hash.0) };
        (result[0], result[1])
    }

    fn reset(&mut self) {
        let init0 = V4x64U::new(
            0x243f6a8885a308d3,
            0x13198a2e03707344,
            0xa4093822299f31d0,
            0xdbe6d5d5fe4cce2f,
        );
        let init1 = V4x64U::new(
            0x452821e638d01377,
            0xbe5466cf34e90c6c,
            0xc0acf169b5f18a8c,
            0x3bd39e10cb0ef593,
        );

        let key = V4x64U::from(unsafe {
            _mm256_load_si256((&self.key[0] as *const u64) as *const __m256i)
        });

        self.v0 = key ^ init0;
        self.v1 = key.rotate_by_32() ^ init1;
        self.mul0 = init0;
        self.mul1 = init1;
    }

    fn process_all(&mut self, data: &[u8]) {
        self.reset();
        let mut slice = &data[..];
        while slice.len() >= 32 {
            let packet = V4x64U::from(unsafe {
                _mm256_load_si256((&slice[0] as *const u8) as *const __m256i)
            });
            self.update(packet);
            slice = &slice[32..];
        }

        if (!slice.is_empty()) {
            self.update_remainder(&slice);
        }
    }

    fn update_remainder(&mut self, bytes: &[u8]) {
        let size256 = unsafe { _mm256_broadcastd_epi32(_mm_cvtsi64_si128(bytes.len() as i64)) };
        self.v0 += V4x64U::from(size256);
        let shifted_left = V4x64U::from(unsafe { _mm256_sllv_epi32(self.v1.0, size256) });
        let tip = unsafe { _mm256_broadcastd_epi32(_mm_cvtsi32_si128(32)) };
        let shifted_right =
            V4x64U::from(unsafe { _mm256_srlv_epi32(self.v1.0, _mm256_sub_epi32(tip, size256)) });
        self.v1 = shifted_left | shifted_right;

        let size_mod32 = bytes.len();
        let size_mod4 = size_mod32 & 3;
        let size = unsafe { _mm256_castsi256_si128(size256) };
        if size_mod32 & 16 != 0 {
            let packetL = unsafe { _mm_load_si128((&bytes[0] as *const u8) as *const __m128i) };
            let int_mask = unsafe { _mm_cmpgt_epi32(size, _mm_set_epi32(31, 27, 23, 19)) };
            let int_lanes = unsafe {
                _mm_maskload_epi32((&bytes[0] as *const u8).offset(16) as *const i32, int_mask)
            };
            let remainder = &bytes[(size_mod32 & !3) + size_mod4 - 4..];
            let last4 = LE::read_i32(remainder);
            let packetH = unsafe { _mm_insert_epi32(int_lanes, last4, 3) };
            let packetL256 = unsafe { _mm256_castsi128_si256(packetL) };
            let packet = unsafe { _mm256_inserti128_si256(packetL256, packetH, 1) };
            self.update(V4x64U::from(packet));
        } else {
            let int_mask = unsafe { _mm_cmpgt_epi32(size, _mm_set_epi32(15, 11, 7, 3)) };
            let packetL =
                unsafe { _mm_maskload_epi32(&bytes[0] as *const u8 as *const i32, int_mask) };
            let remainder = &bytes[size_mod32 & !3..];
            let last3 = unordered_load3(remainder);
            let packetH = unsafe { _mm_cvtsi64_si128(last3 as i64) };
            let packetL256 = unsafe { _mm256_castsi128_si256(packetL) };
            let packet = unsafe { _mm256_inserti128_si256(packetL256, packetH, 1) };
            self.update(V4x64U::from(packet));
        }
    }

    fn zipper_merge(v: &V4x64U) -> V4x64U {
        let hi = 0x070806090D0A040B;
        let lo = 0x000F010E05020C03;
        v.shuffle(&V4x64U::new(hi, lo, hi, lo))
    }

    fn update(&mut self, packet: V4x64U) {
        self.v1 += packet;
        self.v1 += self.mul0;
        self.mul0 ^= self
            .v1
            .mul_low32(&V4x64U::from(unsafe { _mm256_srli_epi64(self.v0.0, 32) }));
        self.v0 += self.mul1;
        self.mul1 ^= self
            .v0
            .mul_low32(&V4x64U::from(unsafe { _mm256_srli_epi64(self.v1.0, 32) }));
        self.v0 += AvxHash::zipper_merge(&self.v1);
        self.v1 += AvxHash::zipper_merge(&self.v0);
    }

    fn permute(v: &V4x64U) -> V4x64U {
        let indices = V4x64U::new(
            0x0000000200000003,
            0x0000000000000001,
            0x0000000600000007,
            0x0000000400000005,
        );
        V4x64U::from(unsafe { _mm256_permutevar8x32_epi32(v.0, indices.0) })
    }

    fn modular_reduction(x: &V4x64U, init: &V4x64U) -> V4x64U {
        let zero = *x ^ *x;
        let top_bits2 = V4x64U::from(unsafe { _mm256_srli_epi64(x.0, 62) });
        let ones = V4x64U::from(unsafe { _mm256_cmpeq_epi64(x.0, x.0) });
        let shifted1_unmasked = *x + *x;
        let top_bits1 = V4x64U::from(unsafe { _mm256_srli_epi64(x.0, 63) });
        let upper_8bytes = V4x64U::from(unsafe { _mm256_slli_si256(ones.0, 8) });
        let shifted2 = shifted1_unmasked + shifted1_unmasked;
        let upper_bit_of_128 = V4x64U::from(unsafe { _mm256_slli_epi64(upper_8bytes.0, 63) });
        let new_low_bits2 = V4x64U::from(unsafe { _mm256_unpacklo_epi64(zero.0, top_bits2.0) });
        let shifted1 = shifted1_unmasked.and_not(&upper_bit_of_128);
        let new_low_bits1 = V4x64U::from(unsafe { _mm256_unpacklo_epi64(zero.0, top_bits1.0) });

        *init ^ shifted2 ^ new_low_bits2 ^ shifted1 ^ new_low_bits1
    }
}
