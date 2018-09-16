use byteorder::{ByteOrder, LE};
use key::Key;
use traits::HighwayHash;

#[derive(Default)]
pub struct PortableHash {
    key: Key,
    v0: [u64; 4],
    v1: [u64; 4],
    mul0: [u64; 4],
    mul1: [u64; 4],
}

impl HighwayHash for PortableHash {
    fn hash64(data: &[u8], key: &Key) -> u64 {
        let mut hash = Self::new(key);
        hash.process_all(data);
        hash.finalize64()
    }

    fn hash128(data: &[u8], key: &Key) -> u128 {
        let mut hash = Self::new(key);
        hash.process_all(data);
        hash.finalize128()
    }

    fn hash256(data: &[u8], key: &Key) -> (u128, u128) {
        let mut hash = Self::new(key);
        hash.process_all(data);
        hash.finalize256()
    }
}

impl PortableHash {
    pub fn new(key: &Key) -> Self {
        PortableHash {
            key: key.clone(),
            ..Default::default()
        }
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
        for _i in 0..4 {
            self.permute_and_update();
        }

        self.v0[0]
            .wrapping_add(self.v1[0])
            .wrapping_add(self.mul0[0])
            .wrapping_add(self.mul1[0])
    }

    fn finalize128(&mut self) -> u128 {
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

        u128::from(low) + (u128::from(high) << 64)
    }

    fn finalize256(&mut self) -> (u128, u128) {
        for _i in 0..10 {
            self.permute_and_update();
        }

        let low = PortableHash::module_reduction(
            self.v1[1].wrapping_add(self.mul1[1]),
            self.v1[0].wrapping_add(self.mul1[0]),
            self.v0[1].wrapping_add(self.mul0[1]),
            self.v0[0].wrapping_add(self.mul0[0]),
        );
        let high = PortableHash::module_reduction(
            self.v1[3].wrapping_add(self.mul1[3]),
            self.v1[2].wrapping_add(self.mul1[2]),
            self.v0[3].wrapping_add(self.mul0[3]),
            self.v0[2].wrapping_add(self.mul0[2]),
        );

        (low, high)
    }

    fn module_reduction(a3_unmasked: u64, a2: u64, a1: u64, a0: u64) -> u128 {
        let a3 = a3_unmasked & 0x3FFFFFFFFFFFFFFF;
        let high = a1 ^ ((a3 << 1) | (a2 >> 63)) ^ ((a3 << 2) | (a2 >> 62));
        let low = a0 ^ (a2 << 1) ^ (a2 << 2);
        u128::from(low) + (u128::from(high) << 64)
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
        let lanes: [u64; 4] = [
            LE::read_u64(&packet[0..8]),
            LE::read_u64(&packet[8..16]),
            LE::read_u64(&packet[16..24]),
            LE::read_u64(&packet[24..32]),
        ];

        self.update(lanes);
    }

    fn process_all(&mut self, data: &[u8]) {
        self.reset();
        let mut slice = &data[..];
        while slice.len() >= 32 {
            self.update_packet(&slice);
            slice = &slice[32..];
        }

        if !slice.is_empty() {
            self.update_remainder(&slice);
        }
    }

    fn rotate_32_by(count: u64, lanes: &mut [u64; 4]) {
        for i in 0..4 {
            let half0: u32 = lanes[i] as u32;
            let half1: u32 = (lanes[i] >> 32) as u32;
            lanes[i] = u64::from((half0 << count) | (half0 >> (32 - count)));
            lanes[i] |= u64::from((half1 << count) | (half1 >> (32 - count))) << 32;
        }
    }

    fn update_remainder(&mut self, bytes: &[u8]) {
        let size_mod4 = bytes.len() & 3;
        let remainder_jump = bytes.len() & !3;
        let remainder = &bytes[remainder_jump..];
        let size = bytes.len() as u64;
        let mut packet: [u8; 32] = Default::default();

        for i in 0..4 {
            self.v0[i] += (size << 32) + size;
        }

        PortableHash::rotate_32_by(size, &mut self.v1);
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

        self.update_packet(&packet);
    }
}
