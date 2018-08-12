#![allow(unused)]

extern crate byteorder;

use byteorder::{ByteOrder, LE};
use std::ops::Index;

pub struct Key([u64; 4]); 

#[derive(Default)]
struct PortableState {
    v0: [u64; 4],
    v1: [u64; 4],
    mul0: [u64; 4],
    mul1: [u64; 4],
}

impl Index<usize> for Key {
	type Output = u64;
    fn index(&self, index: usize) -> &u64 {
		&self.0[index]
    }
}

impl PortableState {
    fn reset(&mut self, key: &Key) {
		  self.mul0[0] = 0xdbe6d5d5fe4cce2f;
		  self.mul0[1] = 0xa4093822299f31d0;
		  self.mul0[2] = 0x13198a2e03707344;
		  self.mul0[3] = 0x243f6a8885a308d3;
		  self.mul1[0] = 0x3bd39e10cb0ef593;
		  self.mul1[1] = 0xc0acf169b5f18a8c;
		  self.mul1[2] = 0xbe5466cf34e90c6c;
		  self.mul1[3] = 0x452821e638d01377;
		  self.v0[0] = self.mul0[0] ^ key[0];
		  self.v0[1] = self.mul0[1] ^ key[1];
		  self.v0[2] = self.mul0[2] ^ key[2];
		  self.v0[3] = self.mul0[3] ^ key[3];
		  self.v1[0] = self.mul1[0] ^ ((key[0] >> 32) | (key[0] << 32));
		  self.v1[1] = self.mul1[1] ^ ((key[1] >> 32) | (key[1] << 32));
		  self.v1[2] = self.mul1[2] ^ ((key[2] >> 32) | (key[2] << 32));
		  self.v1[3] = self.mul1[3] ^ ((key[3] >> 32) | (key[3] << 32));
    }
}

pub struct PortableHash;

impl PortableHash {
    pub fn hash64(data: &[u8], key: &Key) -> u64 {
		let mut state = PortableState::default();
		 PortableHash::process_all(data, key, &mut state);
		PortableHash::finalize64(&mut state)
    }

    pub fn hash128(data: &[u8], key: &Key) -> u128 {
        0
    }

    pub fn hash256(data: &[u8], key: &Key) -> (u128, u128) {
        (0, 0)
    }

    fn finalize64(state: &mut PortableState) -> u64 {
		for i in 0..4 {
		PortableHash::permute_and_update(state);
	  }
	  state.v0[0] + state.v1[0] + state.mul0[0] + state.mul1[0]
    }

fn permute(v: &[u64; 4]) -> [u64; 4] {
	[(v[2] >> 32) | (v[2] << 32),
		(v[3] >> 32) | (v[3] << 32),
		(v[0] >> 32) | (v[0] << 32),
		(v[1] >> 32) | (v[1] << 32)
	]
}

 fn permute_and_update(state: &mut PortableState) {
  let permuted: [u64; 4] =  PortableHash::permute(&state.v0);
 PortableHash::update(permuted, state)
}

    fn update(lanes: [u64; 4], state: &mut PortableState) {
		for i in 0..4 {
			state.v1[i] += state.mul0[i] + lanes[i];
			state.mul0[i] ^= (state.v1[i] & 0xffffffff) * (state.v0[i] >> 32);
			state.v0[i] += state.mul1[i];
			state.mul1[i] ^= (state.v0[i] & 0xffffffff) * (state.v1[i] >> 32);
        }

      PortableHash::zipper_merge_and_add(state.v1[1], state.v1[0], &mut state.v0, 1, 0);
      PortableHash::zipper_merge_and_add(state.v1[3], state.v1[2], &mut state.v0, 3, 2);
      PortableHash::zipper_merge_and_add(state.v0[1], state.v0[0], &mut state.v1, 1, 0);
      PortableHash::zipper_merge_and_add(state.v0[3], state.v0[2], &mut state.v1, 3, 2);
    }

	fn zipper_merge_and_add(v1: u64, v0: u64, lane: &mut [u64; 4], add1: usize, add0: usize) {
	  lane[add0] += (((v0 & 0xff000000) | (v1 & 0xff00000000)) >> 24) |
			   (((v0 & 0xff0000000000) | (v1 & 0xff000000000000)) >> 16) |
			   (v0 & 0xff0000) | ((v0 & 0xff00) << 32) |
			   ((v1 & 0xff00000000000000) >> 8) | (v0 << 56);
	  lane[add1] += (((v1 & 0xff000000) | (v0 & 0xff00000000)) >> 24) |
			   (v1 & 0xff0000) | ((v1 & 0xff0000000000) >> 16) |
			   ((v1 & 0xff00) << 24) | ((v0 & 0xff000000000000) >> 8) |
			   ((v1 & 0xff) << 48) | (v0 & 0xff00000000000000);
	}

    fn update_packet(packet: &[u8], state: &mut PortableState) {
        let lanes: [u64; 4] = [
            LE::read_u64(&packet[0..8]),
            LE::read_u64(&packet[8..16]),
            LE::read_u64(&packet[16..24]),
            LE::read_u64(&packet[24..32]),
        ];
        PortableHash::update(lanes, state);
    }


	fn process_all(data: &[u8], key: &Key, state: &mut PortableState) {
        state.reset(key);
        let mut slice = &data[..];
        while slice.len() >= 32 {
            PortableHash::update_packet(&slice, state);
            slice = &slice[32..];
        }

		if (slice.len() > 0) {
			PortableHash::update_remainder(&slice, state);
		}
    }

fn rotate_32_by(count: u64, lanes: &mut [u64; 4]) {
for i in 0..4 {
    let half0: u32 = lanes[i] as u32;
    let half1: u32 = (lanes[i] >> 32) as u32;
    lanes[i] = u64::from((half0 << count) | (half0 >> (32 - count)));
    lanes[i] |= u64::from(((half1 << count) | (half1 >> (32 - count)))) << 32;
  }
}

   fn update_remainder(bytes: &[u8], state: &mut PortableState) {
  let size_mod4 = bytes.len() & 3;
  let remainder = &bytes[bytes.len() & !3..];
  let size = bytes.len() as u64;
  let mut packet: [u8; 32] = Default::default();

  for i in 0..4 {
    state.v0[i] += (size << 32) + size;
  }

  PortableHash::rotate_32_by(size, &mut state.v1);
	for i in 0..remainder.len() - bytes.len() {
    packet[i] = bytes[i];
  }
  if (size & 16 != 0) {
for i in 0..4 {
      packet[28 + i] = remainder[i + size_mod4 - 4];
    }
  } else {
    if (size_mod4 != 0) {
      packet[16 + 0] = remainder[0];
      packet[16 + 1] = remainder[size_mod4 >> 1];
      packet[16 + 2] = remainder[size_mod4 - 1];
    }
  }
  PortableHash::update_packet(&mut packet, state);
} 
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
