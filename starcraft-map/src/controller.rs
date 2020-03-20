use byteorder::ReadBytesExt;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use super::errors::*;

// The owner of the player
#[derive(Debug, Clone, FromPrimitive, Eq, PartialEq)]
pub enum Controller {
    Inactive = 00,
    RescuePassive = 03,
    Unused = 04,
    Computer = 05,
    HumanOpenSlot = 06,
    Neutral = 07,
}

pub fn read_controller(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<Controller> {
    cursor
        .read_u8()
        .chain_err(|| {
            format!(
                "failed to read controller at position {}",
                cursor.position()
            )
        })
        .map(|controller| match FromPrimitive::from_u8(controller) {
            Some(Controller::Inactive) => return Ok(Controller::Inactive),
            Some(Controller::RescuePassive) => return Ok(Controller::RescuePassive),
            Some(Controller::Unused) => return Ok(Controller::Unused),
            Some(Controller::Computer) => return Ok(Controller::Computer),
            Some(Controller::HumanOpenSlot) => return Ok(Controller::HumanOpenSlot),
            Some(Controller::Neutral) => return Ok(Controller::Neutral),
            None => Err(format!("unsupported controller {}", controller).into()),
        })?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reads_inactive() {
        let buf: Vec<u8> = (Controller::Inactive as u8).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_controller(&mut cursor).expect("should read inactive controller"),
            Controller::Inactive
        )
    }

    #[test]
    fn test_reads_rescue_passive() {
        let buf: Vec<u8> = (Controller::RescuePassive as u64).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_controller(&mut cursor).expect("should read rescue passive controller"),
            Controller::RescuePassive
        )
    }

    #[test]
    fn test_reads_unused() {
        let buf: Vec<u8> = (Controller::Unused as u8).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_controller(&mut cursor).expect("should read unused controller"),
            Controller::Unused
        )
    }

    #[test]
    fn test_reads_computer() {
        let buf: Vec<u8> = (Controller::Computer as u8).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_controller(&mut cursor).expect("should read computer controller"),
            Controller::Computer
        )
    }

    #[test]
    fn test_reads_human_open_slot() {
        let buf: Vec<u8> = (Controller::HumanOpenSlot as u8).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_controller(&mut cursor).expect("should read human open slot controller"),
            Controller::HumanOpenSlot
        )
    }

    #[test]
    fn test_reads_neutral() {
        let buf: Vec<u8> = (Controller::Neutral as u8).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_controller(&mut cursor).expect("should read neutral controller"),
            Controller::Neutral
        )
    }

    #[test]
    fn test_fails_computer_game() {
        let buf: Vec<u8> = (1 as u8).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(read_controller(&mut cursor).is_err(), true)
    }

    #[test]
    fn test_fails_human_occupied() {
        let buf: Vec<u8> = (2 as u8).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(read_controller(&mut cursor).is_err(), true)
    }

    #[test]
    fn test_fails_closed_slot() {
        let buf: Vec<u8> = (8 as u8).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(read_controller(&mut cursor).is_err(), true)
    }

    #[test]
    fn test_fails_if_cursor_is_at_end() {
        let buf: Vec<u8> = vec![];
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(read_controller(&mut cursor).is_err(), true)
    }
}
