use super::*;
use std::fmt;
use std::io::Cursor;

#[derive(Debug, Clone)]
pub enum Tileset {
    Ashworld,
    Badlands,
    Installation,
    Jungle,
    SpacePlatform,
    Desert,
    Arctic,
    Twilight,
}

impl Tileset {
    fn file_name(&self) -> String {
        match self {
            Tileset::Ashworld => "ashworld".into(),
            Tileset::Badlands => "badlands".into(),
            Tileset::Installation => "install".into(),
            Tileset::Jungle => "jungle".into(),
            Tileset::SpacePlatform => "platform".into(),
            Tileset::Desert => "desert".into(),
            Tileset::Arctic => "ice".into(),
            Tileset::Twilight => "twilight".into(),
        }
    }
}

impl fmt::Display for Tileset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Tileset::Ashworld => write!(f, "Ash"),
            Tileset::Badlands => write!(f, "Badland"),
            Tileset::Installation => write!(f, "Installation"),
            Tileset::Jungle => write!(f, "Jungle"),
            Tileset::SpacePlatform => write!(f, "Space Platform"),
            Tileset::Desert => write!(f, "Desert"),
            Tileset::Arctic => write!(f, "Arctic"),
            Tileset::Twilight => write!(f, "Twilight"),
        }
    }
}

impl From<super::chk::Tileset> for Tileset {
    fn from(tileset: super::chk::Tileset) -> Self {
        match tileset {
            chk::Tileset::Arctic => Tileset::Arctic,
            chk::Tileset::Ashworld => Tileset::Ashworld,
            chk::Tileset::Badlands => Tileset::Badlands,
            chk::Tileset::Desert => Tileset::Desert,
            chk::Tileset::Installation => Tileset::Installation,
            chk::Tileset::Jungle => Tileset::Jungle,
            chk::Tileset::SpacePlatform => Tileset::SpacePlatform,
            chk::Tileset::Twilight => Tileset::Twilight,
        }
    }
}

#[derive(Debug)]
enum EXT {
    CV5,
    VF4,
    VX4,
    VR4,
    WPE,
}

impl EXT {
    fn file_name(&self) -> String {
        match self {
            EXT::CV5 => "cv5".into(),
            EXT::VF4 => "vf4".into(),
            EXT::VX4 => "vx4".into(),
            EXT::VR4 => "vr4".into(),
            EXT::WPE => "wpe".into(),
        }
    }
}

pub struct AssetLoader<'a> {
    pub tileset: Tileset,
    pub fs: &'a dyn super::fs::ReadOnlyFileSystem,
}

impl<'a> AssetLoader<'a> {
    pub fn new(tileset: Tileset, fs: &'a dyn super::fs::ReadOnlyFileSystem) -> AssetLoader {
        AssetLoader { tileset, fs }
    }

    fn load_asset(&self, ext: EXT) -> Result<Vec<u8>> {
        let tileset_path = format!("tileset\\{}.{}", self.tileset.file_name(), ext.file_name());
        let buf = self.fs.read(&tileset_path).map_err(|_| {
            ErrorKind::AssetNotFound(self.tileset.file_name(), EXT::WPE.file_name())
        })?;

        Ok(buf)
    }
}

impl<'a> super::AssetLoader for AssetLoader<'a> {
    fn load_cv5s(&self) -> std::result::Result<cv5::CV5s, errors::Error> {
        let buf = self.load_asset(EXT::CV5)?;
        let mut cursor = Cursor::new(&buf);

        cv5::CV5s::from_buffer(&mut cursor).chain_err(|| "failed to load cv5")
    }

    fn load_vf4s(&self) -> std::result::Result<vf4::VF4s, errors::Error> {
        let buf = self.load_asset(EXT::VF4)?;
        let mut cursor = Cursor::new(&buf);

        vf4::VF4s::from_buffer(&mut cursor).chain_err(|| "failed to load vf4s")
    }

    fn load_vx4s(&self) -> std::result::Result<vx4::VX4s, errors::Error> {
        let buf = self.load_asset(EXT::VX4)?;
        let mut cursor = Cursor::new(&buf);

        vx4::VX4s::from_buffer(&mut cursor).chain_err(|| "failed to load vx4s")
    }

    fn load_vr4s(&self) -> std::result::Result<vr4::VR4s, errors::Error> {
        let buf = self.load_asset(EXT::VR4)?;
        let mut cursor = Cursor::new(&buf);

        vr4::VR4s::from_buffer(&mut cursor).chain_err(|| "failed to load vr4s")
    }

    fn load_wpes(&self) -> std::result::Result<wpe::WPEs, errors::Error> {
        let buf = self.load_asset(EXT::WPE)?;
        let mut cursor = Cursor::new(&buf);

        wpe::WPEs::from_buffer(&mut cursor).chain_err(|| "failed to load wpes")
    }
}
