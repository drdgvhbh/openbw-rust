use byteorder::ReadBytesExt;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use super::errors::*;

// The species/race of each player
#[derive(Debug, Clone, FromPrimitive, Eq, PartialEq)]
pub enum Side {
    Zerg = 00,
    Terran = 01,
    Protoss = 02,
    Independent = 03,
    Neutral = 04,
    UserSelectable = 05,
    Random = 06,
    Inactive = 07,
}

pub fn read_size(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<Side> {
    cursor
        .read_u8()
        .chain_err(|| format!("failed to read size at position {}", cursor.position()))
        .map(|side| match FromPrimitive::from_u8(side) {
            Some(Side::Zerg) => return Ok(Side::Zerg),
            Some(Side::Terran) => return Ok(Side::Terran),
            Some(Side::Protoss) => return Ok(Side::Protoss),
            Some(Side::Independent) => return Ok(Side::Independent),
            Some(Side::Neutral) => return Ok(Side::Neutral),
            Some(Side::UserSelectable) => return Ok(Side::UserSelectable),
            Some(Side::Random) => return Ok(Side::Random),
            Some(Side::Inactive) => return Ok(Side::Inactive),
            None => unreachable!(),
        })?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reads_zerg() {
        let buf: Vec<u8> = (Side::Zerg as u8).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_size(&mut cursor).expect("should read zerg side"),
            Side::Zerg
        )
    }

    #[test]
    fn test_reads_terran() {
        let buf: Vec<u8> = (Side::Terran as u8).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_size(&mut cursor).expect("should read zerg side"),
            Side::Terran
        )
    }

    #[test]
    fn test_reads_protoss() {
        let buf: Vec<u8> = (Side::Protoss as u8).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_size(&mut cursor).expect("should read protoss side"),
            Side::Protoss
        )
    }

    #[test]
    fn test_reads_independent() {
        let buf: Vec<u8> = (Side::Independent as u8).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_size(&mut cursor).expect("should read independent side"),
            Side::Independent
        )
    }

    #[test]
    fn test_reads_neutral() {
        let buf: Vec<u8> = (Side::Neutral as u8).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_size(&mut cursor).expect("should read neutral side"),
            Side::Neutral
        )
    }

    #[test]
    fn test_reads_user_selectable() {
        let buf: Vec<u8> = (Side::UserSelectable as u8).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_size(&mut cursor).expect("should read user selectable side"),
            Side::UserSelectable
        )
    }

    #[test]
    fn test_reads_random() {
        let buf: Vec<u8> = (Side::Random as u8).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_size(&mut cursor).expect("should read random side"),
            Side::Random
        )
    }

    #[test]
    fn test_reads_inactive() {
        let buf: Vec<u8> = (Side::Inactive as u8).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_size(&mut cursor).expect("should read inactive side"),
            Side::Inactive
        )
    }
}
