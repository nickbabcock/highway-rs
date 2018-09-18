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
