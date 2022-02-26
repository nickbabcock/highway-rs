use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
#[cfg(target_arch = "x86_64")]
use highway::{AvxHash, SseHash};
use highway::{HighwayBuilder, HighwayHash, Key, PortableHash};

fn builder(c: &mut Criterion) {
    let mut group = c.benchmark_group("highway-builder");

    for size in [1, 4, 16, 64, 256, 1024, 4096, 16384, 65536].iter() {
        group.throughput(Throughput::Bytes(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, size| {
            let data = vec![0u8; *size];
            let key = Key([0, 0, 0, 0]);
            b.iter(|| HighwayBuilder::new(key).hash64(&data));
        });
    }

    group.finish();
}

fn bit64_hash(c: &mut Criterion) {
    let parameters = vec![1, 4, 16, 64, 256, 1024, 4096, 16384, 65536];
    let key = Key([0, 0, 0, 0]);

    let mut group = c.benchmark_group("64bit");
    for i in parameters.iter() {
        group.throughput(Throughput::Bytes(*i as u64));
        group.bench_with_input(BenchmarkId::new("portable", i), i, |b, param| {
            let data = vec![0u8; *param];
            b.iter(|| PortableHash::new(key).hash64(&data))
        });

        #[cfg(target_arch = "x86_64")]
        {
            let key = Key([0, 0, 0, 0]);
            if AvxHash::new(key).is_some() {
                group.bench_with_input(BenchmarkId::new("avx", i), i, |b, param| {
                    let data = vec![0u8; *param];
                    b.iter(|| unsafe { AvxHash::force_new(key) }.hash64(&data))
                });
            }

            if SseHash::new(key).is_some() {
                group.bench_with_input(BenchmarkId::new("sse", i), i, |b, param| {
                    let data = vec![0u8; *param];
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
            b.iter(|| PortableHash::new(key).hash256(&data))
        });

        #[cfg(target_arch = "x86_64")]
        {
            if AvxHash::new(key).is_some() {
                group.bench_with_input(BenchmarkId::new("avx", i), i, |b, param| {
                    let data = vec![0u8; *param];
                    b.iter(|| unsafe { AvxHash::force_new(key) }.hash256(&data))
                });
            }

            if SseHash::new(key).is_some() {
                group.bench_with_input(BenchmarkId::new("sse", i), i, |b, param| {
                    let data = vec![0u8; *param];
                    b.iter(|| unsafe { SseHash::force_new(key) }.hash256(&data))
                });
            }
        }
    }
    group.finish();
}

criterion_group!(benches, builder, bit64_hash, bit256_hash);
criterion_main!(benches);
