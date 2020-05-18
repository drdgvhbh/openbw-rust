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
use crossbeam;
pub use errors::*;
use std::sync::Arc;

pub mod chk;
pub mod cv5;
pub mod fs;
pub mod loader;
pub mod map;
pub mod mpq;
pub mod vf4;
pub mod vr4;
pub mod vx4;
pub mod wpe;

pub trait AssetLoader {
    fn load_cv5s(&self) -> Result<cv5::CV5s>;
    fn load_vf4s(&self) -> Result<vf4::VF4s>;
    fn load_vx4s(&self) -> Result<vx4::VX4s>;
    fn load_vr4s(&self) -> Result<vr4::VR4s>;
    fn load_wpes(&self) -> Result<wpe::WPEs>;
}

pub struct Assets {
    pub cv5s: cv5::CV5s,
    pub vf4s: vf4::VF4s,
    pub vx4s: vx4::VX4s,
    pub vr4s: vr4::VR4s,
    pub wpes: wpe::WPEs,
}

impl Assets {
    pub fn from<AL>(asset_loader: Arc<AL>) -> Result<Assets>
    where
        AL: AssetLoader + Send + Sync,
    {
        let mut cv5s: Result<cv5::CV5s> = Err("".into());
        let mut vf4s: Result<vf4::VF4s> = Err("".into());
        let mut vx4s: Result<vx4::VX4s> = Err("".into());
        let mut vr4s: Result<vr4::VR4s> = Err("".into());
        let mut wpes: Result<wpe::WPEs> = Err("".into());

        crossbeam::scope(|scope| {
            scope.spawn(|_| (cv5s = asset_loader.load_cv5s()));
            scope.spawn(|_| (vf4s = asset_loader.load_vf4s()));
            scope.spawn(|_| (vx4s = asset_loader.load_vx4s()));
            scope.spawn(|_| (vr4s = asset_loader.load_vr4s()));
            scope.spawn(|_| (wpes = asset_loader.load_wpes()));
        })
        .map_err(|_| "failed to load assets concurrently")?;

        Ok(Assets {
            cv5s: cv5s?,
            vf4s: vf4s?,
            vx4s: vx4s?,
            vr4s: vr4s?,
            wpes: wpes?,
        })
    }
}
