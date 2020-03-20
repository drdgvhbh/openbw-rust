#[macro_use]
extern crate error_chain;

use byteorder::{LittleEndian, ReadBytesExt};
use ceres_mpq;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::io::Read;

mod errors {
    error_chain! {}
}
use errors::*;

const MAP_FILE_NAME: &str = "staredit\\scenario.chk";
const HEADER_NAME_BYTE_SIZE: usize = 4usize;
const NAME_SCENARIO_TYPE: &str = "TYPE";
const NAME_FILE_FORMAT_VERSION: &str = "VER ";
const NAME_TILESET: &str = "ERA ";
const NAME_CONTROLLERS: &str = "OWNR";
const NAME_DIMENSIONS: &str = "DIM ";
const NAME_SIDE: &str = "SIDE";
const NAME_MEGATILE: &str = "MTXM";
const NAME_STRING_DATA: &str = "STR ";

mod controller;
mod dimensions;
mod megatile;
mod string_data;
mod tileset;

use controller::read_controller;
pub use controller::Controller;
use dimensions::read_dimensions;
pub use dimensions::Dimensions;
use tileset::read_tileset;
pub use tileset::Tileset;
mod side;
use megatile::read_megatile_id;
pub use megatile::MegaTileID;
use side::read_size;
pub use side::Side;
use string_data::read_str_data;

/// The type of scenario
#[derive(Debug, FromPrimitive)]
pub enum ScenarioType {
    /// Starcraft
    RAWS = 0x53574152,
    /// Brood War
    RAWB = 0x42574152,
}

/// The file format version
#[derive(Debug, FromPrimitive)]
pub enum FileFormatVersion {
    /// 1.00 Starcraft
    Starcraft = 59,

    /// 1.04 Starcraft and above ("hybrid")
    StarcraftHybrid = 63,

    /// Brood War
    BroodWar = 205,
}

#[derive(Debug)]
struct DeserializedMap {
    scenario_type: Option<ScenarioType>,
    file_format_version: Option<FileFormatVersion>,
    tileset: Option<Tileset>,
    controllers: Option<Vec<Controller>>,
    dimensions: Option<Dimensions>,
    side: Option<Vec<Side>>,
    megatiles: Option<Vec<MegaTileID>>,
    str_data: Option<Vec<Vec<u8>>>,
}

impl DeserializedMap {
    fn new() -> DeserializedMap {
        DeserializedMap {
            scenario_type: None,
            file_format_version: None,
            controllers: None,
            tileset: None,
            dimensions: None,
            side: None,
            megatiles: None,
            str_data: None,
        }
    }
}

#[derive(Debug)]
pub struct Map {
    pub scenario_type: Option<ScenarioType>,
    pub file_format_version: FileFormatVersion,
    pub controllers: Vec<Controller>,
    pub tileset: Tileset,
    pub dimensions: Dimensions,
    pub side: Vec<Side>,
    pub megatiles: Vec<MegaTileID>,
    pub str_data: Vec<Vec<u8>>,
}

impl Map {
    pub fn from_mpq_file(file_name: &str) -> Result<Map> {
        let mut mpq_file = std::fs::File::open(file_name)
            .chain_err(|| format!("failed to open mpq file {}", file_name))?;

        let mut buf: Vec<u8> = Vec::new();
        mpq_file
            .read_to_end(&mut buf)
            .chain_err(|| format!("failed to read mpq file {}", file_name))?;

        let mut cursor = std::io::Cursor::new(buf);
        let mut archive =
            ceres_mpq::Archive::open(&mut cursor).chain_err(|| "failed to open mpq archive")?;
        let files = archive
            .files()
            .ok_or_else(|| "Starcraft maps must have a file inside")?;

        if files
            .into_iter()
            .filter(|f| f == MAP_FILE_NAME)
            .collect::<Vec<_>>()
            .len()
            == 0
        {
            let err_msg = format!("Starcraft maps must contain a {} file", MAP_FILE_NAME);
            return Err(err_msg.into());
        }
        let chunks = archive
            .read_file(&MAP_FILE_NAME)
            .chain_err(|| format!("failed to read archive file {}", MAP_FILE_NAME))?;

        Map::from_chunks(&chunks)
    }

    fn from_chunks(chunks: &Vec<u8>) -> Result<Map> {
        let mut map = DeserializedMap::new();
        let mut cursor = std::io::Cursor::new(chunks);

        while cursor.get_ref().len() as u64 - cursor.position() > 0 {
            let header_name_result = read_chunk_header_name(&mut cursor);
            let header_size = read_chunk_header_size(&mut cursor)?;
            let mut chunk: Vec<u8> = vec![0u8; header_size as usize];
            cursor
                .read_exact(&mut chunk)
                .chain_err(|| format!("failed to read chunk at {}", cursor.position()))?;

            if header_name_result.is_err() {
                continue;
            }

            let header_name = header_name_result.unwrap();
            let mut chunk_cursor = std::io::Cursor::new(&chunk);
            match header_name.as_ref() {
                NAME_SCENARIO_TYPE => {
                    map.scenario_type = Some(read_scenario_type(&mut chunk_cursor)?);
                }
                NAME_FILE_FORMAT_VERSION => {
                    if header_size != 2 {
                        return Err(format!("{} size must be 2 bytes", header_name).into());
                    }
                    map.file_format_version = Some(read_file_format_version(&mut chunk_cursor)?);
                }
                NAME_TILESET => {
                    if header_size != 2 {
                        return Err(format!("{} size must be 2 bytes", header_name).into());
                    }
                    map.tileset = Some(read_tileset(&mut chunk_cursor)?);
                }
                NAME_CONTROLLERS => {
                    if header_size != 12 {
                        return Err(format!("{} size must be 12 bytes", header_name).into());
                    }
                    let mut controllers: Vec<Controller> =
                        vec![Controller::Inactive; header_size as usize];
                    for i in 0..(header_size as usize) {
                        controllers[i] = read_controller(&mut chunk_cursor)?;
                    }
                    map.controllers = Some(controllers);
                }
                NAME_DIMENSIONS => {
                    if header_size != 4 {
                        return Err(format!("{} size must be 4 bytes", header_name).into());
                    }
                    map.dimensions = Some(read_dimensions(&mut chunk_cursor)?);
                }
                NAME_SIDE => {
                    if header_size != 12 {
                        return Err(format!("{} size must be 12 bytes", header_name).into());
                    }
                    let mut sides: Vec<Side> = vec![Side::Inactive; header_size as usize];
                    for i in 0..(header_size as usize) {
                        sides[i] = read_size(&mut chunk_cursor)?;
                    }
                    map.side = Some(sides);
                }
                NAME_MEGATILE => {
                    if header_size > 0x20000 {
                        return Err(format!(
                            "{} size must be less than 0x20000 bytes",
                            header_name
                        )
                        .into());
                    }
                    let tile_count = (header_size / 2) as usize;
                    let mut megatiles: Vec<MegaTileID> = Vec::with_capacity(tile_count);
                    for i in 0..tile_count {
                        megatiles.push(read_megatile_id(&mut chunk_cursor)?);
                    }
                    map.megatiles = Some(megatiles);
                }
                NAME_STRING_DATA => {
                    map.str_data = Some(read_str_data(&mut chunk_cursor)?);
                }
                _ => {}
            };
        }

        Ok(Map {
            scenario_type: map.scenario_type,
            file_format_version: map.file_format_version.ok_or_else(|| {
                format!(
                    "file format version - \"{}\" is required",
                    NAME_FILE_FORMAT_VERSION
                )
            })?,
            controllers: map
                .controllers
                .ok_or_else(|| format!("controllers - \"{}\" is required", NAME_CONTROLLERS))?,
            tileset: map
                .tileset
                .ok_or_else(|| format!("tileset - \"{}\" is required", NAME_TILESET))?,
            dimensions: map
                .dimensions
                .ok_or_else(|| format!("dimensions - \"{}\" is required", NAME_DIMENSIONS))?,
            side: map
                .side
                .ok_or_else(|| format!("side - \"{}\" is required", NAME_SIDE))?,
            megatiles: map
                .megatiles
                .ok_or_else(|| format!("megatiles - \"{}\" is required", NAME_MEGATILE))?,
            str_data: map
                .str_data
                .ok_or_else(|| format!("string data - \"{}\" is required", NAME_STRING_DATA))?,
        })
    }
}

fn read_scenario_type(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<ScenarioType> {
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
                None => return Err(format!("unsupported scenario type: {}", scenario_type).into()),
            },
        )?
}

fn read_file_format_version(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<FileFormatVersion> {
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

fn read_chunk_header_name(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<String> {
    let mut name_buf = [0u8; HEADER_NAME_BYTE_SIZE];
    let header_name = cursor
        .read_exact(&mut name_buf)
        .map(|_| std::str::from_utf8(&name_buf))
        .chain_err(|| "failed to read chunk header name")?
        .chain_err(|| "chunk header is not utf8 encoded")?;

    Ok(header_name.to_string())
}

fn read_chunk_header_size(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<u32> {
    let header_size = cursor
        .read_u32::<LittleEndian>()
        .chain_err(|| "failed to read chunk header size")?;

    Ok(header_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reads_utf8_chunk_header() {
        let buf: Vec<u8> = "ERA ".into();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_chunk_header_name(&mut cursor).expect("should read utf8 chunk header"),
            "ERA "
        )
    }

    #[test]
    fn test_fail_read_non_utf8_chunk_header() {
        let buf: Vec<u8> = vec![221, 248, 206, 155];
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(read_chunk_header_name(&mut cursor).is_err(), true)
    }

    #[test]
    fn test_reads_chunk_header_size() {
        let buf: Vec<u8> = (204 as u64).to_le_bytes().to_vec();
        let mut cursor = std::io::Cursor::new(&buf);
        assert_eq!(
            read_chunk_header_size(&mut cursor).expect("should read chunk header size"),
            204
        )
    }
}
