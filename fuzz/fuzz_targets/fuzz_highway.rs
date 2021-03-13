#![no_main]

use highway::{AvxHash, HighwayHash, HighwayHasher, Key, PortableHash, SseHash};
use libc::size_t;
use libfuzzer_sys::arbitrary;

extern "C" {
    fn HighwayHash64(data: *const u8, size: size_t, key: *const u64) -> u64;
}

#[derive(Debug, arbitrary::Arbitrary)]
pub struct FuzzKey {
    pub key: [u64; 4],
    pub data: Vec<u8>,
}

libfuzzer_sys::fuzz_target!(|input: FuzzKey| {
    let data = &input.data;
    let key = Key(input.key);
    let mut hashes = [0u64; 2];
    for hash in &mut hashes {
        let portable64 = PortableHash::new(key).hash64(data);
        *hash = portable64;
        let expected = unsafe { HighwayHash64(data.as_ptr(), data.len(), input.key.as_ptr()) };
        assert_eq!(portable64, expected);

        let portable128 = PortableHash::new(key).hash128(data);
        let portable256 = PortableHash::new(key).hash256(data);

        let builder64 = HighwayHasher::new(key).hash64(data);
        let builder128 = HighwayHasher::new(key).hash128(data);
        let builder256 = HighwayHasher::new(key).hash256(data);

        assert_eq!(builder64, portable64);
        assert_eq!(builder128, portable128);
        assert_eq!(builder256, portable256);

        if let Some(hash) = AvxHash::new(key).map(|x| x.hash64(data)) {
            assert_eq!(hash, portable64)
        }

        if let Some(hash) = AvxHash::new(key).map(|x| x.hash128(data)) {
            assert_eq!(hash, portable128)
        }

        if let Some(hash) = AvxHash::new(key).map(|x| x.hash256(data)) {
            assert_eq!(hash, portable256)
        }

        if let Some(hash) = SseHash::new(key).map(|x| x.hash64(data)) {
            assert_eq!(hash, portable64)
        }

        if let Some(hash) = SseHash::new(key).map(|x| x.hash128(data)) {
            assert_eq!(hash, portable128)
        }

        if let Some(hash) = SseHash::new(key).map(|x| x.hash256(data)) {
            assert_eq!(hash, portable256)
        }
    }

    assert_eq!(hashes[0], hashes[1]);
});
