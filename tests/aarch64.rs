#![cfg(target_arch = "aarch64")]
use highway::{HighwayHash, Key, NeonHash, PortableHash};

#[test]
fn hash_zeroes() {
    let key = Key([0, 0, 0, 0]);
    let hash = unsafe { NeonHash::force_new(key) }.hash64(&[]);
    assert_eq!(0x7035_DA75_B9D5_4469, hash);
}

#[test]
fn hash_simple() {
    let key = Key([1, 2, 3, 4]);
    let b: Vec<u8> = (0..33).map(|x| 128 + x as u8).collect();
    let hash = unsafe { NeonHash::force_new(key) }.hash64(&b[..]);
    assert_eq!(0x53c5_16cc_e478_cad7, hash);
}

#[test]
fn neon_eq_portable() {
    let data: Vec<u8> = (0..100).map(|x| x as u8).collect();
    let key = Key([
        0x0706_0504_0302_0100,
        0x0F0E_0D0C_0B0A_0908,
        0x1716_1514_1312_1110,
        0x1F1E_1D1C_1B1A_1918,
    ]);

    for i in 0..data.len() {
        let hash64 = PortableHash::new(key).hash64(&data[..i]);
        assert_eq!(
            unsafe { NeonHash::force_new(key) }.hash64(&data[..i]),
            hash64
        );

        let (head, tail) = &data[..i].split_at(i / 2);
        let mut hasher = unsafe { NeonHash::force_new(key) };
        hasher.append(head);
        let mut snd = unsafe { NeonHash::force_from_checkpoint(hasher.checkpoint()) };
        snd.append(tail);
        assert_eq!(hash64, snd.finalize64());

        assert_eq!(
            unsafe { NeonHash::force_new(key) }.hash128(&data[..i]),
            PortableHash::new(key).hash128(&data[..i])
        );

        assert_eq!(
            unsafe { NeonHash::force_new(key) }.hash256(&data[..i]),
            PortableHash::new(key).hash256(&data[..i])
        );
    }
}
