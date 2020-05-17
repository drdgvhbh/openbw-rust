use super::errors::*;
use rgb;
use std::io::{Cursor, Read};
use std::mem::MaybeUninit;

#[derive(Debug, Clone)]
pub struct WPE(pub rgb::RGB8);

impl WPE {
    const BLOCK_SIZE: usize = 3;

    pub fn from_buffer(cursor: &mut Cursor<&Vec<u8>>) -> Result<Vec<WPE>> {
        let buf_size = cursor.get_ref().len();
        let out_size = buf_size / (WPE::BLOCK_SIZE + 1);
        let mut colors = Vec::with_capacity(out_size);

        let mut buf = Vec::with_capacity(buf_size);
        buf.resize(buf_size, unsafe { MaybeUninit::uninit().assume_init() });
        cursor
            .read(&mut buf)
            .chain_err(|| format!("failed to read wpe bytes",))?;

        let mut alpha_offset = 0;
        for i in 0..out_size {
            colors.push(WPE(rgb::RGB8 {
                r: buf[i * 3 + alpha_offset],
                g: buf[i * 3 + 1 + alpha_offset],
                b: buf[i * 3 + 2 + alpha_offset],
            }));
            alpha_offset += 1;
        }

        return Ok(colors);
    }
}
