use super::errors::*;

pub trait FileSystem {
    fn read(&mut self, file_name: &str) -> Result<Vec<u8>>;
}
