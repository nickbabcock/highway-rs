#[macro_use]
extern crate criterion;
extern crate highway;

use criterion::{Criterion, ParameterizedBenchmark, Throughput};
use highway::{Key, PortableHash, SseHash};

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
            parameters,
        ).with_function("sse", |b, param| {
            let data = vec![0u8; *param];
            let key = Key([0, 0, 0, 0]);
            b.iter(|| SseHash::hash64(&data, &key))
        })
        .throughput(|s| Throughput::Bytes(*s as u32)),
    );
}

criterion_group!(benches, hashing);
criterion_main!(benches);
