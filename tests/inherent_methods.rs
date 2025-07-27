use highway::{HighwayHasher, Key, PortableHash};

#[test]
fn test_highway_hasher_without_trait_import() {
    let key = Key([1, 2, 3, 4]);
    let mut hasher = HighwayHasher::new(key);

    // These should work without importing HighwayHash trait
    hasher.append(&[255]);
    let res64: u64 = hasher.finalize64();
    assert_eq!(0x07858f24d_2d79b2b2, res64);

    let mut hasher128 = HighwayHasher::new(key);
    hasher128.append(&[255]);
    let res128: [u64; 2] = hasher128.finalize128();
    assert_eq!([0xbb007d2462e77f3c, 0x224508f916b3991f], res128);

    let mut hasher256 = HighwayHasher::new(key);
    hasher256.append(&[255]);
    let res256: [u64; 4] = hasher256.finalize256();
    let expected: [u64; 4] = [
        0x7161cadbf7cd70e1,
        0xaac4905de62b2f5e,
        0x7b02b936933faa7,
        0xc8efcfc45b239f8d,
    ];
    assert_eq!(expected, res256);
}

#[test]
fn test_portable_hash_without_trait_import() {
    let key = Key([1, 2, 3, 4]);
    let mut hasher = PortableHash::new(key);

    // These should work without importing HighwayHash trait
    hasher.append(&[255]);
    let res64: u64 = hasher.finalize64();
    assert_eq!(0x07858f24d_2d79b2b2, res64);

    let mut hasher128 = PortableHash::new(key);
    hasher128.append(&[255]);
    let res128: [u64; 2] = hasher128.finalize128();
    assert_eq!([0xbb007d2462e77f3c, 0x224508f916b3991f], res128);

    let mut hasher256 = PortableHash::new(key);
    hasher256.append(&[255]);
    let res256: [u64; 4] = hasher256.finalize256();
    let expected: [u64; 4] = [
        0x7161cadbf7cd70e1,
        0xaac4905de62b2f5e,
        0x7b02b936933faa7,
        0xc8efcfc45b239f8d,
    ];
    assert_eq!(expected, res256);
}
