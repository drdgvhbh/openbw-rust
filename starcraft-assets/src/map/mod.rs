use ceres_mpq;
use std::io::{Cursor, Read};

mod chk;
pub use chk::*;

use super::errors::*;

#[derive(Debug)]
pub struct Map {
    pub scenario_type: Option<chk::ScenarioType>,
    pub file_format_version: chk::FileFormatVersion,
    pub tileset: chk::Tileset,
    pub controllers: Vec<chk::Controller>,
    pub dimensions: chk::Dimensions,
    pub sides: Vec<chk::Side>,
    pub mega_tile_ids: Vec<chk::MegaTileID>,
    pub str_data: chk::StringData,
}

const MAP_FILE_NAME: &str = "staredit\\scenario.chk";

impl Map {
    pub fn from_mpq_file(file_name: &str) -> Result<Map> {
        let mut mpq_file = std::fs::File::open(file_name)
            .chain_err(|| format!("failed to open mpq file {}", file_name))?;

        let mut buf: Vec<u8> = Vec::new();
        mpq_file
            .read_to_end(&mut buf)
            .chain_err(|| format!("failed to read mpq file {}", file_name))?;

        let mut cursor = Cursor::new(buf);
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
        let mut cursor = std::io::Cursor::new(chunks);

        let mut scenario_type: Option<chk::ScenarioType> = None;
        let mut file_format_version: Option<chk::FileFormatVersion> = None;
        let mut tileset: Option<chk::Tileset> = None;
        let mut controllers: Option<Vec<chk::Controller>> = None;
        let mut dimensions: Option<chk::Dimensions> = None;
        let mut sides: Option<Vec<chk::Side>> = None;
        let mut mega_tile_ids: Option<Vec<chk::MegaTileID>> = None;
        let mut str_data: Option<chk::StringData> = None;

        while cursor.get_ref().len() as u64 - cursor.position() > 0 {
            let chunk_header = chk::Header::from_buffer(&mut cursor)?;

            let mut chunk: Vec<u8> = vec![0u8; chunk_header.size];
            cursor
                .read_exact(&mut chunk)
                .chain_err(|| format!("failed to read chunk at {}", cursor.position()))?;

            let mut cursor = Cursor::new(&chunk);

            let deserialized_chunk = chk::Chunk::read(&chunk_header, &mut cursor)?;
            match deserialized_chunk {
                Some(chk::Chunk::ScenarioType(a)) => scenario_type = Some(a),
                Some(chk::Chunk::FileFormatVersion(a)) => file_format_version = Some(a),
                Some(chk::Chunk::Tileset(a)) => tileset = Some(a),
                Some(chk::Chunk::Controllers(a)) => controllers = Some(a),
                Some(chk::Chunk::Dimensions(a)) => dimensions = Some(a),
                Some(chk::Chunk::Sides(a)) => sides = Some(a),
                Some(chk::Chunk::MegaTileIDs(a)) => mega_tile_ids = Some(a),
                Some(chk::Chunk::StringData(a)) => str_data = Some(a),
                None => {}
            };
        }

        Ok(Map {
            scenario_type: scenario_type,
            file_format_version: file_format_version.ok_or("file format version is required")?,
            tileset: tileset.ok_or("tileset is required")?,
            controllers: controllers.ok_or("tileset is required")?,
            dimensions: dimensions.ok_or("dimensions is required")?,
            sides: sides.ok_or("side is required")?,
            mega_tile_ids: mega_tile_ids.ok_or("megaTileIds is required")?,
            str_data: str_data.ok_or("string data is required")?,
        })
    }
}
