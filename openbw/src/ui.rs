use super::errors::*;
use rgb;
use starcraft_assets;
use std::mem::MaybeUninit;

pub fn generate_bitmap(
    dimensions: &starcraft_assets::chk::Dimensions,
    megatiles: &Vec<starcraft_assets::chk::MegaTileID>,
    assets: &starcraft_assets::Assets,
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
            let megatile_id = assets.cv5s.0[megatile.group_index()].0[megatile.subtile_index()];

            let x2 = i % width % 32 / 8;
            let y2 = i / width % 32 / 8;
            let minitile = &assets.vx4s.0[megatile_id][x2 + y2 * 4];
            let wpe_ref = &assets.vr4s.0[minitile.index()];

            let x3 = i % width % 32 % 8;
            let y3 = i / width % 32 % 8;
            let color = if minitile.is_horizontally_flipped() {
                &assets.wpes.0[wpe_ref[(7 - x3) + y3 * 8]]
            } else {
                &assets.wpes.0[wpe_ref[x3 + y3 * 8]]
            };

            color.clone()
        })
        .map(|x| x.0)
        .collect::<Vec<_>>())
}
