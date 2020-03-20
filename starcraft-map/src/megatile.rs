use byteorder::{LittleEndian, ReadBytesExt};

use super::errors::*;

#[derive(Debug, Clone)]
pub struct MegaTileID {
    raw_value: u16,
}

impl MegaTileID {
    pub fn group_index(&self) -> usize {
        return ((self.raw_value >> 4) & 0x7ff) as usize;
    }

    pub fn subtile_index(&self) -> usize {
        return (self.raw_value & 0xf) as usize;
    }
}

pub fn read_megatile_id(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<MegaTileID> {
    Ok(MegaTileID {
        raw_value: cursor
            .read_u16::<LittleEndian>()
            .chain_err(|| format!("failed to read megatile at position {}", cursor.position()))?,
    })
}
