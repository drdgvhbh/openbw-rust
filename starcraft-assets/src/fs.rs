use super::errors::*;

pub trait ReadonlyFileSystem: Send + Sync {
    fn read(&self, file_name: &str) -> Result<Vec<u8>>;
}
