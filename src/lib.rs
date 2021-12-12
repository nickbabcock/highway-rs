/*!

This crate is a native Rust port of [Google's
HighwayHash](https://github.com/google/highwayhash), which is a fast, keyed,
portable (output is hardware independent) and strong hash function.

## Caution

HighwayHash (the algorithm) has not undergone extensive cryptanalysis like SipHash (the default hashing algorithm in Rust), but according to the authors, HighwayHash output bits are uniformly distributed and should withstand differential and rotational attacks. Hence HighwayHash is referred to as a strong hash function, not a cryptographic hash function. I encourage anyone interested to [peruse the paper](https://arxiv.org/abs/1612.06257) to understand the risks.

## Examples

The quickest way to get started:

```rust
use highway::{HighwayHasher, HighwayHash};
let res: u64 = HighwayHasher::default().hash64(&[]);
let res2: [u64; 2] = HighwayHasher::default().hash128(&[]);
let res3: [u64; 4] = HighwayHasher::default().hash256(&[]);
```

A more complete tour of the API follows:

```rust
use highway::{HighwayHasher, HighwayHash, Key};

// HighwayHash requires a key that should be hidden from attackers
// to ensure outputs are unpredictable, so attackers can't mount
// DoS attacks.
let key = Key([1, 2, 3, 4]);

// A HighwayHasher is the recommended approach to hashing,
// as it will select the fastest algorithm available
let mut hasher = HighwayHasher::new(key);

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
use highway::{HighwayHasher, HighwayHash, Key};

// Generate 128bit hash
let key = Key([1, 2, 3, 4]);
let mut hasher128 = HighwayHasher::new(key);
hasher128.append(&[255]);
let res128: [u64; 2] = hasher128.finalize128();
assert_eq!([0xbb007d2462e77f3c, 0x224508f916b3991f], res128);

// Generate 256bit hash
let key = Key([1, 2, 3, 4]);
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
```

Use highway hash in standard rust collections

```rust
# #[cfg(feature = "std")]
# {
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
# }
```

Or if utilizing a key is not important, one can use the default

```rust
# #[cfg(feature = "std")]
# {
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use highway::HighwayHasher;
let mut map =
  HashMap::with_hasher(BuildHasherDefault::<HighwayHasher>::default());

map.insert(1, 2);
assert_eq!(map.get(&1), Some(&2));
# }
```

Hashing a file, or anything implementing `Read`

```rust
# #[cfg(feature = "std")]
# {
use std::fs::File;
use std::hash::Hasher;
use highway::{PortableHash, HighwayHash};

let mut file = File::open("./README.md").unwrap();
let mut hasher = PortableHash::default();
std::io::copy(&mut file, &mut hasher).unwrap();
let hash64 = hasher.finish(); // core Hasher API
let hash256 = hasher.finalize256(); // HighwayHash API
# }
```

## Wasm SIMD

When deploying HighwayHash to a Wasm environment, one can opt into using the Wasm SIMD instructions by adding a Rust flag:

```bash
RUSTFLAGS="-C target-feature=+simd128" wasm-pack build
```

Then the `WasmHash` struct becomes available.

```rust,ignore
use highway::{HighwayHash, Key, WasmHash};
let key = Key([0, 0, 0, 0]);
let hasher = WasmHash::new(key);
let result = hasher.hash64(&[]);
```

Once opted in, the execution environment must support Wasm SIMD instructions, which Chrome, Firefox, and Node LTS have stabilized since mid-2021. The opt in is required as there is not a way for Wasm to detect SIMD capabilities at runtime.

## Use Cases

HighwayHash can be used against untrusted user input where weak hashes can't be used due to exploitation, verified cryptographic hashes are too slow, and a strong hash function meets requirements. Some specific scenarios given by the authors of HighwayHash:

- Use 64bit hashes to for authenticating short lived messages
- Use 256bit hashes for checksums. Think file storage (S3) or any longer lived data where there is a need for strong guarantees against collisions.

HighwayHash may not be a good fit if the payloads trend small (< 100 bytes) and speed is up of the utmost importance, as HighwayHash hits its stride at larger payloads.

### `no_std` crates

This crate has a feature, `std`, that is enabled by default. To use this crate
in a `no_std` context, add the following to your `Cargo.toml`:

```toml
[dependencies]
highway = { version = "x", default-features = false }
```

Be aware that the `no_std` version is unable to detect CPU features and so will always default to the portable implementation. If building for a known SSE 4.1 or AVX 2 machine (and the majority of machines in the last decade will support SSE 4.1), these hashers can still be constructed with `force_new`.

*/
#![allow(non_snake_case)]
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![warn(missing_docs)]

#[macro_use]
mod macros;
mod builder;
mod hash;
mod internal;
mod key;
mod portable;
mod traits;

pub use crate::builder::HighwayBuilder;
pub use crate::hash::{HighwayBuildHasher, HighwayHasher};
pub use crate::key::Key;
pub use crate::portable::PortableHash;
pub use crate::traits::HighwayHash;

#[cfg(target_arch = "x86_64")]
mod avx;
#[cfg(target_arch = "x86_64")]
mod sse;
#[cfg(target_arch = "x86_64")]
mod v2x64u;
#[cfg(target_arch = "x86_64")]
mod v4x64u;
#[cfg(all(target_family = "wasm", target_feature = "simd128"))]
mod wasm;

#[cfg(target_arch = "x86_64")]
pub use crate::avx::AvxHash;
#[cfg(target_arch = "x86_64")]
pub use crate::sse::SseHash;

#[cfg(all(target_family = "wasm", target_feature = "simd128"))]
pub use crate::wasm::WasmHash;

#[cfg(feature = "std")]
#[cfg(doctest)]
doc_comment::doctest!("../README.md");
