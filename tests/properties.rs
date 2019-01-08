#[macro_use]
extern crate quickcheck;
extern crate highway;
mod quick_tests {
    use highway::{HighwayBuilder, HighwayHash, Key, PortableHash};

    quickcheck! {
        fn portable64_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
            let key = Key([k1, k2, k3, k4]);
            let hash1 = PortableHash::new(&key).hash64(data.as_slice());
            let hash2 = PortableHash::new(&key).hash64(data.as_slice());
            hash1 == hash2
        }

        fn portable128_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
            let key = Key([k1, k2, k3, k4]);
            let hash1 = PortableHash::new(&key).hash128(data.as_slice());
            let hash2 = PortableHash::new(&key).hash128(data.as_slice());
            hash1 == hash2
        }

        fn portable256_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
            let key = Key([k1, k2, k3, k4]);
            let hash1 = PortableHash::new(&key).hash256(data.as_slice());
            let hash2 = PortableHash::new(&key).hash256(data.as_slice());
            hash1 == hash2
        }

        fn builder64_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
            let key = Key([k1, k2, k3, k4]);
            let hash1 = HighwayBuilder::new(&key).hash64(data.as_slice());
            let hash2 = HighwayBuilder::new(&key).hash64(data.as_slice());
            hash1 == hash2
        }

        fn builder128_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
            let key = Key([k1, k2, k3, k4]);
            let hash1 = HighwayBuilder::new(&key).hash128(data.as_slice());
            let hash2 = HighwayBuilder::new(&key).hash128(data.as_slice());
            hash1 == hash2
        }

        fn builder256_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
            let key = Key([k1, k2, k3, k4]);
            let hash1 = HighwayBuilder::new(&key).hash256(data.as_slice());
            let hash2 = HighwayBuilder::new(&key).hash256(data.as_slice());
            hash1 == hash2
        }

        fn all64_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
            let key = Key([k1, k2, k3, k4]);
            let hash1 = PortableHash::new(&key).hash64(data.as_slice());
            let hash2 = HighwayBuilder::new(&key).hash64(data.as_slice());
            let mut res = hash1 == hash2;

            #[cfg(target_arch = "x86_64")]
            {
                use highway::{AvxHash, SseHash};
                if let Some(h) = AvxHash::new(&key) {
                    res &= h.hash64(data.as_slice()) == hash1;
                }

                if let Some(h) = SseHash::new(&key) {
                    res &= h.hash64(data.as_slice()) == hash1;
                }
            }

            res
        }

        fn all128_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
            let key = Key([k1, k2, k3, k4]);
            let hash1 = PortableHash::new(&key).hash128(data.as_slice());
            let hash2 = HighwayBuilder::new(&key).hash128(data.as_slice());
            let mut res = hash1 == hash2;

            #[cfg(target_arch = "x86_64")]
            {
                use highway::{AvxHash, SseHash};
                if let Some(h) = AvxHash::new(&key) {
                    res &= h.hash128(data.as_slice()) == hash1;
                }

                if let Some(h) = SseHash::new(&key) {
                    res &= h.hash128(data.as_slice()) == hash1;
                }
            }

            res
        }

        fn all256_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
            let key = Key([k1, k2, k3, k4]);
            let hash1 = PortableHash::new(&key).hash256(data.as_slice());
            let hash2 = HighwayBuilder::new(&key).hash256(data.as_slice());
            let mut res = hash1 == hash2;

            #[cfg(target_arch = "x86_64")]
            {
                use highway::{AvxHash, SseHash};
                if let Some(h) = AvxHash::new(&key) {
                    res &= h.hash256(data.as_slice()) == hash1;
                }

                if let Some(h) = SseHash::new(&key) {
                    res &= h.hash256(data.as_slice()) == hash1;
                }
            }

            res
        }
    }
}

#[cfg(target_arch = "x86_64")]
mod quick_simd_tests {
    use highway::{AvxHash, HighwayHash, Key, SseHash};
    quickcheck! {
        fn avx64_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
            let key = Key([k1, k2, k3, k4]);
            let hash1 = AvxHash::new(&key).map(|x| x.hash64(data.as_slice()));
            let hash2 = AvxHash::new(&key).map(|x| x.hash64(data.as_slice()));
            hash1 == hash2
        }

        fn avx128_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
            let key = Key([k1, k2, k3, k4]);
            let hash1 = AvxHash::new(&key).map(|x| x.hash128(data.as_slice()));
            let hash2 = AvxHash::new(&key).map(|x| x.hash128(data.as_slice()));
            hash1 == hash2
        }

        fn avx256_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
            let key = Key([k1, k2, k3, k4]);
            let hash1 = AvxHash::new(&key).map(|x| x.hash256(data.as_slice()));
            let hash2 = AvxHash::new(&key).map(|x| x.hash256(data.as_slice()));
            hash1 == hash2
        }

        fn sse64_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
            let key = Key([k1, k2, k3, k4]);
            let hash1 = SseHash::new(&key).map(|x| x.hash64(data.as_slice()));
            let hash2 = SseHash::new(&key).map(|x| x.hash64(data.as_slice()));
            hash1 == hash2
        }

        fn sse128_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
            let key = Key([k1, k2, k3, k4]);
            let hash1 = SseHash::new(&key).map(|x| x.hash128(data.as_slice()));
            let hash2 = SseHash::new(&key).map(|x| x.hash128(data.as_slice()));
            hash1 == hash2
        }

        fn sse256_eq(k1: u64, k2: u64, k3: u64, k4: u64, data: Vec<u8>) -> bool {
            let key = Key([k1, k2, k3, k4]);
            let hash1 = SseHash::new(&key).map(|x| x.hash256(data.as_slice()));
            let hash2 = SseHash::new(&key).map(|x| x.hash256(data.as_slice()));
            hash1 == hash2
        }
    }
}
