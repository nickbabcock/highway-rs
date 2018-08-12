#![allow(unused)]

extern crate byteorder;

use byteorder::{ByteOrder, LE};
use std::num::Wrapping;
use std::ops::Index;

#[derive(Default, Clone)]
pub struct Key([u64; 4]);

impl Index<usize> for Key {
    type Output = u64;
    fn index(&self, index: usize) -> &u64 {
        &self.0[index]
    }
}

#[derive(Default)]
pub struct PortableHash {
    key: Key,
    v0: [u64; 4],
    v1: [u64; 4],
    mul0: [u64; 4],
    mul1: [u64; 4],
}

impl PortableHash {
    pub fn new(key: &Key) -> Self {
        PortableHash { key: key.clone(), ..Default::default() }
    }

    pub fn hash64(data: &[u8], key: &Key) -> u64 {
        let mut hash = PortableHash::new(key);
        hash.process_all(data);
        hash.finalize64()
    }

    pub fn hash128(data: &[u8], key: &Key) -> u128 {
        let mut hash = PortableHash::new(key);
        hash.process_all(data);
        hash.finalize128()
    }

    pub fn hash256(data: &[u8], key: &Key) -> (u128, u128) {
        (0, 0)
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
        for i in 0..4 {
            self.permute_and_update();
        }

        self.v0[0]
            .wrapping_add(self.v1[0])
            .wrapping_add(self.mul0[0])
            .wrapping_add(self.mul1[0])
    }

    fn finalize128(&mut self) -> u128 {
        for i in 0..6 {
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

        if (!slice.is_empty()) {
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
        if (size & 16 != 0) {
            for i in 0..4 {
                packet[28 + i] = bytes[remainder_jump + i + size_mod4 - 4];
            }
        } else if (size_mod4 != 0) {
            packet[16] = remainder[0];
            packet[16 + 1] = remainder[size_mod4 >> 1];
            packet[16 + 2] = remainder[size_mod4 - 1];
        }

        self.update_packet(&packet);
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

       let expected128 = [
   0x33565E767F093E6F_0FED268F9D8FFEC7,
   0xDC291DF9EB9CDCB4_D6B0A8893681E7A8,
   0x78085638DC32E868_3D15AD265A16DA04,
   0xBFE69A0FD9CEDD79_0607621B295F0BEB,
   0x2E922AD039319208_26399EB46DACE49E,
   0x193810906C63C23A_3250BDC386D12ED8,
   0x7CDE576F37ED1019_6F476AB3CB896547,
   0xBE1F03FF9F02796C_2A401FCA697171B4,
   0x695CF1C63BEC0AC2_A1E96D84280552E8,
   0x1A85B98C5B5000CC_142A2102F31E63B2,
   0x929E1F3B2DA45559_51A1B70E26B6BC5B,
   0xBED21F22C47B7D13_88990362059A415B,
   0xA818BA8CE0F9C8D4_CD1F1F5F1CAF9566,
   0xB2E94C78B8DDB848_A225564112FE6157,
   0xCECD1DBC025641A2_BD492FEBD1CC0919,
   0xE0796C0B6E26BCD7_142237A52BC4AF54,
   0x029EA3D5019F18C8_414460FFD5A401AD,
   0xECB878B1169B5EA0_C52A4B96C51C9962,
   0xF93A46D616F8D531_D940CA8F11FBEACE,
   0x3FFDBF8DF51D7C93_8AC49D0AE5C0CBF5,
   0x7DCD3A6BA5EBAA46_AC6D279B852D00A8,
   0x3173C398163DD9D5_F11621BD93F08A56,
   0xB3123CDA411898ED_0C4CE250F68CF89F,
   0x7CE274479169080E_15AB97ED3D9A51CE,
   0xD0D9D98BD8AA2D77_CD001E198D4845B8,
   0x7DD304F6397F7E16_34F3D617A0493D79,
   0x130829166567304F_5CB56890A9F4C6B6,
   0x6F828B7E3FD9748C_30DA6F8B245BD1C0,
   0x93F6DA0CAC5F441C_E0580349204C12C0,
   0x5FB897114FB65976_F648731BA5073045,
   0x509A4918EB7E0991_024F8354738A5206,
   0x52415E3A07F5D446_06E7B465E8A57C29,
   0x16FC1958F9B3E4B9_1984DF66C1434AAA,
   0xF958B59DE5A2849D_111678AFE0C6C36C,
   0xC96ED5D243658536_773FBC8440FB0490,
   0xEA336A0BC1EEACE9_91E3DC710BB6C941,
   0xF2E94F8C828FC59E_25CFE3815D7AD9D4,
   0x7479C4C8F850EC04_B9FB38B83CC288F2,
   0x6E26B1C16F48DBF4_1D85D5C525982B8C,
   0x2134D599058B3FD0_8A4E55BD6060BDE7,
   0xE8052D1AE61D6423_2A958FF994778F36,
   0x3ACF9C87D7E8C0B9_89233AE6BE453233,
   0x418FB49BCA2A5140_4458F5E27EA9C8D5,
   0x1017F69633C861E6_090301837ED12A68,
   0x339DF1AD3A4BA6E4_330DD84704D49590,
   0x363B3D95E3C95EF6_569363A663F2C576,
   0x2BA0E8087D4E28E9_ACC8D08586B90737,
   0x8DB620A45160932E_39C27A27C86D9520,
   0x6ED3561A10E47EE6_8E6A4AEB671A072D,
   0xD80E6E656EDE842E_0011D765B1BEC74A,
   0xCE088794D7088A7D_2515D62B936AC64C,
   0x264F0094EB23CCEF_91621552C16E23AF,
   0xD8654807D3A31086_1E21880D97263480,
   0xA517E1E09D074739_39D76AAF097F432D,
   0x2F51215F69F976D4_0F17A4F337C65A14,
   0x568C3DC4D1F13CD1_A0FB5CDA12895E44,
   0xBAD5DA947E330E69_93C8FC00D89C46CE,
   0x584D6EE72CBFAC2B_817C07501D1A5694,
   0xF98E647683C1E0ED_91D668AF73F053BF,
   0xBC4CC3DF166083D8_5281E1EF6B3CCF8B,
   0xFF969D000C16787B_AAD61B6DBEAAEEB9,
   0x14B919BD905F1C2D_4325D84FC0475879,
   0xF1F720C5A53A2B86_79A176D1AA6BA6D1,
   0x3AEA94A8AD5F4BCB_74BD7018022F3EF0,
   0xE0BC0571DE918FC8_98BB1F7198D4C4F2];
 

        let data: Vec<u8> = (0..65).map(|x| x as u8).collect();
        let key = Key([
            0x0706050403020100,
            0x0F0E0D0C0B0A0908,
            0x1716151413121110,
            0x1F1E1D1C1B1A1918,
        ]);

        for i in 0..64 {
            assert_eq!(expected64[i], PortableHash::hash64(&data[..i], &key));
            assert_eq!(expected128[i], PortableHash::hash128(&data[..i], &key));
        }
    }
}
