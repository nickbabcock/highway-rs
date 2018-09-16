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

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod avx;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod sse;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod v2x64u;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod v4x64u;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use avx::AvxHash;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use sse::SseHash;
