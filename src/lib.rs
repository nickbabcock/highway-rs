#![allow(non_snake_case)]
extern crate byteorder;

#[macro_use]
mod macros;
mod avx;
mod internal;
mod key;
mod portable;
mod sse;
mod v2x64u;
mod v4x64u;
mod traits;
mod builder;

pub use traits::HighwayHash;
pub use builder::HighwayBuilder;
pub use avx::AvxHash;
pub use key::Key;
pub use portable::PortableHash;
pub use sse::SseHash;
