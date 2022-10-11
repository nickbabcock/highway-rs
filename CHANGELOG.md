## v0.8.1 - 2022-10-11

- Annotate hashing constructors with `#[must_use]`
- A small performance increase, mainly for the portable implementation (other hashers may benefited as well), by eliminating all emitted panics
- Minor pedantic clippy lints applied

## v0.8.0 - 2022-02-28

- The `HighwayBuilder` type has been removed in favor of the former alias `HighwayHasher`
- Add Neon SIMD implementation for aarch64 targets which enabled throughput improvements of over 4x. The downside with this implementation is that all aarch64 environments are assumed to support NEON SIMD. Thus, aarch64 environments without NEON SIMD are not supported.
- Minimum supported rust version updated to 1.59 for aarch64 targets

## v0.7.0 - 2021-12-12

- Update minimum supported rust to 1.54
- Add Wasm SIMD implementation for 3x performance gain. See readme for caveats and how to opt-in
- `no_std` builds will use a SIMD implementation when opted in at compile time

## v0.6.4 - 2021-04-16

Allow for forwards compatibility with later rust compilers due to changes in some AVX2 usage

## v0.6.3 - 2020-12-04

Extremely minor update that removes the last vestiges of `unsafe` from the portable implementation -- without sacrificing performance. No changes in behavior.

## v0.6.2 - 2020-11-19

Fix hash calculation on big endian platforms. This regression was introduced in v0.5.0 and all users are advised to upgrade.

## v0.6.1 - 2020-11-08

No code changes -- just some docs updates and this crate is now tagged with the `hasher` keyword.

## v0.6.0 - 2020-10-25

- `no_std` compatible when default cargo features are disabled. To get SIMD implementations, one will need to call the `force_new` constructors explicitly.
- All highway hash implementations now implement the `Hasher` and `Write` trait
- Make `HighwayHasher` an alias to `HighwayBuilder` and recommend the use of `HighwayHasher` in documentation

## v0.5.0 - 2020-06-24

- 60% throughput increase to the portable highway hash implementation

## v0.4.0 - 2020-05-09

- Highway-rs is now dependency free!
- Use rust 2018 edition
- **Breaking change**: hashes take an owned `Key` instead of a reference. This is not only better API design, but it could also lead to marginal better performance as the hash implementations would clone the reference regardless. Unfortunately, it is a breaking change, but one should only need to remove an ampersand.

## v0.3.0 - 2019-08-08

Allow the use of highway hash in standard rust collections

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

- Added clone implementations to many structures
- impl Default for HighwayBuilder

## v0.2.0 - 2019-05-25

- Change 128bit hash return type from u128 to [u64; 2] to match the return type from the reference implementation
- Change 256bit hash return type from (u128, u128) to [u64; 4] to match the return type from the reference implementation

You can use the following code to migrate the current return types to the old ones.

```rust
fn u64_to_u128(data: &[u64]) -> u128 {
    u128::from(data[0]) + (u128::from(data[1]) << 64)
}

fn u64_to_u256(data: &[u64]) -> (u128, u128) {
    (u64_to_u128(data), u64_to_u128(&data[2..]))
}
```

## v0.1.4 - 2018-10-01

- Fix: debug arithmetic overflow panic in portable hash

## v0.1.3 - 2018-09-30

- Remove `SseHash::finalize64` as part of public API (accidentally included)

## v0.1.2 - 2018-09-23

- Fix: AVX enabled hash could segfault on unaligned loads of user input.

## v0.1.1 - 2018-09-20

- Fix: SIMD enabled hash functions would return the improper response when not compiled with either an explicit `target-cpu=native` or if `target-feature=+avx2` was omitted

## v0.1.0 - 2018-09-19

- Initial Release
