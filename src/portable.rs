use byteorder::{ByteOrder, LE};
use internal::{Filled, HashPacket, PACKET_SIZE};
use key::Key;
use traits::HighwayHash;

/// Portable HighwayHash implementation. Will run on any platform Rust will run on.
#[derive(Debug, Default)]
pub struct PortableHash {
    key: Key,
    buffer: HashPacket,
    v0: [u64; 4],
    v1: [u64; 4],
    mul0: [u64; 4],
    mul1: [u64; 4],
}

impl HighwayHash for PortableHash {
    fn hash64(mut self, data: &[u8]) -> u64 {
        self.append(data);
        self.finalize64()
    }

    fn hash128(mut self, data: &[u8]) -> [u64; 2] {
        self.append(data);
        self.finalize128()
    }

    fn hash256(mut self, data: &[u8]) -> [u64; 4] {
        self.append(data);
        self.finalize256()
    }

    fn append(&mut self, data: &[u8]) {
        self.append(data);
    }

    fn finalize64(mut self) -> u64 {
        Self::finalize64(&mut self)
    }

    fn finalize128(mut self) -> [u64; 2] {
        Self::finalize128(&mut self)
    }

    fn finalize256(mut self) -> [u64; 4] {
        Self::finalize256(&mut self)
    }
}

impl PortableHash {
    /// Create a new `PortableHash` from a `Key`
    pub fn new(key: &Key) -> Self {
        let mut h = PortableHash {
            key: key.clone(),
            ..Default::default()
        };
        h.reset();
        h
    }

    fn reset(&mut self) {
        self.mul0[0] = 0xdbe6d5d5fe4cce2f;
        self.mul0[1] = 0xa4093822299f31d0;
        self.mul0[2] = 0x13198a2e03707344;
        self.mul0[3] = 0x243f6a8885a308d3;
        self.mul1[0] = 0x3bd39e10cb0ef593;
        self.mul1[1] = 0xc0acf169b5f18a8c;
        self.mul1[2] = 0xbe5466cf34e90c6c;
        self.mul1[3] = 0x452821e638d01377;
        self.v0[0] = self.mul0[0] ^ self.key[0];
        self.v0[1] = self.mul0[1] ^ self.key[1];
        self.v0[2] = self.mul0[2] ^ self.key[2];
        self.v0[3] = self.mul0[3] ^ self.key[3];
        self.v1[0] = self.mul1[0] ^ ((self.key[0] >> 32) | (self.key[0] << 32));
        self.v1[1] = self.mul1[1] ^ ((self.key[1] >> 32) | (self.key[1] << 32));
        self.v1[2] = self.mul1[2] ^ ((self.key[2] >> 32) | (self.key[2] << 32));
        self.v1[3] = self.mul1[3] ^ ((self.key[3] >> 32) | (self.key[3] << 32));
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
        let a3 = a3_unmasked & 0x3FFFFFFFFFFFFFFF;
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
        self.update(permuted)
    }

    fn update(&mut self, lanes: [u64; 4]) {
        for i in 0..4 {
            self.v1[i] = self.v1[i].wrapping_add(self.mul0[i].wrapping_add(lanes[i]));
            self.mul0[i] ^= (self.v1[i] & 0xffffffff).wrapping_mul(self.v0[i] >> 32);
            self.v0[i] = self.v0[i].wrapping_add(self.mul1[i]);
            self.mul1[i] ^= (self.v0[i] & 0xffffffff).wrapping_mul(self.v1[i] >> 32);
        }

        PortableHash::zipper_merge_and_add(self.v1[1], self.v1[0], &mut self.v0, 1, 0);
        PortableHash::zipper_merge_and_add(self.v1[3], self.v1[2], &mut self.v0, 3, 2);
        PortableHash::zipper_merge_and_add(self.v0[1], self.v0[0], &mut self.v1, 1, 0);
        PortableHash::zipper_merge_and_add(self.v0[3], self.v0[2], &mut self.v1, 3, 2);
    }

    fn zipper_merge_and_add(v1: u64, v0: u64, lane: &mut [u64; 4], add1: usize, add0: usize) {
        lane[add0] = lane[add0].wrapping_add(
            (((v0 & 0xff000000) | (v1 & 0xff00000000)) >> 24)
                | (((v0 & 0xff0000000000) | (v1 & 0xff000000000000)) >> 16)
                | (v0 & 0xff0000)
                | ((v0 & 0xff00) << 32)
                | ((v1 & 0xff00000000000000) >> 8)
                | (v0 << 56),
        );
        lane[add1] = lane[add1].wrapping_add(
            (((v1 & 0xff000000) | (v0 & 0xff00000000)) >> 24)
                | (v1 & 0xff0000)
                | ((v1 & 0xff0000000000) >> 16)
                | ((v1 & 0xff00) << 24)
                | ((v0 & 0xff000000000000) >> 8)
                | ((v1 & 0xff) << 48)
                | (v0 & 0xff00000000000000),
        );
    }

    fn update_packet(&mut self, packet: &[u8]) {
        self.update(PortableHash::to_lanes(packet));
    }

    fn to_lanes(packet: &[u8]) -> [u64; 4] {
        [
            LE::read_u64(&packet[0..8]),
            LE::read_u64(&packet[8..16]),
            LE::read_u64(&packet[16..24]),
            LE::read_u64(&packet[24..32]),
        ]
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
        let size_mod4 = bytes.len() & 3;
        let remainder_jump = bytes.len() & !3;
        let remainder = &bytes[remainder_jump..];
        let size = bytes.len() as u64;
        let mut packet: [u8; 32] = Default::default();

        packet[..remainder_jump].clone_from_slice(&bytes[..remainder_jump]);
        if size & 16 != 0 {
            for i in 0..4 {
                packet[28 + i] = bytes[remainder_jump + i + size_mod4 - 4];
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
        self.update_packet(&packet);
    }

    fn append(&mut self, data: &[u8]) {
        match self.buffer.fill(data) {
            Filled::Consumed => {}
            Filled::Full(new_data) => {
                let l = PortableHash::to_lanes(self.buffer.as_slice());
                self.update(l);

                let mut rest = &new_data[..];
                while rest.len() >= PACKET_SIZE {
                    self.update_packet(&rest);
                    rest = &rest[PACKET_SIZE..];
                }

                self.buffer.set_to(rest);
            }
        }
    }
}
