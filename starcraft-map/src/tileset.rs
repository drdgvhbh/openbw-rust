use byteorder::{LittleEndian, ReadBytesExt};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use super::errors::*;

const TILESET_MASK: u16 = 0b0111;

/// The tileset of the scenario
#[derive(Debug, Clone, FromPrimitive, Eq, PartialEq)]
pub enum Tileset {
    Badlands = 00,
    SpacePlatform = 01,
    Installation = 02,
    Ashworld = 03,
    Jungle = 04,
    Desert = 05,
    Arctic = 06,
    Twilight = 07,
}

pub fn read_tileset(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<Tileset> {
    cursor
        .read_u16::<LittleEndian>()
        .chain_err(|| {
            format!(
                "failed to read tileset version at position {}",
                cursor.position()
            )
        })
        .map(|tileset| tileset & TILESET_MASK)
        .map(|tileset| match FromPrimitive::from_u16(tileset) {
            Some(Tileset::Badlands) => return Ok(Tileset::Badlands),
            Some(Tileset::SpacePlatform) => return Ok(Tileset::SpacePlatform),
            Some(Tileset::Installation) => return Ok(Tileset::Installation),
            Some(Tileset::Ashworld) => return Ok(Tileset::Ashworld),
            Some(Tileset::Jungle) => return Ok(Tileset::Jungle),
            Some(Tileset::Desert) => return Ok(Tileset::Desert),
            Some(Tileset::Arctic) => return Ok(Tileset::Arctic),
            Some(Tileset::Twilight) => return Ok(Tileset::Twilight),
            None => unreachable!(),
        })?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reads_badlands() {
        let buf: Vec<u8> = (80 as u64).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_tileset(&mut cursor).expect("should read badlands tileset"),
            Tileset::Badlands
        )
    }

    #[test]
    fn test_reads_space_platform() {
        let buf: Vec<u8> = (81 as u64).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_tileset(&mut cursor).expect("should read space platform tileset"),
            Tileset::SpacePlatform
        )
    }

    #[test]
    fn test_reads_installation() {
        let buf: Vec<u8> = (82 as u64).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_tileset(&mut cursor).expect("should read installation tileset"),
            Tileset::Installation
        )
    }

    #[test]
    fn test_reads_ashworld() {
        let buf: Vec<u8> = (83 as u64).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_tileset(&mut cursor).expect("should read ashworld tileset"),
            Tileset::Ashworld
        )
    }

    #[test]
    fn test_reads_jungle() {
        let buf: Vec<u8> = (84 as u64).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_tileset(&mut cursor).expect("should read jungle tileset"),
            Tileset::Jungle
        )
    }

    #[test]
    fn test_reads_desert() {
        let buf: Vec<u8> = (85 as u64).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_tileset(&mut cursor).expect("should read desert tileset"),
            Tileset::Desert
        )
    }

    #[test]
    fn test_reads_arctic() {
        let buf: Vec<u8> = (86 as u64).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_tileset(&mut cursor).expect("should read arctic tileset"),
            Tileset::Arctic
        )
    }

    #[test]
    fn test_reads_twlight() {
        let buf: Vec<u8> = (87 as u64).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_tileset(&mut cursor).expect("should read twlight tileset"),
            Tileset::Twilight
        )
    }

    #[test]
    fn test_fails_reads_non_little_endian_input() {
        let buf: Vec<u8> = vec![0];
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(read_tileset(&mut cursor).is_err(), true)
    }
}
