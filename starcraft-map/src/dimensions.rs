use byteorder::{LittleEndian, ReadBytesExt};

use super::errors::*;

/// The dimensions of the map
///
/// The Width/Height of the map is measured in the number of square 32x32p tiles.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Dimensions {
    pub width: usize,
    pub height: usize,
}

pub fn read_dimensions(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<Dimensions> {
    let width = cursor
        .read_u16::<LittleEndian>()
        .chain_err(|| format!("failed to read width at position {}", cursor.position()))?;
    let height = cursor
        .read_u16::<LittleEndian>()
        .chain_err(|| format!("failed to read height at position {}", cursor.position()))?;

    Ok(Dimensions {
        width: width as usize,
        height: height as usize,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reads_dimensions() {
        let mut buf: Vec<u8> = (256 as u16).to_le_bytes().to_vec();
        buf.extend((256 as u16).to_le_bytes().to_vec());

        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_dimensions(&mut cursor).expect("should read dimensions"),
            Dimensions {
                width: 256,
                height: 256
            }
        )
    }
}
