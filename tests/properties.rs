#[macro_use]
extern crate quickcheck_macros;

mod quick_tests {
    use highway::{HighwayHash, HighwayHasher, Key, PortableHash};

    #[quickcheck]
    fn portable64_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
        let key = Key([k1, k2, k3, k4]);
        let hash1 = PortableHash::new(key).hash64(data.as_slice());
        let hash2 = PortableHash::new(key).hash64(data.as_slice());
        hash1 == hash2
    }

    #[quickcheck]
    fn portable128_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
        let key = Key([k1, k2, k3, k4]);
        let hash1 = PortableHash::new(key).hash128(data.as_slice());
        let hash2 = PortableHash::new(key).hash128(data.as_slice());
        hash1 == hash2
    }

    #[quickcheck]
    fn portable256_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
        let key = Key([k1, k2, k3, k4]);
        let hash1 = PortableHash::new(key).hash256(data.as_slice());
        let hash2 = PortableHash::new(key).hash256(data.as_slice());
        hash1 == hash2
    }

    #[quickcheck]
    fn builder64_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
        let key = Key([k1, k2, k3, k4]);
        let hash1 = HighwayHasher::new(key).hash64(data.as_slice());
        let hash2 = HighwayHasher::new(key).hash64(data.as_slice());
        hash1 == hash2
    }

    #[quickcheck]
    fn builder128_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
        let key = Key([k1, k2, k3, k4]);
        let hash1 = HighwayHasher::new(key).hash128(data.as_slice());
        let hash2 = HighwayHasher::new(key).hash128(data.as_slice());
        hash1 == hash2
    }

    #[quickcheck]
    fn builder256_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
        let key = Key([k1, k2, k3, k4]);
        let hash1 = HighwayHasher::new(key).hash256(data.as_slice());
        let hash2 = HighwayHasher::new(key).hash256(data.as_slice());
        hash1 == hash2
    }

    #[quickcheck]
    fn all64_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
        let key = Key([k1, k2, k3, k4]);
        let hash1 = PortableHash::new(key).hash64(data.as_slice());
        let hash2 = HighwayHasher::new(key).hash64(data.as_slice());

        #[cfg(target_arch = "x86_64")]
        {
            use highway::{AvxHash, SseHash};
            let mut res = hash1 == hash2;
            if let Some(h) = AvxHash::new(key) {
                res &= h.hash64(data.as_slice()) == hash1;
            }

            if let Some(h) = SseHash::new(key) {
                res &= h.hash64(data.as_slice()) == hash1;
            }
            res
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            hash1 == hash2
        }
    }

    #[quickcheck]
    fn all128_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
        let key = Key([k1, k2, k3, k4]);
        let hash1 = PortableHash::new(key).hash128(data.as_slice());
        let hash2 = HighwayHasher::new(key).hash128(data.as_slice());

        #[cfg(target_arch = "x86_64")]
        {
            use highway::{AvxHash, SseHash};
            let mut res = hash1 == hash2;
            if let Some(h) = AvxHash::new(key) {
                res &= h.hash128(data.as_slice()) == hash1;
            }

            if let Some(h) = SseHash::new(key) {
                res &= h.hash128(data.as_slice()) == hash1;
            }
            res
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            hash1 == hash2
        }
    }

    #[quickcheck]
    fn all256_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
        let key = Key([k1, k2, k3, k4]);
        let hash1 = PortableHash::new(key).hash256(data.as_slice());
        let hash2 = HighwayHasher::new(key).hash256(data.as_slice());

        #[cfg(target_arch = "x86_64")]
        {
            use highway::{AvxHash, SseHash};
            let mut res = hash1 == hash2;
            if let Some(h) = AvxHash::new(key) {
                res &= h.hash256(data.as_slice()) == hash1;
            }

            if let Some(h) = SseHash::new(key) {
                res &= h.hash256(data.as_slice()) == hash1;
            }
            res
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            hash1 == hash2
        }
    }

    #[quickcheck]
    fn checkpoint_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) {
        let key = Key([k1, k2, k3, k4]);
        let (head, tail) = data.split_at(data.len() / 2);

        let hash1 = PortableHash::new(key).hash256(data.as_slice());

        let mut hasher = PortableHash::new(key);
        hasher.append(head);
        let mut snd = PortableHash::from_checkpoint(hasher.checkpoint());
        snd.append(tail);
        assert_eq!(hash1.as_slice(), snd.finalize256().as_slice());

        let mut hasher = HighwayHasher::new(key);
        hasher.append(head);
        let mut snd = HighwayHasher::from_checkpoint(hasher.checkpoint());
        snd.append(tail);
        assert_eq!(hash1.as_slice(), snd.finalize256().as_slice());

        #[cfg(target_arch = "x86_64")]
        {
            use highway::SseHash;
            if let Some(mut hasher) = SseHash::new(key) {
                hasher.append(head);
                let mut snd = unsafe { SseHash::force_from_checkpoint(hasher.checkpoint()) };
                snd.append(tail);
                assert_eq!(hash1.as_slice(), snd.finalize256().as_slice());
            }
        }
    }
}

#[cfg(target_arch = "x86_64")]
mod quick_simd_tests {
    use highway::{AvxHash, HighwayHash, Key, SseHash};

    #[quickcheck]
    fn avx64_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
        let key = Key([k1, k2, k3, k4]);
        let hash1 = AvxHash::new(key).map(|x| x.hash64(data.as_slice()));
        let hash2 = AvxHash::new(key).map(|x| x.hash64(data.as_slice()));
        hash1 == hash2
    }

    #[quickcheck]
    fn avx128_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
        let key = Key([k1, k2, k3, k4]);
        let hash1 = AvxHash::new(key).map(|x| x.hash128(data.as_slice()));
        let hash2 = AvxHash::new(key).map(|x| x.hash128(data.as_slice()));
        hash1 == hash2
    }

    #[quickcheck]
    fn avx256_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
        let key = Key([k1, k2, k3, k4]);
        let hash1 = AvxHash::new(key).map(|x| x.hash256(data.as_slice()));
        let hash2 = AvxHash::new(key).map(|x| x.hash256(data.as_slice()));
        hash1 == hash2
    }

    #[quickcheck]
    fn sse64_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
        let key = Key([k1, k2, k3, k4]);
        let hash1 = SseHash::new(key).map(|x| x.hash64(data.as_slice()));
        let hash2 = SseHash::new(key).map(|x| x.hash64(data.as_slice()));
        hash1 == hash2
    }

    #[quickcheck]
    fn sse128_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
        let key = Key([k1, k2, k3, k4]);
        let hash1 = SseHash::new(key).map(|x| x.hash128(data.as_slice()));
        let hash2 = SseHash::new(key).map(|x| x.hash128(data.as_slice()));
        hash1 == hash2
    }

    #[quickcheck]
    fn sse256_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
        let key = Key([k1, k2, k3, k4]);
        let hash1 = SseHash::new(key).map(|x| x.hash256(data.as_slice()));
        let hash2 = SseHash::new(key).map(|x| x.hash256(data.as_slice()));
        hash1 == hash2
    }
}
