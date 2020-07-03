use super::errors::*;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use std::mem::{size_of, MaybeUninit};
use std::ops::{Index, IndexMut};

/// MiniTile graphic references for each MegaTile. Referenced by CV5.
#[derive(Debug, Clone)]
pub struct VF4 {
    value: u16,
}

#[derive(Debug, Clone)]
pub struct VF4s(pub Vec<[VF4; VF4s::BLOCK_SIZE]>);

impl Index<usize> for VF4s {
    type Output = [VF4; VF4s::BLOCK_SIZE];

    fn index(&self, i: usize) -> &Self::Output {
        &self.0[i]
    }
}

impl IndexMut<usize> for VF4s {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.0[i]
    }
}

impl VF4 {
    const WALKABLE: u16 = 0x0001;
    const MID: u16 = 0x0002;
    const HIGH: u16 = 0x0004;
    const LOW: u16 = 0x0004 | 0x0002;
    const BLOCKS_VIEW: u16 = 0x0008;
    const RAMP: u16 = 0x0010;

    pub fn is_walkable(&self) -> bool {
        return self.value & VF4::WALKABLE == VF4::WALKABLE;
    }

    pub fn is_elevation_mid(&self) -> bool {
        return self.value & VF4::MID == VF4::MID;
    }

    pub fn is_elevation_high(&self) -> bool {
        return self.value & VF4::HIGH == VF4::HIGH;
    }

    pub fn is_elevation_low(&self) -> bool {
        return self.value & VF4::LOW == VF4::LOW;
    }

    pub fn blocks_view(&self) -> bool {
        return self.value & VF4::BLOCKS_VIEW == VF4::BLOCKS_VIEW;
    }

    pub fn is_ramp(&self) -> bool {
        return self.value & VF4::RAMP == VF4::RAMP;
    }
}

impl VF4s {
    const BLOCK_SIZE: usize = 16;

    pub fn from_buffer(cursor: &mut Cursor<&Vec<u8>>) -> Result<VF4s> {
        let buf_size = cursor.get_ref().len();
        let out_size = buf_size / (VF4s::BLOCK_SIZE * size_of::<u16>());
        let mut vf4s: Vec<[VF4; VF4s::BLOCK_SIZE]> = Vec::with_capacity(out_size);

        vf4s.resize(out_size, unsafe { MaybeUninit::uninit().assume_init() });
        let mut out_bytes: [u16; VF4s::BLOCK_SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
        for i in 0..out_size {
            let previous_position = cursor.position();
            cursor
                .read_u16_into::<LittleEndian>(&mut out_bytes)
                .chain_err(|| format!("failed to read vf4 at position: '{}'", previous_position))?;
            for j in 0..VF4s::BLOCK_SIZE {
                vf4s[i][j] = VF4 {
                    value: out_bytes[j],
                }
            }
        }

        return Ok(VF4s(vf4s));
    }
}
