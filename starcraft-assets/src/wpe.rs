use super::errors::*;
use rgb;
use std::io::{Cursor, Read};
use std::mem::MaybeUninit;

/// 256-color RGB Palette.
#[derive(Debug, Clone)]
pub struct WPE(pub [u8; WPEs::BLOCK_SIZE]);

#[derive(Debug, Clone)]
pub struct WPEs(pub Vec<WPE>);

impl WPEs {
    const BLOCK_SIZE: usize = 3;

    pub fn from_buffer(cursor: &mut Cursor<&Vec<u8>>) -> Result<WPEs> {
        let buf_size = cursor.get_ref().len();
        let out_size = buf_size / (WPEs::BLOCK_SIZE + 1);

        let mut colors = Vec::with_capacity(out_size);
        colors.resize(out_size, unsafe { MaybeUninit::uninit().assume_init() });

        let mut out_bytes: [u8; WPEs::BLOCK_SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
        for i in 0..out_size {
            let previous_position = cursor.position();
            cursor
                .read(&mut out_bytes)
                .chain_err(|| format!("failed to read wpe at position: '{}'", previous_position))?;
            cursor.set_position(cursor.position() + 1);
            colors[i] = WPE([out_bytes[0], out_bytes[1], out_bytes[2]])
        }

        return Ok(WPEs(colors));
    }
}
