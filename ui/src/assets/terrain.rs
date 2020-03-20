pub mod errors {
    error_chain! {
        errors {
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
use byteorder::{LittleEndian, ReadBytesExt};
use errors::*;
use rgb;
use starcraft_map;
use std::fmt;
use std::io::Cursor;
use std::io::Read;
use std::mem::MaybeUninit;
use std::sync::{Arc, Mutex};
use std::thread;

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

impl From<starcraft_map::Tileset> for Tileset {
    fn from(tileset: starcraft_map::Tileset) -> Self {
        match tileset {
            starcraft_map::Tileset::Arctic => Tileset::Arctic,
            starcraft_map::Tileset::Ashworld => Tileset::Ashworld,
            starcraft_map::Tileset::Badlands => Tileset::Badlands,
            starcraft_map::Tileset::Desert => Tileset::Desert,
            starcraft_map::Tileset::Installation => Tileset::Installation,
            starcraft_map::Tileset::Jungle => Tileset::Jungle,
            starcraft_map::Tileset::SpacePlatform => Tileset::SpacePlatform,
            starcraft_map::Tileset::Twilight => Tileset::Twilight,
        }
    }
}

#[derive(Debug)]
pub enum EXT {
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

pub trait FileSystem {
    fn read(&mut self, file_name: &str) -> Result<Vec<u8>>;
}

pub const WPE_BLOCK_SIZE: usize = 3;
pub const CV5_STRUCT_SIZE: usize = 52;
pub const VF4_BLOCK_SIZE: usize = 16;
pub const VX4_BLOCK_SIZE: usize = 16;
pub const VR4_BLOCK_SIZE: usize = 64;

fn tileset_path(ext: &EXT, tileset: &Tileset) -> String {
    format!("tileset\\{}.{}", tileset.file_name(), ext.file_name())
}

#[derive(Debug, Clone)]
pub struct VX4 {
    value: u16,
}

impl VX4 {
    pub fn is_horizontally_flipped(&self) -> bool {
        return self.value & 1 == 1;
    }

    pub fn index(&self) -> usize {
        return (self.value >> 1) as usize;
    }
}

#[derive(Debug, Clone)]
pub struct VF4 {
    value: u16,
}

const WALKABLE: u16 = 0x0001;
const MID: u16 = 0x0002;
const HIGH: u16 = 0x0004;
const LOW: u16 = 0x0004 | 0x0002;
const BLOCKS_VIEW: u16 = 0x0008;
const RAMP: u16 = 0x0010;
const MEGA_TILE_REFERENCE_COUNT: usize = 16;

impl VF4 {
    pub fn is_walkable(&self) -> bool {
        return self.value & WALKABLE == WALKABLE;
    }

    pub fn is_elevation_mid(&self) -> bool {
        return self.value & MID == MID;
    }

    pub fn is_elevation_high(&self) -> bool {
        return self.value & HIGH == HIGH;
    }

    pub fn is_elevation_low(&self) -> bool {
        return self.value & LOW == LOW;
    }

    pub fn blocks_view(&self) -> bool {
        return self.value & BLOCKS_VIEW == BLOCKS_VIEW;
    }

    pub fn is_ramp(&self) -> bool {
        return self.value & RAMP == RAMP;
    }
}

#[derive(Debug, Clone)]
pub struct CV5 {
    pub megatile_references: [usize; MEGA_TILE_REFERENCE_COUNT],
}

#[derive(Debug)]
pub struct TilesetAssetLoader<FS>
where
    FS: FileSystem,
{
    new_fs: fn() -> FS,
}

impl<FS> TilesetAssetLoader<FS>
where
    FS: FileSystem,
{
    pub fn new(new_fs: fn() -> FS) -> TilesetAssetLoader<FS> {
        TilesetAssetLoader { new_fs: new_fs }
    }

    pub fn load_cv5(&self, out: &mut Vec<CV5>, tileset: Tileset) -> Result<()> {
        let mut fs = (self.new_fs)();
        let file_path = &tileset_path(&EXT::CV5, &tileset);
        let src = fs
            .read(&file_path)
            .map_err(|_| ErrorKind::AssetNotFound(tileset.file_name(), EXT::WPE.file_name()))?;

        let mut cursor = Cursor::new(&src);
        let size = src.len() / CV5_STRUCT_SIZE;
        out.resize(size, unsafe { MaybeUninit::uninit().assume_init() });
        for i in 0..size {
            cursor.set_position(cursor.position() + 20);
            let mut megatile_references: [u16; MEGA_TILE_REFERENCE_COUNT] =
                unsafe { MaybeUninit::uninit().assume_init() };
            cursor
                .read_u16_into::<LittleEndian>(&mut megatile_references)
                .chain_err(|| {
                    ErrorKind::IncorrectFileFormat(cursor.position(), file_path.to_string())
                })?;
            out[i] = CV5 {
                megatile_references: unsafe { MaybeUninit::uninit().assume_init() },
            };
            for j in 0..MEGA_TILE_REFERENCE_COUNT {
                out[i].megatile_references[j] = megatile_references[j] as usize;
            }
        }

        Ok(())
    }

    pub fn load_vf4(&self, out: &mut Vec<[VF4; VF4_BLOCK_SIZE]>, tileset: Tileset) -> Result<()> {
        let mut fs = (self.new_fs)();
        let file_path = &tileset_path(&EXT::VF4, &tileset);
        let src = fs
            .read(&file_path)
            .map_err(|_| ErrorKind::AssetNotFound(tileset.file_name(), EXT::WPE.file_name()))?;

        let mut cursor = Cursor::new(&src);

        let size = src.len() / (VF4_BLOCK_SIZE * std::mem::size_of::<u16>()) as usize;
        out.resize(size, unsafe { MaybeUninit::uninit().assume_init() });
        let mut out_bytes: [u16; VF4_BLOCK_SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
        for i in 0..size {
            cursor
                .read_u16_into::<LittleEndian>(&mut out_bytes)
                .chain_err(|| {
                    ErrorKind::IncorrectFileFormat(cursor.position(), file_path.to_string())
                })?;
            for j in 0..VF4_BLOCK_SIZE {
                out[i][j] = VF4 {
                    value: out_bytes[j],
                }
            }
        }

        Ok(())
    }

    pub fn load_vx4(&self, out: &mut Vec<[VX4; VX4_BLOCK_SIZE]>, tileset: Tileset) -> Result<()> {
        let mut fs = (self.new_fs)();
        let file_path = &tileset_path(&EXT::VX4, &tileset);
        let src = fs
            .read(&file_path)
            .map_err(|_| ErrorKind::AssetNotFound(tileset.file_name(), EXT::WPE.file_name()))?;

        let mut cursor = Cursor::new(&src);

        let size = src.len() / (VX4_BLOCK_SIZE * std::mem::size_of::<u16>()) as usize;
        out.resize(size, unsafe { MaybeUninit::uninit().assume_init() });
        let mut out_bytes: [u16; VX4_BLOCK_SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
        for i in 0..size {
            cursor
                .read_u16_into::<LittleEndian>(&mut out_bytes)
                .chain_err(|| {
                    ErrorKind::IncorrectFileFormat(cursor.position(), file_path.to_string())
                })?;
            for j in 0..VX4_BLOCK_SIZE {
                out[i][j] = VX4 {
                    value: out_bytes[j],
                };
            }
        }

        Ok(())
    }

    pub fn load_vr4(&self, out: &mut Vec<[usize; VR4_BLOCK_SIZE]>, tileset: Tileset) -> Result<()> {
        let mut fs = (self.new_fs)();
        let file_path = &tileset_path(&EXT::VR4, &tileset);
        let src = fs
            .read(&file_path)
            .map_err(|_| ErrorKind::AssetNotFound(tileset.file_name(), EXT::WPE.file_name()))?;

        let mut cursor = Cursor::new(&src);

        let size = src.len() / VR4_BLOCK_SIZE as usize;
        out.resize(size, unsafe { MaybeUninit::uninit().assume_init() });
        let mut out_bytes: [u8; VR4_BLOCK_SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
        for i in 0..size {
            cursor.read(&mut out_bytes).chain_err(|| {
                ErrorKind::IncorrectFileFormat(cursor.position(), file_path.to_string())
            })?;
            for j in 0..VR4_BLOCK_SIZE {
                out[i][j] = out_bytes[j] as usize;
            }
        }

        Ok(())
    }

    pub fn load_wpe(&self, out: &mut Vec<rgb::RGB8>, tileset: Tileset) -> Result<()> {
        let mut fs = (self.new_fs)();
        let file_path = &tileset_path(&EXT::WPE, &tileset);
        let src = fs
            .read(&file_path)
            .map_err(|_| ErrorKind::AssetNotFound(tileset.file_name(), EXT::WPE.file_name()))?;
        let mut cursor = Cursor::new(&src);

        let size = src.len() / (WPE_BLOCK_SIZE + 1) as usize;
        out.resize(size, unsafe { MaybeUninit::uninit().assume_init() });
        let mut out_bytes: [u8; WPE_BLOCK_SIZE] = unsafe { MaybeUninit::uninit().assume_init() };
        for i in 0..size {
            cursor.read(&mut out_bytes).chain_err(|| {
                ErrorKind::IncorrectFileFormat(cursor.position(), file_path.to_string())
            })?;
            cursor.set_position(cursor.position() + 1);
            out[i] = rgb::RGB8 {
                r: out_bytes[0],
                g: out_bytes[1],
                b: out_bytes[2],
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct TerrainData {
    pub cv5: Vec<CV5>,
    pub vf4: Vec<[VF4; VF4_BLOCK_SIZE]>,
    pub vx4: Vec<[VX4; VX4_BLOCK_SIZE]>,
    pub vr4: Vec<[usize; VR4_BLOCK_SIZE]>,
    pub wpe: Vec<rgb::RGB8>,
}

impl TerrainData {
    pub fn load<FS>(loader: TilesetAssetLoader<FS>, tileset: Tileset) -> Result<TerrainData>
    where
        FS: FileSystem + 'static,
    {
        let terrain_data = Arc::new(Mutex::new(TerrainData {
            cv5: Vec::new(),
            vf4: Vec::new(),
            vx4: Vec::new(),
            vr4: Vec::new(),
            wpe: Vec::new(),
        }));
        let loader_arc = Arc::new(loader);
        let mut threads = vec![];
        for ext in &[EXT::CV5, EXT::VF4, EXT::VX4, EXT::VR4, EXT::WPE] {
            let loader_clone = loader_arc.clone();
            match ext {
                EXT::CV5 => {
                    let terrain_data_clone = terrain_data.clone();
                    let scenario_type_clone = tileset.clone();
                    threads.push(thread::spawn(move || {
                        let mut cv5 = Vec::new();
                        loader_clone
                            .load_cv5(&mut cv5, scenario_type_clone)
                            .unwrap();
                        terrain_data_clone.lock().unwrap().cv5 = cv5;
                    }));
                }
                EXT::VF4 => {
                    let terrain_data_clone = terrain_data.clone();
                    let scenario_type_clone = tileset.clone();
                    threads.push(thread::spawn(move || {
                        let mut vf4 = Vec::new();
                        loader_clone
                            .load_vf4(&mut vf4, scenario_type_clone)
                            .unwrap();
                        terrain_data_clone.lock().unwrap().vf4 = vf4;
                    }));
                }
                EXT::VX4 => {
                    let terrain_data_clone = terrain_data.clone();
                    let scenario_type_clone = tileset.clone();
                    threads.push(thread::spawn(move || {
                        let mut vx4 = Vec::new();
                        loader_clone
                            .load_vx4(&mut vx4, scenario_type_clone)
                            .unwrap();
                        terrain_data_clone.lock().unwrap().vx4 = vx4;
                    }));
                }
                EXT::VR4 => {
                    let terrain_data_clone = terrain_data.clone();
                    let scenario_type_clone = tileset.clone();
                    threads.push(thread::spawn(move || {
                        let mut vr4 = Vec::new();
                        loader_clone
                            .load_vr4(&mut vr4, scenario_type_clone)
                            .unwrap();
                        terrain_data_clone.lock().unwrap().vr4 = vr4;
                    }));
                }
                EXT::WPE => {
                    let terrain_data_clone = terrain_data.clone();
                    let scenario_type_clone = tileset.clone();
                    threads.push(thread::spawn(move || {
                        let mut wpe = Vec::new();
                        loader_clone
                            .load_wpe(&mut wpe, scenario_type_clone)
                            .unwrap();
                        terrain_data_clone.lock().unwrap().wpe = wpe;
                    }));
                }
            }
        }
        for thread in threads {
            thread.join().unwrap();
        }

        let lock = Arc::try_unwrap(terrain_data).ok();
        Ok(lock.unwrap().into_inner().expect("Mutex cannot be locked"))
    }
}
