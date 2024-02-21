use highway::{HighwayHash, PortableHash};
use std::io::Read;

// Using debug_assertions as a poor man's way to omit no_panic compilation on
// unoptimized builds.
#[cfg_attr(not(debug_assertions), no_panic::no_panic)]
#[inline(never)]
fn hash_data<H: HighwayHash>(mut hasher: H, data: &[u8]) -> u64 {
    hasher.append(data);
    hasher.finalize64()
}

fn main() {
    let stdin = std::io::stdin();
    let mut data = Vec::new();
    stdin.lock().read_to_end(&mut data).unwrap();
    let hasher = PortableHash::default();
    println!("{}", hash_data(hasher, &data));

    #[cfg(target_arch = "x86_64")]
    {
        if let Some(hasher) = highway::AvxHash::new(highway::Key::default()) {
            println!("{}", hash_data(hasher, &data));
        }

        if let Some(hasher) = highway::SseHash::new(highway::Key::default()) {
            println!("{}", hash_data(hasher, &data));
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        let hasher = unsafe { highway::NeonHash::force_new(highway::Key::default()) };
        println!("{}", hash_data(hasher, &data));
    }
}
