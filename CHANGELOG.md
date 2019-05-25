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
