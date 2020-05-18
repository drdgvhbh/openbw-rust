use super::errors::*;
use std::io::{Cursor, Read};
use std::mem::MaybeUninit;

pub type VR4 = [usize; VR4s::BLOCK_SIZE];

#[derive(Clone)]
pub struct VR4s(pub Vec<VR4>);

impl VR4s {
    pub const BLOCK_SIZE: usize = 64;

    pub fn from_buffer(cursor: &mut Cursor<&Vec<u8>>) -> Result<VR4s> {
        let buf_size = cursor.get_ref().len();
        let out_size = buf_size / VR4s::BLOCK_SIZE;

        let mut vr4s: Vec<VR4> = Vec::with_capacity(out_size);
        vr4s.resize(out_size, unsafe { MaybeUninit::uninit().assume_init() });

        let mut out_bytes: [u8; VR4s::BLOCK_SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
        for i in 0..out_size {
            let previous_position = cursor.position();
            cursor.read(&mut out_bytes).chain_err(|| {
                format!("failed to read vr4s at position: '{}'", previous_position)
            })?;

            for j in 0..VR4s::BLOCK_SIZE {
                vr4s[i][j] = out_bytes[j] as usize;
            }
        }

        return Ok(VR4s(vr4s));
    }
}
