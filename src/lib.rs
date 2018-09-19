//! This crate is a native Rust port of [Google's
//! HighwayHash](https://github.com/google/highwayhash), which is a fast, keyed, and strong hash
//! function that can take advantage of SIMD instructions (SSE 4.1 and AVX 2) for speed ups that
//! allow it to be faster than traditional cryptographic hash functions and even outpace less secure
//! functions at large payloads. HighwayHash allows for an incremental approach to hashing and can
//! output 64bit, 128bit, and 256bit values.
//! 
//! ## Caution
//! 
//! HighwayHash (the algorithm) has not gone undergone extensive cryptanalysis like SipHash (the default hashing algorithm in Rust), but according to the authors, HighwayHash output bits are uniformly distributed and should withstand differential and rotational attacks. Hence HighwayHash is referred to as a strong hash function, not a cryptographic hash function. I encourage anyone interested to [peruse the paper](https://arxiv.org/abs/1612.06257) to understand the risks.
//! 
//! ## Examples
//! 
//! ```rust
//! use highway::{HighwayBuilder, HighwayHash, Key};
//! 
//! // HighwayHash requires a key that should be hidden from attackers
//! // to ensure outputs are unpredictable, so attackers can't mount
//! // DoS attacks.
//! let key = Key([1, 2, 3, 4]);
//! 
//! // A HighwayBuilder is the recommended approach to hashing, 
//! // as it will select the fastest algorithm available
//! let mut hasher = HighwayBuilder::new(&key);
//! 
//! // Append some data
//! hasher.append(&[255]);
//! 
//! // After all data has been appended, you ask for
//! // 64, 128, or 256bit output. The hasher is consumed
//! // after finalization.
//! let res: u64 = hasher.finalize64();
//! 
//! assert_eq!(0x07858f24d_2d79b2b2, res);
//! ```
//! 
//! Creating a 128bit and 256bit hash is just as simple.
//! 
//! ```rust
//! use highway::{HighwayBuilder, HighwayHash, Key};
//! 
//! // Generate 128bit hash
//! let key = Key([1, 2, 3, 4]);
//! let mut hasher128 = HighwayBuilder::new(&key);
//! hasher128.append(&[255]);
//! let res128: u128 = hasher128.finalize128();
//! assert_eq!(0x224508f9_16b3991f_bb007d24_62e77f3c, res128);
//! 
//! // Generate 256bit hash
//! let key = Key([1, 2, 3, 4]);
//! let mut hasher256 = HighwayBuilder::new(&key);
//! hasher256.append(&[255]);
//! let (res256lo, res256hi): (u128, u128) = hasher256.finalize256();
//! assert_eq!(0xaac4905d_e62b2f5e_7161cadb_f7cd70e1, res256lo);
//! assert_eq!(0xc8efcfc4_5b239f8d_07b02b93_6933faa7, res256hi);
//! ```
//! 
//! ## Use Cases
//! 
//! HighwayHash can be used against untrusted user input where weak hashes can't be used due to exploitation, verified cryptographic hashes are too slow, and a strong hash function meets requirements. Some specific scenarios given by the authors of HighwayHash:
//! 
//! - Use 64bit hashes to for authenticating short lived messages
//! - Use 128 / 256bit hashes are good for checksums (ie: longer lived data, or strong guarantees against collisions)
//!

#![allow(non_snake_case)]
extern crate byteorder;

#[macro_use]
mod macros;
mod builder;
mod internal;
mod key;
mod portable;
mod traits;

pub use builder::HighwayBuilder;
pub use key::Key;
pub use portable::PortableHash;
pub use traits::HighwayHash;

#[cfg(target_arch = "x86_64")]
mod avx;
#[cfg(target_arch = "x86_64")]
mod sse;
#[cfg(target_arch = "x86_64")]
mod v2x64u;
#[cfg(target_arch = "x86_64")]
mod v4x64u;

#[cfg(target_arch = "x86_64")]
pub use avx::AvxHash;
#[cfg(target_arch = "x86_64")]
pub use sse::SseHash;
