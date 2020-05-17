pub mod errors {
    error_chain! {}
}

use ceres_mpq;
use starcraft_assets;
use std::fs;

use std::io::{Cursor, Read, Seek};

#[derive(Debug)]
pub struct Archive<R>
where
    R: Read + Seek,
{
    archive: ceres_mpq::Archive<R>,
}

impl<R> Archive<R>
where
    R: Read + Seek,
{
    pub fn new(archive: ceres_mpq::Archive<R>) -> Archive<R> {
        Archive { archive: archive }
    }

    pub fn open(file_name: &str) -> errors::Result<Archive<Cursor<Vec<u8>>>> {
        use errors::*;

        let mut archive_file = fs::File::open(file_name)
            .chain_err(|| format!("failed to open archive file: {}", file_name))?;
        let mut buf: Vec<u8> = Vec::new();
        archive_file
            .read_to_end(&mut buf)
            .chain_err(|| format!("failed to read archive file: {}", file_name))?;

        let archive =
            ceres_mpq::Archive::open(Cursor::new(buf)).chain_err(|| "failed to open archive")?;

        Ok(Archive { archive: archive })
    }

    pub fn read_file(&mut self, name: &str) -> errors::Result<Vec<u8>> {
        use errors::*;

        self.archive
            .read_file(name)
            .chain_err(|| format!("failed to read file: {}", name))
    }

    pub fn files(&mut self) -> Option<Vec<String>> {
        self.archive.files()
    }

    pub fn start(&self) -> u64 {
        self.archive.start()
    }

    pub fn end(&self) -> u64 {
        self.archive.end()
    }

    pub fn size(&self) -> u64 {
        self.archive.size()
    }

    pub fn reader(&mut self) -> &mut R {
        self.archive.reader()
    }
}

impl<R> starcraft_assets::mpq::MPQArchive for Archive<R>
where
    R: Read + Seek,
{
    fn read_file(
        &mut self,
        file_name: &str,
    ) -> std::result::Result<Vec<u8>, starcraft_assets::Error> {
        use starcraft_assets::errors::*;
        use starcraft_assets::mpq;

        Archive::read_file(self, file_name)
            .chain_err(|| format!("failed to read file: {}", file_name))
    }
}
