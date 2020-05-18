use super::errors::*;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;
use std::mem::MaybeUninit;

/// A List of MegaTile references
#[derive(Debug, Clone)]
pub struct CV5(pub [usize; CV5::MEGA_TILE_REFERENCE_COUNT]);

impl CV5 {
    const MEGA_TILE_REFERENCE_COUNT: usize = 16;
}

pub struct CV5s(pub Vec<CV5>);

impl CV5s {
    const BLOCK_SIZE: usize = 52;

    pub fn from_buffer(cursor: &mut Cursor<&Vec<u8>>) -> Result<CV5s> {
        let buf_size = cursor.get_ref().len();
        let out_size = buf_size / CV5s::BLOCK_SIZE;
        let mut cv5s = Vec::with_capacity(out_size);

        cv5s.resize(out_size, unsafe { MaybeUninit::uninit().assume_init() });
        for i in 0..out_size {
            let previous_position = cursor.position();

            cursor.set_position(cursor.position() + 20);
            let mut megatile_references: [u16; CV5::MEGA_TILE_REFERENCE_COUNT] =
                unsafe { MaybeUninit::uninit().assume_init() };
            cursor
                .read_u16_into::<LittleEndian>(&mut megatile_references)
                .chain_err(|| format!("failed to read cv5 at position: '{}'", previous_position))?;
            cv5s[i] = CV5(unsafe { MaybeUninit::uninit().assume_init() });
            for j in 0..CV5::MEGA_TILE_REFERENCE_COUNT {
                cv5s[i].0[j] = megatile_references[j] as usize;
            }
        }

        return Ok(CV5s(cv5s));
    }
}
