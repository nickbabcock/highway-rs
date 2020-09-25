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
    assert!(hash::<highway::AvxHash>().is_ok());
    assert!(hash::<highway::PortableHash>().is_ok());
    assert!(hash::<highway::SseHash>().is_ok());
    assert!(hash::<highway::HighwayHasher>().is_ok());
}
