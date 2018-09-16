#![allow(non_snake_case)]
extern crate byteorder;

#[macro_use]
mod macros;
mod avx;
mod builder;
mod internal;
mod key;
mod portable;
mod sse;
mod traits;
mod v2x64u;
mod v4x64u;

pub use avx::AvxHash;
pub use builder::HighwayBuilder;
pub use key::Key;
pub use portable::PortableHash;
pub use sse::SseHash;
pub use traits::HighwayHash;
