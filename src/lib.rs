/*!

This crate is a native Rust port of [Google's HighwayHash](https://github.com/google/highwayhash),
which is a fast, keyed, and strong hash function.

## Caution

HighwayHash (the algorithm) has not gone undergone extensive cryptanalysis like SipHash (the default hashing algorithm in Rust), but according to the authors, HighwayHash output bits are uniformly distributed and should withstand differential and rotational attacks. Hence HighwayHash is referred to as a strong hash function, not a cryptographic hash function. I encourage anyone interested to [peruse the paper](https://arxiv.org/abs/1612.06257) to understand the risks.

## Examples

The quickest way to get started:

```rust
use highway::{HighwayBuilder, HighwayHash};
let res: u64 = HighwayBuilder::default().hash64(&[]);
let res2: [u64; 2] = HighwayBuilder::default().hash128(&[]);
let res3: [u64; 4] = HighwayBuilder::default().hash256(&[]);
```

A more complete tour of the API follows:

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

Hashing a file, or anything implementing `Read`

```rust
use std::fs::File;
use std::hash::Hasher;
use highway::{PortableHash, HighwayHash};

let mut file = File::open("./README.md").unwrap();
let mut hasher = PortableHash::default();
std::io::copy(&mut file, &mut hasher).unwrap();
let hash64 = hasher.finish(); // core Hasher API
let hash256 = hasher.finalize256(); // HighwayHash API
```

*/
#![allow(non_snake_case)]

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

#[cfg(target_arch = "x86_64")]
pub use crate::avx::AvxHash;
#[cfg(target_arch = "x86_64")]
pub use crate::sse::SseHash;

#[cfg(doctest)]
doc_comment::doctest!("../README.md");
