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

pub enum Filled<'a> {
    Consumed,
    Full(&'a [u8]),
}

pub const PACKET_SIZE: usize = 32;

#[derive(Default, Debug)]
pub struct HashPacket {
    buf: [u8; PACKET_SIZE],
    buf_index: usize
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
