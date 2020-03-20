pub mod errors {
    error_chain! {
        errors {
            IO
        }
    }
}

pub trait MPQArchive {
    fn read_file(&mut self, name: &str) -> errors::Result<Vec<u8>>;
}

pub struct UnifiedMPQArchive<A>
where
    A: MPQArchive,
{
    archives: Vec<A>,
}

impl<A> UnifiedMPQArchive<A>
where
    A: MPQArchive,
{
    pub fn new() -> UnifiedMPQArchive<A> {
        UnifiedMPQArchive {
            archives: Vec::new(),
        }
    }

    pub fn from_existing(archives: impl IntoIterator<Item = A>) -> UnifiedMPQArchive<A> {
        UnifiedMPQArchive {
            archives: archives.into_iter().collect::<Vec<A>>(),
        }
    }

    pub fn add(&mut self, archive: A) {
        self.archives.push(archive)
    }

    pub fn read_file(&mut self, file_name: &str) -> errors::Result<Vec<u8>> {
        for archive in &mut self.archives {
            let result = archive.read_file(file_name);
            if result.is_err() {
                continue;
            }

            return Ok(result.unwrap());
        }

        Err(format!("{} not found", file_name).into())
    }
}

impl<A> super::terrain::FileSystem for UnifiedMPQArchive<A>
where
    A: MPQArchive,
{
    fn read(
        &mut self,
        file_name: &str,
    ) -> std::result::Result<Vec<u8>, super::terrain::errors::Error> {
        use super::terrain::errors::*;
        UnifiedMPQArchive::read_file(self, file_name)
            .chain_err(|| format!("failed to read: {}", file_name))
    }
}
