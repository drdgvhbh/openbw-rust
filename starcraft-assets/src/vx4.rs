use super::errors::*;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use std::mem::MaybeUninit;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone)]
pub struct VX4 {
    value: u16,
}

impl VX4 {
    pub fn is_horizontally_flipped(&self) -> bool {
        return self.value & 1 == 1;
    }

    pub fn index(&self) -> usize {
        return (self.value >> 1) as usize;
    }
}

#[derive(Debug, Clone)]
pub struct VX4s(pub Vec<[VX4; VX4s::BLOCK_SIZE]>);

impl Index<usize> for VX4s {
    type Output = [VX4; VX4s::BLOCK_SIZE];

    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

impl IndexMut<usize> for VX4s {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}

impl VX4s {
    const BLOCK_SIZE: usize = 16;

    pub fn from_buffer(cursor: &mut Cursor<&Vec<u8>>) -> Result<VX4s> {
        let buf_size = cursor.get_ref().len();
        let out_size = buf_size / (VX4s::BLOCK_SIZE * std::mem::size_of::<u16>());

        let mut vx4s: Vec<[VX4; VX4s::BLOCK_SIZE]> = Vec::with_capacity(out_size);
        vx4s.resize(out_size, unsafe { MaybeUninit::uninit().assume_init() });

        let mut out_bytes: [u16; VX4s::BLOCK_SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
        for i in 0..out_size {
            let previous_position = cursor.position();
            cursor
                .read_u16_into::<LittleEndian>(&mut out_bytes)
                .chain_err(|| {
                    format!("failed to read vx4s at position: '{}'", previous_position)
                })?;
            for j in 0..VX4s::BLOCK_SIZE {
                vx4s[i][j] = VX4 {
                    value: out_bytes[j],
                };
            }
        }

        Ok(VX4s(vx4s))
    }
}
