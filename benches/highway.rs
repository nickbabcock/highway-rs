#[macro_use]
extern crate criterion;
extern crate farmhash;
extern crate fnv;
extern crate highway;
extern crate sha2;

use criterion::{Criterion, ParameterizedBenchmark, Throughput};
use highway::{AvxHash, HighwayHash, Key, PortableHash, SseHash};
use sha2::{Digest, Sha256};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

fn hashing(c: &mut Criterion) {
    let parameters = vec![1, 4, 16, 64, 256, 1024, 4096, 16384, 65536];
    let key = Key([0, 0, 0, 0]);

    let mut bit64 = ParameterizedBenchmark::new(
        "portable",
        |b, param| {
            let data = vec![0u8; *param];
            let key = Key([0, 0, 0, 0]);
            b.iter(|| PortableHash::new(&key).hash64(&data))
        },
        parameters.clone(),
    ).with_function("hashmap default", |b, param| {
        let data = vec![0u8; *param];
        b.iter(|| {
            let mut hasher = DefaultHasher::new();
            hasher.write(&data);
            hasher.finish()
        })
    }).with_function("fnv", |b, param| {
        let data = vec![0u8; *param];
        b.iter(|| {
            let mut hasher = fnv::FnvHasher::with_key(0);
            hasher.write(&data);
            hasher.finish()
        })
    }).with_function("farmhash", |b, param| {
        let data = vec![0u8; *param];
        b.iter(|| farmhash::hash64(&data))
    }).throughput(|s| Throughput::Bytes(*s as u32));

    if AvxHash::new(&key).is_some() {
        bit64 = bit64.with_function("avx", |b, param| {
            let data = vec![0u8; *param];
            let key = Key([0, 0, 0, 0]);
            b.iter(|| unsafe { AvxHash::force_new(&key) }.hash64(&data))
        });
    }

    if SseHash::new(&key).is_some() {
        bit64 = bit64.with_function("sse", |b, param| {
            let data = vec![0u8; *param];
            let key = Key([0, 0, 0, 0]);
            b.iter(|| unsafe { SseHash::force_new(&key) }.hash64(&data))
        });
    }

    c.bench("64bit", bit64);

    let mut bit256 = ParameterizedBenchmark::new(
        "portable",
        |b, param| {
            let data = vec![0u8; *param];
            let key = Key([0, 0, 0, 0]);
            b.iter(|| PortableHash::new(&key).hash256(&data))
        },
        parameters.clone(),
    ).with_function("sha2", |b, param| {
        let data = vec![0u8; *param];
        b.iter(|| Sha256::digest(&data))
    }).throughput(|s| Throughput::Bytes(*s as u32));

    if AvxHash::new(&key).is_some() {
        bit256 = bit256.with_function("avx", |b, param| {
            let data = vec![0u8; *param];
            let key = Key([0, 0, 0, 0]);
            b.iter(|| unsafe { AvxHash::force_new(&key) }.hash256(&data))
        });
    }

    if SseHash::new(&key).is_some() {
        bit256 = bit256.with_function("sse", |b, param| {
            let data = vec![0u8; *param];
            let key = Key([0, 0, 0, 0]);
            b.iter(|| unsafe { SseHash::force_new(&key) }.hash256(&data))
        });
    }

    c.bench("256bit", bit256);
}

criterion_group!(benches, hashing);
criterion_main!(benches);
