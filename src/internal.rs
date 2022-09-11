#[cfg(any(
    target_arch = "x86_64",
    target_arch = "aarch64",
    all(target_family = "wasm", target_feature = "simd128")
))]
pub fn unordered_load3(from: &[u8]) -> u64 {
    if from.is_empty() {
        return 0;
    }

    let size_mod4 = from.len() % 4;

    u64::from(from[0])
        + (u64::from(from[size_mod4 >> 1]) << 8)
        + (u64::from(from[size_mod4 - 1]) << 16)
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Filled<'a> {
    Consumed,
    Full(&'a [u8]),
}

pub const PACKET_SIZE: usize = 32;

/// The c layout is needed as we'll be interpretting the buffer as different types and passing it
/// to simd instructions, so we need to subscribe to the whole "do what C does", else we will
/// segfault.
#[repr(C)]
#[derive(Default, Debug, Clone, Copy)]
pub struct HashPacket {
    buf: [u8; PACKET_SIZE],
    buf_index: usize,
}

impl HashPacket {
    pub fn len(&self) -> usize {
        self.buf_index
    }

    pub fn is_empty(&self) -> bool {
        self.buf_index == 0
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buf[..self.buf_index]
    }

    pub fn fill<'a>(&mut self, data: &'a [u8]) -> Filled<'a> {
        if data.len() < PACKET_SIZE - self.buf_index {
            let new_ind = self.buf_index + data.len();
            self.buf[self.buf_index..new_ind].copy_from_slice(data);
            self.buf_index = new_ind;
            Filled::Consumed
        } else {
            let (begin, end) = data.split_at(PACKET_SIZE - self.buf_index);
            self.buf[self.buf_index..].copy_from_slice(begin);
            self.buf_index = PACKET_SIZE;
            Filled::Full(end)
        }
    }

    pub fn set_to(&mut self, data: &[u8]) {
        self.buf_index = data.len();
        self.buf[..data.len()].copy_from_slice(data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_packet() {
        let mut packet: HashPacket = Default::default();
        for i in 0..31 {
            assert_eq!(&vec![0; i as usize][..], packet.as_slice());
            if let Filled::Full(_) = packet.fill(&[0]) {
                assert!(false);
            }

            assert_eq!(i + 1, packet.len() as u8);
            assert_eq!(&vec![0; (i + 1) as usize][..], packet.as_slice());
        }
    }

    #[test]
    fn test_hash_cusp_full_packet() {
        let mut packet: HashPacket = Default::default();
        assert_eq!(Filled::Full(&[]), packet.fill(&[0; 32]));
        assert_eq!(32, packet.len());
    }

    #[test]
    fn test_hash_packet_set_to() {
        let mut packet: HashPacket = Default::default();
        for i in 0..31 {
            let d = vec![0; i as usize];
            packet.set_to(&d[..]);
            assert_eq!(&d[..], packet.as_slice());
            assert_eq!(d.len(), packet.len());
        }
    }
}
