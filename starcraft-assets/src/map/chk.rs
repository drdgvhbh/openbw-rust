use super::super::errors::*;
use byteorder::{LittleEndian, ReadBytesExt};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io::{BufRead, Cursor, Read};

const HEADER_NAME_BYTE_SIZE: usize = 4usize;

pub struct Header {
    name: [u8; HEADER_NAME_BYTE_SIZE],
    pub size: usize,
}

impl Header {
    pub fn new(name: [u8; 4usize], size: usize) -> Result<Header> {
        std::str::from_utf8(&name).chain_err(|| "header name is not utf8")?;

        Ok(Header {
            name: name,
            size: size,
        })
    }

    pub fn from_buffer(cursor: &mut Cursor<&Vec<u8>>) -> Result<Header> {
        let mut name = [0u8; HEADER_NAME_BYTE_SIZE];
        cursor
            .read_exact(&mut name)
            .chain_err(|| "failed to read header name")?;
        let size = cursor
            .read_u32::<LittleEndian>()
            .chain_err(|| "failed to read header size")?;

        Header::new(name, size as usize)
    }

    pub fn name(&self) -> &str {
        unsafe {
            return &std::str::from_utf8_unchecked(&self.name);
        }
    }
}

#[derive(Debug)]
pub enum ChunkName {
    Type,
    Version,
    Tileset,
    Controllers,
    Dimensions,
    Side,
    MegaTileIDs,
    StringData,
}

impl Display for ChunkName {
    fn fmt(&self, f: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.as_str())
    }
}

impl ChunkName {
    pub fn from_str(s: &str) -> Option<ChunkName> {
        match s {
            "TYPE" => Some(ChunkName::Type),
            "VER " => Some(ChunkName::Version),
            "ERA " => Some(ChunkName::Tileset),
            "OWNR" => Some(ChunkName::Controllers),
            "DIM " => Some(ChunkName::Dimensions),
            "SIDE" => Some(ChunkName::Side),
            "MTXM" => Some(ChunkName::MegaTileIDs),
            "STR " => Some(ChunkName::StringData),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ChunkName::Type => "TYPE",
            ChunkName::Version => "VER ",
            ChunkName::Tileset => "ERA ",
            ChunkName::Controllers => "OWNR",
            ChunkName::Dimensions => "DIM ",
            ChunkName::Side => "SIDE",
            ChunkName::MegaTileIDs => "MTXM",
            ChunkName::StringData => "STR ",
        }
    }
}

#[derive(Debug)]
pub enum Chunk {
    ScenarioType(ScenarioType),
    FileFormatVersion(FileFormatVersion),
    Tileset(Tileset),
    Controllers(Vec<Controller>),
    Dimensions(Dimensions),
    Sides(Vec<Side>),
    MegaTileIDs(Vec<MegaTileID>),
    StringData(StringData),
}

impl Chunk {
    pub fn read(header: &Header, cursor: &mut Cursor<&Vec<u8>>) -> Result<Option<Chunk>> {
        let chunk_name = ChunkName::from_str(header.name());
        if chunk_name.is_none() {
            return Ok(None);
        }

        match chunk_name.unwrap() {
            ChunkName::Type => ScenarioType::from_buffer(cursor)
                .map(|scenario_type| Some(Chunk::ScenarioType(scenario_type))),
            ChunkName::Version => {
                if header.size != 2 {
                    return Err(format!("{} size must be 2 bytes", header.name()).into());
                }

                return FileFormatVersion::from_buffer(cursor)
                    .map(|version| Some(Chunk::FileFormatVersion(version)));
            }
            ChunkName::Tileset => {
                if header.size != 2 {
                    return Err(format!("{} size must be 2 bytes", header.name()).into());
                }

                Tileset::from_buffer(cursor).map(|tileset| Some(Chunk::Tileset(tileset)))
            }
            ChunkName::Controllers => {
                if header.size != 12 {
                    return Err(format!("{} size must be 12 bytes", header.name()).into());
                }

                let mut controllers: Vec<Controller> = vec![Controller::Inactive; header.size];
                for i in 0..header.size {
                    controllers[i] = Controller::from_buffer(cursor)?;
                }
                Ok(Some(Chunk::Controllers(controllers)))
            }
            ChunkName::Dimensions => {
                if header.size != 4 {
                    return Err(format!("{} size must be 4 bytes", header.name()).into());
                }

                Dimensions::from_buffer(cursor).map(|dimension| Some(Chunk::Dimensions(dimension)))
            }
            ChunkName::Side => {
                if header.size != 12 {
                    return Err(format!("{} size must be 12 bytes", header.name()).into());
                }

                let mut sides: Vec<Side> = vec![Side::Inactive; header.size];
                for i in 0..(header.size) {
                    sides[i] = Side::from_buffer(cursor)?;
                }

                Ok(Some(Chunk::Sides(sides)))
            }
            ChunkName::MegaTileIDs => {
                if header.size > 0x20000 {
                    return Err(
                        format!("{} size must be less than 0x20000 bytes", header.name()).into(),
                    );
                }

                let tile_count = header.size / 2;
                let mut mega_tiles: Vec<MegaTileID> = Vec::with_capacity(tile_count);
                for _ in 0..tile_count {
                    mega_tiles.push(MegaTileID::from_buffer(cursor)?);
                }

                Ok(Some(Chunk::MegaTileIDs(mega_tiles)))
            }
            ChunkName::StringData => {
                StringData::from_buffer(cursor).map(|str_data| Some(Chunk::StringData(str_data)))
            }
        }
    }
}

#[derive(Debug, FromPrimitive)]
pub enum ScenarioType {
    /// Starcraft
    RAWS = 0x53574152,
    /// Brood War
    RAWB = 0x42574152,
}

impl ScenarioType {
    pub fn from_buffer(cursor: &mut Cursor<&Vec<u8>>) -> Result<ScenarioType> {
        cursor
            .read_u32::<LittleEndian>()
            .chain_err(|| "failed to read scenario type")
            .map(
                |scenario_type| match FromPrimitive::from_u32(scenario_type) {
                    Some(ScenarioType::RAWB) => {
                        return Ok(ScenarioType::RAWB);
                    }
                    Some(ScenarioType::RAWS) => {
                        return Ok(ScenarioType::RAWS);
                    }
                    None => {
                        return Err(format!("unsupported scenario type: {}", scenario_type).into())
                    }
                },
            )?
    }
}

#[derive(Debug, FromPrimitive)]
pub enum FileFormatVersion {
    /// 1.00 Starcraft
    Starcraft = 59,

    /// 1.04 Starcraft and above ("hybrid")
    StarcraftHybrid = 63,

    /// Brood War
    BroodWar = 205,
}

impl FileFormatVersion {
    pub fn from_buffer(cursor: &mut Cursor<&Vec<u8>>) -> Result<FileFormatVersion> {
        cursor
            .read_u16::<LittleEndian>()
            .chain_err(|| "failed to read file format version")
            .map(|ver| match FromPrimitive::from_u16(ver) {
                Some(FileFormatVersion::Starcraft) => {
                    return Ok(FileFormatVersion::Starcraft);
                }
                Some(FileFormatVersion::StarcraftHybrid) => {
                    return Ok(FileFormatVersion::StarcraftHybrid);
                }
                Some(FileFormatVersion::BroodWar) => {
                    return Ok(FileFormatVersion::BroodWar);
                }
                None => return Err(format!("unsupported file format version: {}", ver).into()),
            })?
    }
}

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

impl Tileset {
    const MASK: u16 = 0b0111;
    pub fn from_buffer(cursor: &mut Cursor<&Vec<u8>>) -> Result<Tileset> {
        cursor
            .read_u16::<LittleEndian>()
            .chain_err(|| {
                format!(
                    "failed to read tileset version at position {}",
                    cursor.position()
                )
            })
            .map(|tileset| tileset & Tileset::MASK)
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
}

#[derive(Debug, Clone, FromPrimitive, Eq, PartialEq)]
pub enum Controller {
    Inactive = 00,
    RescuePassive = 03,
    Unused = 04,
    Computer = 05,
    HumanOpenSlot = 06,
    Neutral = 07,
}

impl Controller {
    pub fn from_buffer(cursor: &mut Cursor<&Vec<u8>>) -> Result<Controller> {
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
}

/// The dimensions of the map
///
/// The Width/Height of the map is measured in the number of square 32x32p tiles.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Dimensions {
    pub width: usize,
    pub height: usize,
}

impl Dimensions {
    pub fn from_buffer(cursor: &mut Cursor<&Vec<u8>>) -> Result<Dimensions> {
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
}

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

impl Side {
    pub fn from_buffer(cursor: &mut Cursor<&Vec<u8>>) -> Result<Side> {
        cursor
            .read_u8()
            .chain_err(|| format!("failed to read size at position {}", cursor.position()))
            .map(|side| match FromPrimitive::from_u8(side) {
                Some(Side::Zerg) => Ok(Side::Zerg),
                Some(Side::Terran) => Ok(Side::Terran),
                Some(Side::Protoss) => Ok(Side::Protoss),
                Some(Side::Independent) => Ok(Side::Independent),
                Some(Side::Neutral) => Ok(Side::Neutral),
                Some(Side::UserSelectable) => Ok(Side::UserSelectable),
                Some(Side::Random) => Ok(Side::Random),
                Some(Side::Inactive) => Ok(Side::Inactive),
                None => unreachable!(),
            })?
    }
}

#[derive(Debug, Clone)]
pub struct MegaTileID {
    raw_value: u16,
}

impl MegaTileID {
    pub fn from_buffer(cursor: &mut Cursor<&Vec<u8>>) -> Result<MegaTileID> {
        Ok(MegaTileID {
            raw_value: cursor.read_u16::<LittleEndian>().chain_err(|| {
                format!("failed to read megatile at position {}", cursor.position())
            })?,
        })
    }

    pub fn group_index(&self) -> usize {
        return ((self.raw_value >> 4) & 0x7ff) as usize;
    }

    pub fn subtile_index(&self) -> usize {
        return (self.raw_value & 0xf) as usize;
    }
}

#[derive(Debug, Clone)]
pub struct StringData(Vec<Vec<u8>>);

impl StringData {
    pub fn from_buffer(cursor: &mut Cursor<&Vec<u8>>) -> Result<StringData> {
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

        Ok(StringData(str_data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_does_not_accept_non_utf8_characters() {
        assert_eq!(Header::new([221, 248, 206, 155], 30).is_err(), true);

        let buf = [
            &vec![221, 248, 206, 155][..],
            &(204 as u64).to_le_bytes().to_vec()[..],
        ]
        .concat();
        let mut cursor = Cursor::new(&buf);

        assert_eq!(Header::from_buffer(&mut cursor).is_err(), true);
    }
}
