use crate::internal::{HashPacket, PACKET_SIZE};
use crate::key::Key;
use crate::traits::HighwayHash;

/// Portable HighwayHash implementation. Will run on any platform Rust will run on.
#[derive(Debug, Default, Clone)]
pub struct PortableHash {
    v0: [u64; 4],
    v1: [u64; 4],
    mul0: [u64; 4],
    mul1: [u64; 4],
    buffer: HashPacket,
}

impl HighwayHash for PortableHash {
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

impl PortableHash {
    /// Create a new `PortableHash` from a `Key`
    #[must_use]
    pub fn new(key: Key) -> Self {
        let mul0 = [
            0xdbe6_d5d5_fe4c_ce2f,
            0xa409_3822_299f_31d0,
            0x1319_8a2e_0370_7344,
            0x243f_6a88_85a3_08d3,
        ];
        let mul1 = [
            0x3bd3_9e10_cb0e_f593,
            0xc0ac_f169_b5f1_8a8c,
            0xbe54_66cf_34e9_0c6c,
            0x4528_21e6_38d0_1377,
        ];

        PortableHash {
            v0: [
                mul0[0] ^ key[0],
                mul0[1] ^ key[1],
                mul0[2] ^ key[2],
                mul0[3] ^ key[3],
            ],
            v1: [
                mul1[0] ^ ((key[0] >> 32) | (key[0] << 32)),
                mul1[1] ^ ((key[1] >> 32) | (key[1] << 32)),
                mul1[2] ^ ((key[2] >> 32) | (key[2] << 32)),
                mul1[3] ^ ((key[3] >> 32) | (key[3] << 32)),
            ],
            mul0,
            mul1,
            buffer: HashPacket::default(),
        }
    }

    fn finalize64(&mut self) -> u64 {
        if !self.buffer.is_empty() {
            self.update_remainder();
        }

        for _i in 0..4 {
            self.permute_and_update();
        }

        self.v0[0]
            .wrapping_add(self.v1[0])
            .wrapping_add(self.mul0[0])
            .wrapping_add(self.mul1[0])
    }

    fn finalize128(&mut self) -> [u64; 2] {
        if !self.buffer.is_empty() {
            self.update_remainder();
        }

        for _i in 0..6 {
            self.permute_and_update();
        }

        let low = self.v0[0]
            .wrapping_add(self.mul0[0])
            .wrapping_add(self.v1[2])
            .wrapping_add(self.mul1[2]);

        let high = self.v0[1]
            .wrapping_add(self.mul0[1])
            .wrapping_add(self.v1[3])
            .wrapping_add(self.mul1[3]);

        [low, high]
    }

    fn finalize256(&mut self) -> [u64; 4] {
        if !self.buffer.is_empty() {
            self.update_remainder();
        }

        for _i in 0..10 {
            self.permute_and_update();
        }

        let (lowest, low) = PortableHash::module_reduction(
            self.v1[1].wrapping_add(self.mul1[1]),
            self.v1[0].wrapping_add(self.mul1[0]),
            self.v0[1].wrapping_add(self.mul0[1]),
            self.v0[0].wrapping_add(self.mul0[0]),
        );
        let (high, highest) = PortableHash::module_reduction(
            self.v1[3].wrapping_add(self.mul1[3]),
            self.v1[2].wrapping_add(self.mul1[2]),
            self.v0[3].wrapping_add(self.mul0[3]),
            self.v0[2].wrapping_add(self.mul0[2]),
        );

        [lowest, low, high, highest]
    }

    fn module_reduction(a3_unmasked: u64, a2: u64, a1: u64, a0: u64) -> (u64, u64) {
        let a3 = a3_unmasked & 0x3FFF_FFFF_FFFF_FFFF;
        let high = a1 ^ ((a3 << 1) | (a2 >> 63)) ^ ((a3 << 2) | (a2 >> 62));
        let low = a0 ^ (a2 << 1) ^ (a2 << 2);
        (low, high)
    }

    fn permute(v: &[u64; 4]) -> [u64; 4] {
        [
            (v[2] >> 32) | (v[2] << 32),
            (v[3] >> 32) | (v[3] << 32),
            (v[0] >> 32) | (v[0] << 32),
            (v[1] >> 32) | (v[1] << 32),
        ]
    }

    fn permute_and_update(&mut self) {
        let permuted: [u64; 4] = PortableHash::permute(&self.v0);
        self.update(permuted);
    }

    fn update(&mut self, lanes: [u64; 4]) {
        for (i, lane) in lanes.iter().enumerate() {
            self.v1[i] = self.v1[i].wrapping_add(*lane);
        }

        for i in 0..4 {
            self.v1[i] = self.v1[i].wrapping_add(self.mul0[i]);
        }

        for i in 0..4 {
            self.mul0[i] ^= (self.v1[i] & 0xffff_ffff).wrapping_mul(self.v0[i] >> 32);
        }

        for i in 0..4 {
            self.v0[i] = self.v0[i].wrapping_add(self.mul1[i]);
        }

        for i in 0..4 {
            self.mul1[i] ^= (self.v0[i] & 0xffff_ffff).wrapping_mul(self.v1[i] >> 32);
        }

        PortableHash::zipper_merge_and_add(self.v1[1], self.v1[0], &mut self.v0, 1, 0);
        PortableHash::zipper_merge_and_add(self.v1[3], self.v1[2], &mut self.v0, 3, 2);
        PortableHash::zipper_merge_and_add(self.v0[1], self.v0[0], &mut self.v1, 1, 0);
        PortableHash::zipper_merge_and_add(self.v0[3], self.v0[2], &mut self.v1, 3, 2);
    }

    fn zipper_merge_and_add(v1: u64, v0: u64, lane: &mut [u64; 4], add1: usize, add0: usize) {
        lane[add0] = lane[add0].wrapping_add(
            (((v0 & 0xff00_0000) | (v1 & 0x00ff_0000_0000)) >> 24)
                | (((v0 & 0xff00_0000_0000) | (v1 & 0x00ff_0000_0000_0000)) >> 16)
                | (v0 & 0x00ff_0000)
                | ((v0 & 0xff00) << 32)
                | ((v1 & 0xff00_0000_0000_0000) >> 8)
                | (v0 << 56),
        );
        lane[add1] = lane[add1].wrapping_add(
            (((v1 & 0xff00_0000) | (v0 & 0x00ff_0000_0000)) >> 24)
                | (v1 & 0x00ff_0000)
                | ((v1 & 0xff00_0000_0000) >> 16)
                | ((v1 & 0xff00) << 24)
                | ((v0 & 0x00ff_0000_0000_0000) >> 8)
                | ((v1 & 0xff) << 48)
                | (v0 & 0xff00_0000_0000_0000),
        );
    }

    fn data_to_lanes(d: &[u8]) -> [u64; 4] {
        let mut result = [0u64; 4];
        for (x, dest) in d.chunks_exact(8).zip(result.iter_mut()) {
            *dest = u64::from_le_bytes([x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7]])
        }
        result
    }

    fn rotate_32_by(count: u64, lanes: &mut [u64; 4]) {
        for lane in lanes.iter_mut() {
            let half0: u32 = *lane as u32;
            let half1: u32 = (*lane >> 32) as u32;
            *lane = u64::from((half0 << count) | (half0 >> (32 - count)));
            *lane |= u64::from((half1 << count) | (half1 >> (32 - count))) << 32;
        }
    }

    fn update_lanes(&mut self, size: u64) {
        for i in 0..4 {
            self.v0[i] = self.v0[i].wrapping_add((size << 32) + size);
        }

        PortableHash::rotate_32_by(size, &mut self.v1);
    }

    fn remainder(bytes: &[u8]) -> [u8; 32] {
        let mut packet: [u8; 32] = [0u8; 32];
        if bytes.len() > packet.len() {
            debug_assert!(false, "remainder bytes must be less than 32");
            return packet;
        }

        let size_mod4 = bytes.len() & 3;
        let remainder_jump = bytes.len() & !3;
        let remainder = &bytes[remainder_jump..];
        let size = bytes.len() as u64;

        packet[..remainder_jump].clone_from_slice(&bytes[..remainder_jump]);
        if size & 16 != 0 {
            let muxed = packet[28..]
                .iter_mut()
                .zip(&bytes[remainder_jump + size_mod4 - 4..]);

            for (p, b) in muxed {
                *p = *b;
            }
        } else if size_mod4 != 0 {
            packet[16] = remainder[0];
            packet[16 + 1] = remainder[size_mod4 >> 1];
            packet[16 + 2] = remainder[size_mod4 - 1];
        }

        packet
    }

    fn update_remainder(&mut self) {
        let size = self.buffer.len() as u64;
        self.update_lanes(size);
        let packet = PortableHash::remainder(self.buffer.as_slice());
        self.update(PortableHash::data_to_lanes(&packet));
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

impl_write!(PortableHash);
impl_hasher!(PortableHash);
