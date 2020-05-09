![ci](https://github.com/nickbabcock/highway-rs/workflows/ci/badge.svg)[![](https://docs.rs/highway/badge.svg)](https://docs.rs/highway) [![Rust](https://img.shields.io/badge/rust-1.36%2B-blue.svg?maxAge=3600)](https://github.com/nickbabcock/highway-rs) [![Version](https://img.shields.io/crates/v/highway.svg?style=flat-square)](https://crates.io/crates/highway)

# Highway-rs

This crate is a native Rust port of [Google's
HighwayHash](https://github.com/google/highwayhash), which is a fast, keyed, and strong hash
function that can take advantage of SIMD instructions (SSE 4.1 and AVX 2) for speed ups that
allow it to be faster than traditional cryptographic hash functions and even outpace less secure
functions at large payloads. HighwayHash allows for an incremental approach to hashing and can
output 64bit, 128bit, and 256bit values.

## Caution

HighwayHash (the algorithm) has not gone undergone extensive cryptanalysis like SipHash (the default hashing algorithm in Rust), but according to the authors, HighwayHash output bits are uniformly distributed and should withstand differential and rotational attacks. Hence HighwayHash is referred to as a strong hash function, not a cryptographic hash function. I encourage anyone interested to [peruse the paper](https://arxiv.org/abs/1612.06257) to understand the risks.

## Examples

```rust
use highway::{HighwayBuilder, HighwayHash, Key};

// HighwayHash requires a key that should be hidden from attackers
// to ensure outputs are unpredictable, so attackers can't mount
// DoS attacks.
let key = Key([1, 2, 3, 4]);

// A HighwayBuilder is the recommended approach to hashing,
// as it will select the fastest algorithm available
let mut hasher = HighwayBuilder::new(key);

// Append some data
hasher.append(&[255]);

// After all data has been appended, you ask for
// 64, 128, or 256bit output. The hasher is consumed
// after finalization.
let res: u64 = hasher.finalize64();

assert_eq!(0x07858f24d_2d79b2b2, res);
```

Creating a 128bit and 256bit hash is just as simple.

```rust
use highway::{HighwayBuilder, HighwayHash, Key};

// Generate 128bit hash
let key = Key([1, 2, 3, 4]);
let mut hasher128 = HighwayBuilder::new(key);
hasher128.append(&[255]);
let res128: [u64; 2] = hasher128.finalize128();
assert_eq!([0xbb007d2462e77f3c, 0x224508f916b3991f], res128);

// Generate 256bit hash
let key = Key([1, 2, 3, 4]);
let mut hasher256 = HighwayBuilder::new(key);
hasher256.append(&[255]);
let res256: [u64; 4] = hasher256.finalize256();
let expected: [u64; 4] = [
    0x7161cadbf7cd70e1,
    0xaac4905de62b2f5e,
    0x7b02b936933faa7,
    0xc8efcfc45b239f8d,
];
assert_eq!(expected, res256);
```

Use highway hash in standard rust collections

```rust
use std::collections::HashMap;
use highway::{HighwayBuildHasher, Key};
let mut map =
  HashMap::with_hasher(HighwayBuildHasher::new(Key([
    0xcbf29ce484222325,
    0xc3a5c85c97cb3127,
    0xb492b66fbe98f273,
    0x9ae16a3b2f90404f,
  ])));

map.insert(1, 2);
assert_eq!(map.get(&1), Some(&2));
```

Or if utilizing a key is not important, one can use the default

```rust
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use highway::HighwayHasher;
let mut map =
  HashMap::with_hasher(BuildHasherDefault::<HighwayHasher>::default());

map.insert(1, 2);
assert_eq!(map.get(&1), Some(&2));
```

## Use Cases

HighwayHash can be used against untrusted user input where weak hashes can't be used due to exploitation, verified cryptographic hashes are too slow, and a strong hash function meets requirements. Some specific scenarios given by the authors of HighwayHash:

- Use 64bit hashes to for authenticating short lived messages
- Use 128 / 256bit hashes are good for checksums (ie: longer lived data, or strong guarantees against collisions)

## Benchmarks

Benchmarks are ran with the following command:

```bash
cargo clean
RUSTFLAGS="-C target-cpu=native" cargo bench
find ./target -wholename "*/new/raw.csv" -print0 | xargs -0 xsv cat rows > assets/highway.csv
```

And can be analyzed with the R script found in the assets directory

Keep in mind, benchmarks will vary by machine. Newer machines typically handle AVX payloads better than older.

We'll first take a look at the throughput when calculating the 64bit hash of a varying payload with various implementations

![64bit-highwayhash.png](assets/64bit-highwayhash.png)

HighwayHash is not meant to be fast for extremely short payloads, as we can see that it falls short of fnv and Farmhash. HighwayHash has a series of rounds executed when the hash value is finally computed that permutes internal state, and the computation occurs at any payload size. This overhead is where the vast majority of time is spent at shorter payloads. At larger payload sizes we see HighwayHash as one of the top leaders. Some may find HighwayHash more desirable than Farmhash due to HighwayHash offering itself as a strong hash function and having a 256bit output.

Now taking a look at calculating a 256 hash value, we see a similar story.

![256bit-highwayhash.png](assets/256bit-highwayhash.png)

HighwayHash is slow and comparable to other functions, but HighwayHash shines at larger payloads.

What should be noted is that there is a performance difference between calculating the 64bit and 256bit HighwayHash due to the 256bit requiring more rounds of permutation. The graph below depicts these differences.

![64bit-vs-256bit-highwayhash.png](assets/64bit-vs-256bit-highwayhash.png)

Up until 1024 bytes, calculating the 64bit hash is twice as fast when using SIMD instructions; however by 16KiB both implementations reach the same steady state across all implementations.

For those more into numbers and are curious about specifics or want more details about the hash functions at small payloads size, here is a table that breaks down (GB/s) at all payload sizes

![highwayhash-table.png](assets/highwayhash-table.png)

### Builder Benchmarks

Have fun running the builder benchmarks to see how performance differs with flags:

*Default compilation*

```bash
cargo bench -- highway-builder
```

*Explicitly disable avx2*

```bash
RUSTFLAGS="-C target-feature=-avx2" cargo bench -- highway-builder
```

*Explicitly disable avx2 when targeting native cpu*

```bash
RUSTFLAGS="-C target-cpu=native -C target-feature=+sse4.1,-avx2" \
  cargo bench -- highway-builder
```
