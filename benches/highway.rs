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

    c.bench(
        "64bit",
        ParameterizedBenchmark::new(
            "portable",
            |b, param| {
                let data = vec![0u8; *param];
                let key = Key([0, 0, 0, 0]);
                b.iter(|| PortableHash::hash64(&data, &key))
            },
            parameters.clone(),
        ).with_function("sse", |b, param| {
            let data = vec![0u8; *param];
            let key = Key([0, 0, 0, 0]);
            b.iter(|| SseHash::hash64(&data, &key))
        }).with_function("avx", |b, param| {
            let data = vec![0u8; *param];
            let key = Key([0, 0, 0, 0]);
            b.iter(|| AvxHash::hash64(&data, &key))
        }).with_function("hashmap default", |b, param| {
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
        }).throughput(|s| Throughput::Bytes(*s as u32)),
    );

    c.bench(
        "256bit",
        ParameterizedBenchmark::new(
            "portable",
            |b, param| {
                let data = vec![0u8; *param];
                let key = Key([0, 0, 0, 0]);
                b.iter(|| PortableHash::hash256(&data, &key))
            },
            parameters.clone(),
        ).with_function("sse", |b, param| {
            let data = vec![0u8; *param];
            let key = Key([0, 0, 0, 0]);
            b.iter(|| SseHash::hash256(&data, &key))
        }).with_function("avx", |b, param| {
            let data = vec![0u8; *param];
            let key = Key([0, 0, 0, 0]);
            b.iter(|| AvxHash::hash256(&data, &key))
        }).with_function("sha2", |b, param| {
            let data = vec![0u8; *param];
            b.iter(|| Sha256::digest(&data))
        }).throughput(|s| Throughput::Bytes(*s as u32)),
    );
}

criterion_group!(benches, hashing);
criterion_main!(benches);
