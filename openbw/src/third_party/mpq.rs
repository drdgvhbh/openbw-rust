pub mod errors {
    error_chain! {}
}

use ceres_mpq;
use starcraft_assets;
use std::fs::File;
use std::sync::Arc;

use std::io::{Cursor, Read, Seek};

#[derive(Debug)]
pub struct ReadonlyArchive {
    archive: ceres_mpq::ReadonlyArchive,
}

impl ReadonlyArchive {
    pub fn new(archive: ceres_mpq::ReadonlyArchive) -> ReadonlyArchive {
        ReadonlyArchive { archive: archive }
    }

    pub fn open(file_name: &str) -> errors::Result<ReadonlyArchive> {
        use errors::*;

        let file_handle_factory = ceres_mpq::FileHandleFactory {
            file_name: file_name.into(),
        };

        let archive = ceres_mpq::ReadonlyArchive::open(file_handle_factory)
            .chain_err(|| "failed to open archive")?;

        Ok(ReadonlyArchive { archive: archive })
    }

    pub fn read_file(&self, name: &str) -> errors::Result<Vec<u8>> {
        use errors::*;

        self.archive
            .read_file(name)
            .chain_err(|| format!("failed to read file: {}", name))
    }

    pub fn files(&mut self) -> Option<Vec<String>> {
        self.archive.files()
    }

    pub fn size(&self) -> u64 {
        self.archive.size()
    }
}

impl starcraft_assets::mpq::ReadOnlyMPQArchive for ReadonlyArchive {
    fn read_file(&self, file_name: &str) -> std::result::Result<Vec<u8>, starcraft_assets::Error> {
        use starcraft_assets::errors::*;

        ReadonlyArchive::read_file(self, file_name)
            .chain_err(|| format!("failed to read file: {}", file_name))
    }
}
