#![allow(unused)]

extern crate byteorder;

use byteorder::{ByteOrder, LE};
use std::num::Wrapping;
use std::ops::Index;

pub struct Key([u64; 4]);

#[derive(Default)]
struct PortableState {
    v0: [u64; 4],
    v1: [u64; 4],
    mul0: [u64; 4],
    mul1: [u64; 4],
}

impl Index<usize> for Key {
    type Output = u64;
    fn index(&self, index: usize) -> &u64 {
        &self.0[index]
    }
}

impl PortableState {
    fn reset(&mut self, key: &Key) {
        self.mul0[0] = 0xdbe6d5d5fe4cce2f;
        self.mul0[1] = 0xa4093822299f31d0;
        self.mul0[2] = 0x13198a2e03707344;
        self.mul0[3] = 0x243f6a8885a308d3;
        self.mul1[0] = 0x3bd39e10cb0ef593;
        self.mul1[1] = 0xc0acf169b5f18a8c;
        self.mul1[2] = 0xbe5466cf34e90c6c;
        self.mul1[3] = 0x452821e638d01377;
        self.v0[0] = self.mul0[0] ^ key[0];
        self.v0[1] = self.mul0[1] ^ key[1];
        self.v0[2] = self.mul0[2] ^ key[2];
        self.v0[3] = self.mul0[3] ^ key[3];
        self.v1[0] = self.mul1[0] ^ ((key[0] >> 32) | (key[0] << 32));
        self.v1[1] = self.mul1[1] ^ ((key[1] >> 32) | (key[1] << 32));
        self.v1[2] = self.mul1[2] ^ ((key[2] >> 32) | (key[2] << 32));
        self.v1[3] = self.mul1[3] ^ ((key[3] >> 32) | (key[3] << 32));
    }
}

pub struct PortableHash;

impl PortableHash {
    pub fn hash64(data: &[u8], key: &Key) -> u64 {
        let mut state = PortableState::default();
        PortableHash::process_all(data, key, &mut state);
        PortableHash::finalize64(&mut state)
    }

    pub fn hash128(data: &[u8], key: &Key) -> u128 {
        0
    }

    pub fn hash256(data: &[u8], key: &Key) -> (u128, u128) {
        (0, 0)
    }

    fn finalize64(state: &mut PortableState) -> u64 {
        for i in 0..4 {
            PortableHash::permute_and_update(state);
        }

        state.v0[0]
            .wrapping_add(state.v1[0])
            .wrapping_add(state.mul0[0])
            .wrapping_add(state.mul1[0])
    }

    fn permute(v: &[u64; 4]) -> [u64; 4] {
        [
            (v[2] >> 32) | (v[2] << 32),
            (v[3] >> 32) | (v[3] << 32),
            (v[0] >> 32) | (v[0] << 32),
            (v[1] >> 32) | (v[1] << 32),
        ]
    }

    fn permute_and_update(state: &mut PortableState) {
        let permuted: [u64; 4] = PortableHash::permute(&state.v0);
        PortableHash::update(permuted, state)
    }

    fn update(lanes: [u64; 4], state: &mut PortableState) {
        for i in 0..4 {
            state.v1[i] = state.v1[i].wrapping_add(state.mul0[i].wrapping_add(lanes[i]));
            state.mul0[i] ^= (state.v1[i] & 0xffffffff).wrapping_mul(state.v0[i] >> 32);
            state.v0[i] = state.v0[i].wrapping_add(state.mul1[i]);
            state.mul1[i] ^= (state.v0[i] & 0xffffffff).wrapping_mul(state.v1[i] >> 32);
        }

        PortableHash::zipper_merge_and_add(state.v1[1], state.v1[0], &mut state.v0, 1, 0);
        PortableHash::zipper_merge_and_add(state.v1[3], state.v1[2], &mut state.v0, 3, 2);
        PortableHash::zipper_merge_and_add(state.v0[1], state.v0[0], &mut state.v1, 1, 0);
        PortableHash::zipper_merge_and_add(state.v0[3], state.v0[2], &mut state.v1, 3, 2);
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

    fn update_packet(packet: &[u8], state: &mut PortableState) {
        let lanes: [u64; 4] = [
            LE::read_u64(&packet[0..8]),
            LE::read_u64(&packet[8..16]),
            LE::read_u64(&packet[16..24]),
            LE::read_u64(&packet[24..32]),
        ];
        PortableHash::update(lanes, state);
    }

    fn process_all(data: &[u8], key: &Key, state: &mut PortableState) {
        state.reset(key);
        let mut slice = &data[..];
        while slice.len() >= 32 {
            PortableHash::update_packet(&slice, state);
            slice = &slice[32..];
        }

        if (!slice.is_empty()) {
            PortableHash::update_remainder(&slice, state);
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

    fn update_remainder(bytes: &[u8], state: &mut PortableState) {
        let size_mod4 = bytes.len() & 3;
        let remainder_jump = bytes.len() & !3;
        let remainder = &bytes[remainder_jump..];
        let size = bytes.len() as u64;
        let mut packet: [u8; 32] = Default::default();

        for i in 0..4 {
            state.v0[i] += (size << 32) + size;
        }

        PortableHash::rotate_32_by(size, &mut state.v1);
        packet[..remainder_jump].clone_from_slice(&bytes[..remainder_jump]);
        if (size & 16 != 0) {
            for i in 0..4 {
                packet[28 + i] = bytes[remainder_jump + i + size_mod4 - 4];
            }
        } else if (size_mod4 != 0) {
            packet[16] = remainder[0];
            packet[16 + 1] = remainder[size_mod4 >> 1];
            packet[16 + 2] = remainder[size_mod4 - 1];
        }
        PortableHash::update_packet(&packet, state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_hash_simple() {
        let key = Key([1, 2, 3, 4]);
        let b: Vec<u8> = (0..33).map(|x| 128 + x as u8).collect();
        let hash = PortableHash::hash64(&b[..], &key);
        assert_eq!(0x53c516cce478cad7, hash);
    }

    #[test]
    fn portable_hash_simple2() {
        let key = Key([1, 2, 3, 4]);
        let hash = PortableHash::hash64(&[(-1 as i8) as u8], &key);
        assert_eq!(0x7858f24d2d79b2b2, hash);
    }

    #[test]
    fn portable_hash_all() {
        let expected64 = [
            0x907A56DE22C26E53,
            0x7EAB43AAC7CDDD78,
            0xB8D0569AB0B53D62,
            0x5C6BEFAB8A463D80,
            0xF205A46893007EDA,
            0x2B8A1668E4A94541,
            0xBD4CCC325BEFCA6F,
            0x4D02AE1738F59482,
            0xE1205108E55F3171,
            0x32D2644EC77A1584,
            0xF6E10ACDB103A90B,
            0xC3BBF4615B415C15,
            0x243CC2040063FA9C,
            0xA89A58CE65E641FF,
            0x24B031A348455A23,
            0x40793F86A449F33B,
            0xCFAB3489F97EB832,
            0x19FE67D2C8C5C0E2,
            0x04DD90A69C565CC2,
            0x75D9518E2371C504,
            0x38AD9B1141D3DD16,
            0x0264432CCD8A70E0,
            0xA9DB5A6288683390,
            0xD7B05492003F028C,
            0x205F615AEA59E51E,
            0xEEE0C89621052884,
            0x1BFC1A93A7284F4F,
            0x512175B5B70DA91D,
            0xF71F8976A0A2C639,
            0xAE093FEF1F84E3E7,
            0x22CA92B01161860F,
            0x9FC7007CCF035A68,
            0xA0C964D9ECD580FC,
            0x2C90F73CA03181FC,
            0x185CF84E5691EB9E,
            0x4FC1F5EF2752AA9B,
            0xF5B7391A5E0A33EB,
            0xB9B84B83B4E96C9C,
            0x5E42FE712A5CD9B4,
            0xA150F2F90C3F97DC,
            0x7FA522D75E2D637D,
            0x181AD0CC0DFFD32B,
            0x3889ED981E854028,
            0xFB4297E8C586EE2D,
            0x6D064A45BB28059C,
            0x90563609B3EC860C,
            0x7AA4FCE94097C666,
            0x1326BAC06B911E08,
            0xB926168D2B154F34,
            0x9919848945B1948D,
            0xA2A98FC534825EBE,
            0xE9809095213EF0B6,
            0x582E5483707BC0E9,
            0x086E9414A88A6AF5,
            0xEE86B98D20F6743D,
            0xF89B7FF609B1C0A7,
            0x4C7D9CC19E22C3E8,
            0x9A97005024562A6F,
            0x5DD41CF423E6EBEF,
            0xDF13609C0468E227,
            0x6E0DA4F64188155A,
            0xB755BA4B50D7D4A1,
            0x887A3484647479BD,
            0xAB8EEBE9BF2139A0,
            0x75542C5D4CD2A6FF,
        ];

        let data: Vec<u8> = (0..65).map(|x| x as u8).collect();
        let key = Key([
            0x0706050403020100,
            0x0F0E0D0C0B0A0908,
            0x1716151413121110,
            0x1F1E1D1C1B1A1918,
        ]);

        for i in 0..64 {
            assert_eq!(expected64[i], PortableHash::hash64(&data[..i], &key));
        }
    }
}
