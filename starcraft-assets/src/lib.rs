#[macro_use]
extern crate error_chain;

pub mod errors {
    error_chain! {
        errors {
            IO
            AssetNotFound(scenario_type: String, ext: String) {
                description("asset not found")
                display("asset not found: '{}.{}'", scenario_type, ext)
            }
            IncorrectFileFormat(pos: u64, file_path: String) {
                description("incorrect file format")
                display(
                    "incorrect file format: {} is invalid at position: {}",
                    file_path,
                    pos)
            }
        }
    }
}
pub use errors::*;

pub mod chk;
pub mod fs;
pub mod map;
pub mod mpq;
pub mod vf4;
pub mod wpe;
