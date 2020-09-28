fn hash<H>() -> std::io::Result<u64>
where
    H: std::hash::Hasher,
    H: std::io::Write,
    H: Default,
{
    let mut reader = "foobar".as_bytes();
    let mut hasher = H::default();
    std::io::copy(&mut reader, &mut hasher)?;
    Ok(std::hash::Hasher::finish(&hasher))
}

#[test]
fn hashers_should_implement_write_and_hasher() {
    if is_x86_feature_detected!("avx2") {
        assert!(hash::<highway::AvxHash>().is_ok());
    }
    if is_x86_feature_detected!("sse4.1") {
        assert!(hash::<highway::SseHash>().is_ok());
    }
    assert!(hash::<highway::PortableHash>().is_ok());
    assert!(hash::<highway::HighwayHasher>().is_ok());
}
