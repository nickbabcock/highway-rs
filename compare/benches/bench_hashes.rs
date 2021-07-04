use blake2b_simd::Params;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
#[cfg(target_arch = "x86_64")]
use highway::{AvxHash, SseHash};
use highway::{HighwayHash, Key, PortableHash};
use sha2::{Digest, Sha256};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

fn bit64_hash(c: &mut Criterion) {
    let parameters = vec![1, 4, 16, 64, 256, 1024, 4096, 16384, 65536];
    let key = Key([0, 0, 0, 0]);

    let mut group = c.benchmark_group("64bit");
    for i in parameters.iter() {
        group.throughput(Throughput::Bytes(*i as u64));
        group.bench_with_input(BenchmarkId::new("portable", i), i, |b, param| {
            let data = vec![0u8; *param];
            let key = Key([0, 0, 0, 0]);
            b.iter(|| PortableHash::new(key).hash64(&data))
        });

        group.bench_with_input(BenchmarkId::new("hashmap default", i), i, |b, param| {
            let data = vec![0u8; *param];
            b.iter(|| {
                let mut hasher = DefaultHasher::new();
                hasher.write(&data);
                hasher.finish()
            })
        });

        group.bench_with_input(BenchmarkId::new("fnv", i), i, |b, param| {
            let data = vec![0u8; *param];
            b.iter(|| {
                let mut hasher = fnv::FnvHasher::with_key(0);
                hasher.write(&data);
                hasher.finish()
            })
        });

        group.bench_with_input(BenchmarkId::new("fx", i), i, |b, param| {
            let data = vec![0u8; *param];
            b.iter(|| {
                let mut hasher = fxhash::FxHasher64::default();
                hasher.write(&data);
                hasher.finish()
            })
        });

        group.bench_with_input(BenchmarkId::new("farmhash", i), i, |b, param| {
            let data = vec![0u8; *param];
            b.iter(|| farmhash::hash64(&data))
        });

        group.bench_with_input(BenchmarkId::new("t1ha", i), i, |b, param| {
            let data = vec![0u8; *param];
            b.iter(|| t1ha::t1ha0(&data, 1234))
        });

        group.bench_with_input(BenchmarkId::new("ahash", i), i, |b, param| {
            use ahash::AHasher;

            let data = vec![0u8; *param];
            b.iter(|| {
                let mut hasher = AHasher::default();
                hasher.write(&data);
                hasher.finish()
            })
        });


        #[cfg(target_arch = "x86_64")]
        {
            if AvxHash::new(key).is_some() {
                group.bench_with_input(BenchmarkId::new("avx", i), i, |b, param| {
                    let data = vec![0u8; *param];
                    let key = Key([0, 0, 0, 0]);
                    b.iter(|| unsafe { AvxHash::force_new(key) }.hash64(&data))
                });
            }

            if SseHash::new(key).is_some() {
                group.bench_with_input(BenchmarkId::new("sse", i), i, |b, param| {
                    let data = vec![0u8; *param];
                    let key = Key([0, 0, 0, 0]);
                    b.iter(|| unsafe { SseHash::force_new(key) }.hash64(&data))
                });
            }
        }
    }
    group.finish();
}

fn bit256_hash(c: &mut Criterion) {
    let parameters = vec![1, 4, 16, 64, 256, 1024, 4096, 16384, 65536];
    let key = Key([0, 0, 0, 0]);

    let mut group = c.benchmark_group("256bit");
    for i in parameters.iter() {
        group.throughput(Throughput::Bytes(*i as u64));
        group.bench_with_input(BenchmarkId::new("portable", i), i, |b, param| {
            let data = vec![0u8; *param];
            let key = Key([0, 0, 0, 0]);
            b.iter(|| PortableHash::new(key).hash256(&data))
        });

        group.bench_with_input(BenchmarkId::new("sha2", i), i, |b, param| {
            let data = vec![0u8; *param];
            b.iter(|| Sha256::digest(&data))
        });

        group.bench_with_input(BenchmarkId::new("blake3", i), i, |b, param| {
            let data = vec![0u8; *param];
            b.iter(|| blake3::hash(&data))
        });

        group.bench_with_input(BenchmarkId::new("blake2b_simd", i), i, |b, param| {
            let data = vec![0u8; *param];
            b.iter(|| {
                Params::new()
                    .hash_length(32)
                    .key(&[1, 2, 3, 4])
                    .to_state()
                    .update(&data)
                    .finalize()
            })
        });

        #[cfg(target_arch = "x86_64")]
        {
            if AvxHash::new(key).is_some() {
                group.bench_with_input(BenchmarkId::new("avx", i), i, |b, param| {
                    let data = vec![0u8; *param];
                    let key = Key([0, 0, 0, 0]);
                    b.iter(|| unsafe { AvxHash::force_new(key) }.hash256(&data))
                });
            }

            if SseHash::new(key).is_some() {
                group.bench_with_input(BenchmarkId::new("sse", i), i, |b, param| {
                    let data = vec![0u8; *param];
                    let key = Key([0, 0, 0, 0]);
                    b.iter(|| unsafe { SseHash::force_new(key) }.hash256(&data))
                });
            }
        }
    }
    group.finish();
}

criterion_group!(benches, bit64_hash, bit256_hash);
criterion_main!(benches);