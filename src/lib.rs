#![allow(unused)]

extern crate byteorder;

use byteorder::{ByteOrder, LE};
use std::ops::Index;
use std::ops::{
    AddAssign, BitAndAssign, BitOrAssign, BitXor, BitXorAssign, ShlAssign, ShrAssign, SubAssign,
};

#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[derive(Clone)]
struct V2x64U(__m128i);

impl Copy for V2x64U {}

macro_rules! _mm_shuffle {
    ($z:expr, $y:expr, $x:expr, $w:expr) => {
        ($z << 6) | ($y << 4) | ($x << 2) | $w
    };
}

impl V2x64U {
    fn new(hi: u64, low: u64) -> Self {
        V2x64U(unsafe { _mm_set_epi64x(hi as i64, low as i64) })
    }

    fn rotate_by_32(&self) -> Self {
        V2x64U(unsafe { _mm_shuffle_epi32(self.0, _mm_shuffle!(2, 3, 0, 1)) })
    }

    fn shuffle(&self, mask: &V2x64U) -> Self {
        V2x64U::from(unsafe { _mm_shuffle_epi8(self.0, mask.0) })
    }
}

impl From<__m128i> for V2x64U {
    fn from(v: __m128i) -> Self {
        V2x64U(v)
    }
}

impl From<[u64; 2]> for V2x64U {
    fn from(v: [u64; 2]) -> Self {
        let result = V2x64U(unsafe { _mm_loadu_si128((&v[0] as *const u64) as *const __m128i) });
        std::mem::forget(v);
        result
    }
}

impl AddAssign for V2x64U {
    fn add_assign(&mut self, other: Self) {
        self.0 = unsafe { _mm_add_epi64(self.0, other.0) }
    }
}

impl SubAssign for V2x64U {
    fn sub_assign(&mut self, other: Self) {
        self.0 = unsafe { _mm_sub_epi64(self.0, other.0) }
    }
}

impl BitAndAssign for V2x64U {
    fn bitand_assign(&mut self, other: Self) {
        self.0 = unsafe { _mm_and_si128(self.0, other.0) }
    }
}

impl BitOrAssign for V2x64U {
    fn bitor_assign(&mut self, other: Self) {
        self.0 = unsafe { _mm_or_si128(self.0, other.0) }
    }
}

impl BitXorAssign for V2x64U {
    fn bitxor_assign(&mut self, other: Self) {
        self.0 = unsafe { _mm_xor_si128(self.0, other.0) }
    }
}

impl BitXor for V2x64U {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self {
        let mut new = V2x64U(self.0);
        new ^= other;
        new
    }
}

impl ShlAssign<__m128i> for V2x64U {
    fn shl_assign(&mut self, count: __m128i) {
        self.0 = unsafe { _mm_sll_epi64(self.0, count) }
    }
}

impl ShrAssign<__m128i> for V2x64U {
    fn shr_assign(&mut self, count: __m128i) {
        self.0 = unsafe { _mm_srl_epi64(self.0, count) }
    }
}

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

impl SseHash {
    fn reset(&mut self) {
        let init0L = V2x64U::new(0xa4093822299f31d0, 0xdbe6d5d5fe4cce2f);
        let init0H = V2x64U::new(0x243f6a8885a308d3, 0x13198a2e03707344);
        let init1L = V2x64U::new(0xc0acf169b5f18a8c, 0x3bd39e10cb0ef593);
        let init1H = V2x64U::new(0x452821e638d01377, 0xbe5466cf34e90c6c);
        let keyL = V2x64U::from([self.key[0], self.key[1]]);
        let keyH = V2x64U::from([self.key[2], self.key[3]]);
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
            self.mul0H ^= V2x64U(_mm_mul_epu32(self.v1H.0, _mm_srli_epi16(self.v0H.0, 32)));
            self.v0L += self.mul1L;
            self.v0H += self.mul1H;
            self.mul1L ^= V2x64U(_mm_mul_epu32(self.v0L.0, self.v1L.rotate_by_32().0));
            self.mul1H ^= V2x64U(_mm_mul_epu32(self.v0H.0, _mm_srli_epi16(self.v1H.0, 32)));
            self.v0L += SseHash::zipper_merge(&self.v1L);
            self.v0H += SseHash::zipper_merge(&self.v1H);
            self.v1L += SseHash::zipper_merge(&self.v0L);
            self.v1H += SseHash::zipper_merge(&self.v0H);
        }
    }
}

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
        PortableHash {
            key: key.clone(),
            ..Default::default()
        }
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
        let mut hash = PortableHash::new(key);
        hash.process_all(data);
        hash.finalize256()
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

    fn finalize256(&mut self) -> (u128, u128) {
        for i in 0..10 {
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
            0xE0BC0571DE918FC8_98BB1F7198D4C4F2,
        ];

        let expected256 = [
            (
                0xD946017313C7351F_DD44482AC2C874F5,
                0x41DA233145751DF4_B3AEBECCB98714FF,
            ),
            (
                0xE20D44EF3DCAC60F_EDB941BCE45F8254,
                0x2073624CB275E484_72651B9BCB324A47,
            ),
            (
                0x11C4BF1A1B0AE873_3FDFF9DF24AFE454,
                0x1208F6590D33B42C_115169CC6922597A,
            ),
            (
                0x89225E7C6911D1D0_480AA0D70DD1D95C,
                0xE23DFBC390E1C722_8EA8426B8BBB865A,
            ),
            (
                0xA85F9DF6AFD2929B_C9CFC497212BE4DC,
                0x07E4277A374D4F9B_1FDA9F211DF4109E,
            ),
            (
                0xBF4B63BA5E460142_B4B4F566A4DC85B3,
                0x0F74587D388085C6_15F48E68CDDC1DE3,
            ),
            (
                0xA99CFB2784B4CEB6_6445C70A86ADB9B4,
                0xB6526DF29A9D1170_DAE29D40A0B2DB13,
            ),
            (
                0xA4F1F838EB8C6D37_D666B1A00987AD81,
                0x5754D67D062C526C_E9226E07D463E030,
            ),
            (
                0xE6976FF3FCFF3A45_F1B905B0ED768BC0,
                0xD9A0AFEB371E0D33_4FBE518DD9D09778,
            ),
            (
                0xF10FBBD16424F1A1_80D8E4D70D3C2981,
                0xC0BFE8F701B673F2_CF5C2DBE9D3F0CD1,
            ),
            (
                0x8E9492B1FDFE38E0_ADE48C50E5A262BE,
                0x0E41D574DB656DCD_0784B74B2FE9B838,
            ),
            (
                0xBA97A7DE6A1A9738_A1BE77B9531807CF,
                0x3E39B935C74CE8E8_AF274CEF9C8E261F,
            ),
            (
                0x9D11CBDC39E853A0_15AD3802E3405857,
                0x6CD9E9E3CAF4212E_23EA3E993C31B225,
            ),
            (
                0xA367F9C1531F95A6_01C96F5EB1D77C36,
                0x97F1000ABF3BD5D3_1F94A3427CDADCB8,
            ),
            (
                0x0E0C28FA6E21DF5D_0815E91EEEFF8E41,
                0x3FFD01DA1C9D73E6_4EAD8E62ED095374,
            ),
            (
                0x62C3DB018501B146_C11905707842602E,
                0xC884F87BD4FEC347_85F5AD17FA3406C1,
            ),
            (
                0xF7F075D62A627BD9_F51AD989A1B6CD1F,
                0x1AD415C16A174D9F_7E01D5F579F28A06,
            ),
            (
                0x3B9D4ABD3A9275B9_19F4CFA82CA4068E,
                0x8884D50949215613_8000B0DDE9C010C6,
            ),
            (
                0x4EDAA3C5097716EE_126D6C7F81AB9F5D,
                0x9001AC85AA80C32D_AF121573A7DD3E49,
            ),
            (
                0xDF864F4144E71C3D_06AABEF9149155FA,
                0xDE2BA54792491CB6_FDBABCE860BC64DA,
            ),
            (
                0xA087B7328E486E65_ADFC6B4035079FDB,
                0xE3895C440D3CEE44_46D1A9935A4623EA,
            ),
            (
                0x8F3024E20A06E133_B5F9D31DEEA3B3DF,
                0x703F1DCF9BD69749_F24C38C8288FE120,
            ),
            (
                0x1C5D3F969BDACEA0_2B3C0B854794EFE3,
                0x23441C5A79D03075_81F16AAFA563AC2E,
            ),
            (
                0xBC6B8E9461D7F924_418AF8C793FD3762,
                0x3AA0B7BFD417CA6E_776FF26A2A1A9E78,
            ),
            (
                0x0185FEE5B59C1B2A_CD03EA2AD255A3C1,
                0xBE69DD67F83B76E4_D1F438D44F9773E4,
            ),
            (
                0x2C7B31D2A548E0AE_F951A8873887A0FB,
                0xA3C78EC7BE219F72_44803838B6186EFA,
            ),
            (
                0x4B7E8997B4F63488_958FF151EA0D8C08,
                0xD95577556F20EEFA_C78E074351C5386D,
            ),
            (
                0x3318F884351F578C_29A917807FB05406,
                0xE74393465E97AEFF_DD24EA6EF6F6A7FA,
            ),
            (
                0x1FD0D271B09F97DA_98240880935E6CCB,
                0x291649F99F747817_56E786472700B183,
            ),
            (
                0xFFDB2EFF7C596CEB_1BD4954F7054C556,
                0x0F037670537FC153_7C6AC69A1BAB6B5B,
            ),
            (
                0x647CF6EBAF6332C1_8825E38897597498,
                0x72D7632C00BFC5AB_552BD903DC28C917,
            ),
            (
                0xB3728B20B10FB7DA_6880E276601A644D,
                0x8AEF14EF33452EF2_D0BD12060610D16E,
            ),
            (
                0x42D56326A3C11289_BCE38C9039A1C3FE,
                0xC9B03C6BC9475A99_E35595F764FCAEA9,
            ),
            (
                0x6C36EA75BFCE46D0_F60115CBF034A6E5,
                0x7EDAA2ED11007A35_3B17C8D382725990,
            ),
            (
                0xC4776801739F720C_1326E959EDF9DEA2,
                0x8A0DD0D90A2529AB_5169500FD762F62F,
            ),
            (
                0xFF6BB41302DAD144_935149D503D442D4,
                0xE61D53619ECC2230_339CB012CD9D36EC,
            ),
            (
                0xB8AEECA36084E1FC_528BC888AA50B696,
                0x02C14AAD097CEC44_A158151EC0243476,
            ),
            (
                0x1EE65114F760873F_BED688A72217C327,
                0xDDF2E895631597B9_3F5C26B37D3002A6,
            ),
            (
                0xFAFC6324F4B0AB6C_E7DB21CF2B0B51AD,
                0xF0AD888D1E05849C_B0857244C22D9C5B,
            ),
            (
                0x3C594A3163067DEB_05519793CD4DCB00,
                0x5AC86297805CB094_AC75081ACF119E34,
            ),
            (
                0x19644DB2516B7E84_09228D8C22B5779E,
                0x7F785AD725E19391_2B92C8ABF83141A0,
            ),
            (
                0x5EA53C65CA036064_59C42E5D46D0A74B,
                0xBAE6DF143F54E9D4_48A9916BB635AEB4,
            ),
            (
                0xD53D78BCB41DA092_5EB623696D03D0E3,
                0x64802457632C8C11_FE2348DC52F6B10D,
            ),
            (
                0xC6318C25717E80A1_43B61BB2C4B85481,
                0xBD0217E035401D7C_8C4A7F4D6F9C687D,
            ),
            (
                0xB04C4D5EB11D703A_7F51CA5743824C37,
                0xD66775EA215456E2_4D511E1ECBF6F369,
            ),
            (
                0x52B8E8C459FC79B3_39B409EEF87E45CC,
                0x80F07B645EEE0149_44920918D1858C24,
            ),
            (
                0xBFA19026526836E7_CE8694D1BE9AD514,
                0x380C4458D696E1FE_1EA4FDF6E4902A7D,
            ),
            (
                0x1F3B353BE501A7D7_D189E18BF823A0A4,
                0x7E94646F74F9180C_A24F77B4E02E2884,
            ),
            (
                0x2C2E0AA414038D0B_AFF8C635D325EC48,
                0x39EC38E33B501489_4ED37F611A447467,
            ),
            (
                0x013D3E6EBEF274CC_2A2BFDAD5F83F197,
                0xF15A8A5DE932037E_E1563C0477726155,
            ),
            (
                0x10110B9BF9B1FF11_D5D1F91EC8126332,
                0x87BADC5728701552_A175AB26541C6032,
            ),
            (
                0xDDA62AB61B2EEEFB_C7B5A92CD8082884,
                0x6B38BD5CC01F4FFB_8F9882ECFEAE732F,
            ),
            (
                0xA3F0822DA2BF7D8B_CF6EF275733D32F0,
                0x0B28E3EFEBB3172D_304E7435F512406A,
            ),
            (
                0x66AE2A819A8A8828_E698F80701B2E9DB,
                0xA7416170523EB5A4_14EA9024C9B8F2C9,
            ),
            (
                0x17B4DEDAE34452C1_3A917E87E307EDB7,
                0x29CE6BFE789CDD0E_F689F162E711CC70,
            ),
            (
                0x47CD9EAD4C0844A2_0EFF3AD8CB155D8E,
                0xDEF3C25DF0340A51_46C8E40EE6FE21EB,
            ),
            (
                0x32AB0D600717136D_03FD86E62B82D04D,
                0x138CE3F1443739B1_682B0E832B857A89,
            ),
            (
                0x1053E0A9D9ADBFEA_2F77C754C4D7F902,
                0xC48A829C72DD83CA_58E66368544AE70A,
            ),
            (
                0x31BE9E01A8C7D314_F900EB19E466A09F,
                0xB8C0EB0F87FFE7FB_3AFEC6B8CA08F471,
            ),
            (
                0x53CE6877E11AA57B_DB277D8FBE3C8EFB,
                0xB345B56392453CC9_719C94D20D9A7E7D,
            ),
            (
                0x6095E7B336466DC8_37639C3BDBA4F2C9,
                0x82C988CDE5927CD5_3A8049791E65B88A,
            ),
            (
                0x20562E255BA6467E_6B1FB1A714234AE4,
                0xF40CE3FBE41ED768_3E2B892D40F3D675,
            ),
            (
                0x8FC2AAEFF63D266D_8EE11CB1B287C92A,
                0x578AA91DE8D56873_66643487E6EB9F03,
            ),
            (
                0x83B040BE4DEC1ADD_F5B1F8266A3AEB67,
                0xF4A3A447DEFED79F_7FE1C8635B26FBAE,
            ),
            (
                0x1A422A196EDAC1F2_90D8E6FF6AC12475,
                0xC1BDD7C4C351CFBE_9E3765FE1F8EB002,
            ),
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
            assert_eq!(expected128[i], PortableHash::hash128(&data[..i], &key));
            assert_eq!(expected256[i], PortableHash::hash256(&data[..i], &key));
        }
    }
}
