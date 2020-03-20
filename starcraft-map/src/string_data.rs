use byteorder::{LittleEndian, ReadBytesExt};
use std::io::BufRead;

use super::errors::*;

pub fn read_str_data(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<Vec<Vec<u8>>> {
    let starting_pos = cursor.position();

    let str_count = cursor.read_u16::<LittleEndian>().chain_err(|| {
        format!(
            "failed to read string data count at position {}",
            cursor.position()
        )
    })? as usize;
    let mut offsets = vec![0 as u16; str_count];
    let mut str_data = vec![vec![]; str_count];
    for i in 0..str_count {
        let offset = cursor.read_u16::<LittleEndian>().chain_err(|| {
            format!(
                "failed to read string data {} offset at position {}",
                i,
                cursor.position()
            )
        })?;
        offsets[i] = offset;
    }
    for i in 0..str_count {
        cursor.set_position(starting_pos + offsets[i] as u64);
        let p = cursor.position();
        cursor
            .read_until(0, &mut str_data[i])
            .chain_err(|| format!("failed to read string data {} at position {}", i, p,))?;
    }

    Ok(str_data)
}
