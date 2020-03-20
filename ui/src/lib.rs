#[macro_use]
extern crate error_chain;

use rgb;
use starcraft_map;
use std::mem::MaybeUninit;

pub mod assets;
pub mod third_party;

pub mod errors {
    error_chain! {}
}
use errors::*;

pub fn generate_bitmap(
    dimensions: &starcraft_map::Dimensions,
    megatiles: &Vec<starcraft_map::MegaTileID>,
    terrain_data: &assets::terrain::TerrainData,
) -> Result<Vec<rgb::RGB8>> {
    let width = dimensions.width * 32;
    let height = dimensions.height * 32;
    let size = width * height;
    let mut out: Vec<rgb::RGB8> = Vec::new();
    out.resize(size, unsafe { MaybeUninit::uninit().assume_init() });

    use rayon::prelude::*;
    Ok(out
        .into_par_iter()
        .enumerate()
        .map(|(i, _)| {
            let x = i % width / 32;
            let y = i / width / 32;

            let megatile = &megatiles[x + y * dimensions.width];
            let megatile_id = terrain_data.cv5[megatile.group_index()].megatile_references
                [megatile.subtile_index()];

            let x2 = i % width % 32 / 8;
            let y2 = i / width % 32 / 8;
            let minitile = &terrain_data.vx4[megatile_id][x2 + y2 * 4];
            let wpe_ref = &terrain_data.vr4[minitile.index()];

            let x3 = i % width % 32 / 4;
            let y3 = i / width % 32 / 4;
            let color = if minitile.is_horizontally_flipped() {
                &terrain_data.wpe[wpe_ref[63 - (x3 + y3 * 8)]]
            } else {
                &terrain_data.wpe[wpe_ref[x3 + y3 * 8]]
            };

            color.clone()
        })
        .collect::<Vec<_>>())
}
