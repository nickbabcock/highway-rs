#[macro_use]
mod macros;
mod avx;
mod sse;
mod v2x64u;
mod v4x64u;

pub use avx::AvxHash;
pub use sse::SseHash;
