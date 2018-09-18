use byteorder::{ByteOrder, LE};
use internal::unordered_load3;
use key::Key;
use traits::HighwayHash;
use v2x64u::V2x64U;
use v4x64u::V4x64U;
use internal::{PACKET_SIZE, HashPacket, Filled};
use std::arch::x86_64::*;

#[derive(Default)]
pub struct AvxHash {
    key: Key,
    buffer: HashPacket,
    v0: V4x64U,
    v1: V4x64U,
    mul0: V4x64U,
    mul1: V4x64U,
}

impl HighwayHash for AvxHash {
    fn hash64(mut self, data: &[u8]) -> u64 {
        self.append(data);
        self.finalize64()
    }

    fn hash128(mut self, data: &[u8]) -> u128 {
        self.append(data);
        self.finalize128()
    }

    fn hash256(mut self, data: &[u8]) -> (u128, u128) {
        self.append(data);
        self.finalize256()
    }
}

impl AvxHash {
    pub unsafe fn force_new(key: &Key) -> Self {
        let mut h = AvxHash {
            key: key.clone(),
            ..Default::default()
        };
        h.reset();
        h
    }

    pub fn new(key: &Key) -> Option<Self> {
        if is_x86_feature_detected!("avx2") {
            Some(unsafe { Self::force_new(key) })
        } else {
            None
        }
    }

    fn finalize64(&mut self) -> u64 {
        if !self.buffer.is_empty() {
            self.update_remainder();
        }

        for _i in 0..4 {
            let permuted = AvxHash::permute(&self.v0);
            self.update(permuted);
        }

        let sum0 = V2x64U::from(unsafe { _mm256_castsi256_si128((self.v0 + self.mul0).0) });
        let sum1 = V2x64U::from(unsafe { _mm256_castsi256_si128((self.v1 + self.mul1).0) });
        let hash = sum0 + sum1;
        let mut result: u64 = 0;
        // Each lane is sufficiently mixed, so just truncate to 64 bits.
        unsafe { _mm_storel_epi64(&mut result as *mut u64 as *mut __m128i, hash.0) };
        result
    }

    fn finalize128(&mut self) -> u128 {
        if !self.buffer.is_empty() {
            self.update_remainder();
        }

        for _i in 0..6 {
            let permuted = AvxHash::permute(&self.v0);
            self.update(permuted);
        }

        let sum0 = V2x64U::from(unsafe { _mm256_castsi256_si128((self.v0 + self.mul0).0) });
        let sum1 = V2x64U::from(unsafe { _mm256_extracti128_si256((self.v1 + self.mul1).0, 1) });
        let hash = sum0 + sum1;
        let mut result: u128 = 0;
        unsafe { _mm_storeu_si128(&mut result as *mut u128 as *mut __m128i, hash.0) };
        result
    }

    fn finalize256(&mut self) -> (u128, u128) {
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
        let mut result: [u128; 2] = [0, 0];
        unsafe { _mm256_storeu_si256(result.as_mut_ptr() as *mut __m256i, hash.0) };
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
            _mm256_load_si256(self.key.0.as_ptr() as *const __m256i)
        });

        self.v0 = key ^ init0;
        self.v1 = key.rotate_by_32() ^ init1;
        self.mul0 = init0;
        self.mul1 = init1;
    }

    #[inline]
    fn to_lanes(packet: &[u8]) -> V4x64U {
        V4x64U::from(unsafe {
                _mm256_load_si256(packet.as_ptr() as *const __m256i)
            })
    }

    fn remainder(bytes: &[u8]) -> V4x64U {
        let size_mod32 = bytes.len();
        let size256 = unsafe { _mm256_broadcastd_epi32(_mm_cvtsi64_si128(size_mod32 as i64)) };
        let size_mod4 = size_mod32 & 3;
        let size = unsafe { _mm256_castsi256_si128(size256) };
        if size_mod32 & 16 != 0 {
            let packetL = unsafe { _mm_load_si128(bytes.as_ptr() as *const __m128i) };
            let int_mask = unsafe { _mm_cmpgt_epi32(size, _mm_set_epi32(31, 27, 23, 19)) };
            let int_lanes = unsafe {
                _mm_maskload_epi32(bytes.as_ptr().offset(16) as *const i32, int_mask)
            };
            let remainder = &bytes[(size_mod32 & !3) + size_mod4 - 4..];
            let last4 = LE::read_i32(remainder);
            let packetH = unsafe { _mm_insert_epi32(int_lanes, last4, 3) };
            let packetL256 = unsafe { _mm256_castsi128_si256(packetL) };
            let packet = unsafe { _mm256_inserti128_si256(packetL256, packetH, 1) };
            V4x64U::from(packet)
        } else {
            let int_mask = unsafe { _mm_cmpgt_epi32(size, _mm_set_epi32(15, 11, 7, 3)) };
            let packetL =
                unsafe { _mm_maskload_epi32(bytes.as_ptr() as *const i32, int_mask) };
            let remainder = &bytes[size_mod32 & !3..];
            let last3 = unordered_load3(remainder);
            let packetH = unsafe { _mm_cvtsi64_si128(last3 as i64) };
            let packetL256 = unsafe { _mm256_castsi128_si256(packetL) };
            let packet = unsafe { _mm256_inserti128_si256(packetL256, packetH, 1) };
            V4x64U::from(packet)
        }
    }

    fn update_remainder(&mut self) {
        let size = self.buffer.len();
        let size256 = unsafe { _mm256_broadcastd_epi32(_mm_cvtsi64_si128(size as i64)) };
        self.v0 += V4x64U::from(size256);
        let shifted_left = V4x64U::from(unsafe { _mm256_sllv_epi32(self.v1.0, size256) });
        let tip = unsafe { _mm256_broadcastd_epi32(_mm_cvtsi32_si128(32)) };
        let shifted_right =
            V4x64U::from(unsafe { _mm256_srlv_epi32(self.v1.0, _mm256_sub_epi32(tip, size256)) });
        self.v1 = shifted_left | shifted_right;

        let packet = AvxHash::remainder(self.buffer.as_slice());
        self.update(packet);
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

    pub fn append(&mut self, data: &[u8]) -> &mut Self {
        match self.buffer.fill(data) {
            Filled::Consumed => self,
            Filled::Full(new_data) => {
                let packet = AvxHash::to_lanes(self.buffer.as_slice());
				self.update(packet);

				let mut rest = &new_data[..];
                while rest.len() >= PACKET_SIZE {
                    let packet = AvxHash::to_lanes(&rest);
                    self.update(packet);
                    rest = &rest[PACKET_SIZE..];
                }

                self.buffer.set_to(rest);
                self
            }
        }
    }
}
