#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub fn unordered_load3(from: &[u8]) -> u64 {
    if from.is_empty() {
        return 0;
    }

    let size_mod4 = from.len() % 4;

    u64::from(from[0])
        + (u64::from(from[size_mod4 >> 1]) << 8)
        + (u64::from(from[size_mod4 - 1]) << 16)
}
