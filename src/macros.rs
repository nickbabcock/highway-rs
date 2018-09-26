#[cfg(target_arch = "x86_64")]
/// The function, [_MM_SHUFFLE](https://doc.rust-lang.org/core/arch/x86_64/fn._MM_SHUFFLE.html) is
/// only supported on nightly and there has been [some controversy
/// around](https://github.com/rust-lang-nursery/stdsimd/issues/522) it regarding the type
/// signature, so the safe route here is to just go with our own macro.
macro_rules! _mm_shuffle {
    ($z:expr, $y:expr, $x:expr, $w:expr) => {
        ($z << 6) | ($y << 4) | ($x << 2) | $w
    };
}
