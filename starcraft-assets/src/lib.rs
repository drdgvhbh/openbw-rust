#[macro_use]
extern crate error_chain;

mod errors {
    error_chain! {
        errors {
            Error
        }
    }
}
pub use errors::*;

pub mod map;
