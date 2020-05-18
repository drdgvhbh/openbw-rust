use super::errors::*;

pub trait ReadOnlyMPQArchive {
    fn read_file(&self, name: &str) -> Result<Vec<u8>>;
}

pub struct UnifiedMPQArchive<A>
where
    A: ReadOnlyMPQArchive,
{
    archives: Vec<A>,
}

impl<A> UnifiedMPQArchive<A>
where
    A: ReadOnlyMPQArchive,
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

    pub fn read_file(&self, file_name: &str) -> Result<Vec<u8>> {
        for archive in &self.archives {
            let result = archive.read_file(file_name);
            if result.is_err() {
                continue;
            }

            return Ok(result.unwrap());
        }

        Err(format!("{} not found", file_name).into())
    }
}

impl<A> super::fs::ReadOnlyFileSystem for UnifiedMPQArchive<A>
where
    A: ReadOnlyMPQArchive + Sync + Send,
{
    fn read(&self, file_name: &str) -> std::result::Result<Vec<u8>, Error> {
        UnifiedMPQArchive::read_file(self, file_name)
            .chain_err(|| format!("failed to read: {}", file_name))
    }
}
