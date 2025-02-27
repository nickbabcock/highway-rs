use highway::HighwayHash;

// This is a simple example of how to hash data from stdin using a
// HighwayHasher. Analagous to `shasum` and `md5sum` but using HighwayHash.
//
// ```bash
// cargo run --release --example hwysum < README.md
// ```
fn main() {
    let stdin = std::io::stdin();
    let mut lock = stdin.lock();
    let mut hasher = highway::HighwayHasher::new(highway::Key::default());
    let _ = std::io::copy(&mut lock, &mut hasher);
    let hash = hasher.finalize256();
    println!(
        "{:016x}{:016x}{:016x}{:016x}",
        hash[0], hash[1], hash[2], hash[3]
    );
}
