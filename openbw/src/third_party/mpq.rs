pub mod errors {
    error_chain! {}
}

use ceres_mpq;
use starcraft_assets;
use std::fs::File;

use std::io::{Cursor, Read, Seek};

#[derive(Debug)]
pub struct ReadonlyArchive<R>
where
    R: Read + Seek,
{
    archive: ceres_mpq::ReadonlyArchive<R>,
}

impl<R> ReadonlyArchive<R>
where
    R: Read + Seek + Clone,
{
    pub fn new(archive: ceres_mpq::ReadonlyArchive<R>) -> ReadonlyArchive<R> {
        ReadonlyArchive { archive: archive }
    }

    pub fn open(file_name: &str) -> errors::Result<ReadonlyArchive<Cursor<Vec<u8>>>> {
        use errors::*;

        let mut archive_file = File::with_options()
            .read(true)
            .open(file_name)
            .chain_err(|| format!("failed to open archive file: {}", file_name))?;
        let mut buf: Vec<u8> = Vec::new();
        archive_file
            .read_to_end(&mut buf)
            .chain_err(|| format!("failed to read archive file: {}", file_name))?;

        let archive = ceres_mpq::ReadonlyArchive::open(Cursor::new(buf))
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

    pub fn reader(&mut self) -> &mut R {
        self.archive.reader()
    }
}

impl<R> starcraft_assets::mpq::MPQArchive for ReadonlyArchive<R>
where
    R: Read + Seek + Clone,
{
    fn read_file(&self, file_name: &str) -> std::result::Result<Vec<u8>, starcraft_assets::Error> {
        use starcraft_assets::errors::*;

        ReadonlyArchive::read_file(self, file_name)
            .chain_err(|| format!("failed to read file: {}", file_name))
    }
}
