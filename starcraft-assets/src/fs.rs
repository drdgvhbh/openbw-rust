use super::errors::*;

pub trait ReadOnlyFileSystem: Send + Sync {
    fn read(&self, file_name: &str) -> Result<Vec<u8>>;
}
