use super::errors::*;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};
use std::mem::{size_of, MaybeUninit};

#[derive(Debug, Clone)]
pub struct VF4 {
    value: u16,
}

impl VF4 {
    const WALKABLE: u16 = 0x0001;
    const MID: u16 = 0x0002;
    const HIGH: u16 = 0x0004;
    const LOW: u16 = 0x0004 | 0x0002;
    const BLOCKS_VIEW: u16 = 0x0008;
    const RAMP: u16 = 0x0010;

    const BLOCK_SIZE: usize = 16;

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

    pub fn from_buffer(cursor: &mut Cursor<&Vec<u8>>) -> Result<Vec<[VF4; VF4::BLOCK_SIZE]>> {
        let buf_size = cursor.get_ref().len();
        let out_size = buf_size / (VF4::BLOCK_SIZE * size_of::<u16>());
        let mut vf4s = Vec::with_capacity(out_size);

        let mut buf = Vec::with_capacity(out_size);
        buf.resize(out_size, unsafe { MaybeUninit::uninit().assume_init() });
        cursor
            .read_u16_into::<LittleEndian>(&mut buf)
            .chain_err(|| format!("failed to read vf4 bytes",))?;

        for i in 0..out_size {
            let block_number = i / VF4::BLOCK_SIZE;
            let block_index = i % VF4::BLOCK_SIZE;
            if block_number >= vf4s.len() {
                let block: [VF4; VF4::BLOCK_SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
                vf4s.push(block);
            }
            vf4s[block_number][block_index] = VF4 { value: buf[i] }
        }

        return Ok(vf4s);
    }
}
