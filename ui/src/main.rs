use sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;

use starcraft_map;
use std::thread;

use std::io::Cursor;

use openbw::{self, assets, third_party};

use image;
use luminance;
use rgb;

use luminance::blending::{Equation, Factor};
use luminance::context::GraphicsContext as _;
use luminance::pipeline::{BoundTexture, PipelineState};
use luminance::pixel::{NormRGB8UI, NormUnsigned};
use luminance::render_state::RenderState;
use luminance::shader::program::{Program, Uniform};
use luminance::tess::{Mode, TessBuilder};
use luminance::texture::{Dim2, GenMipmaps, Sampler, Texture};
use luminance_derive::UniformInterface;
use luminance_glfw::{Action, GlfwSurface, Key, Surface, WindowDim, WindowEvent, WindowOpt};

const VS: &'static str = include_str!("texture-vs.glsl");
const FS: &'static str = include_str!("texture-fs.glsl");

struct Game {}

struct UIConfig {
    pub screen_width: u32,
    pub screen_height: u32,
}

#[derive(UniformInterface)]
struct ShaderInterface {
    // the 'static lifetime acts as “anything” here
    tex: Uniform<&'static BoundTexture<'static, Dim2, NormUnsigned>>,
}

fn main() {
    use stopwatch::Stopwatch;
    let sw = Stopwatch::start_new();

    let map = starcraft_map::Map::from_mpq_file("(2)Heartbreak Ridge.scx").unwrap();

    let new_unified_archive = || {
        let starcraft_archive =
            third_party::mpq::Archive::<Cursor<Vec<u8>>>::open("StarDat.mpq").unwrap();
        let broodwar_archive =
            third_party::mpq::Archive::<Cursor<Vec<u8>>>::open("BrooDat.mpq").unwrap();
        assets::mpq::UnifiedMPQArchive::from_existing(vec![starcraft_archive, broodwar_archive])
    };
    let terrain_data = assets::terrain::TerrainData::load(
        assets::terrain::TilesetAssetLoader::new(new_unified_archive),
        map.tileset.clone().into(),
    )
    .unwrap();

    println!("Overall: Thing took {}ms", sw.elapsed_ms());
    let sw = Stopwatch::start_new();
    let bitmap = openbw::generate_bitmap(&map.dimensions, &map.megatiles, &terrain_data).unwrap();
    println!("Overall: Thing took {}ms", sw.elapsed_ms());
    let img = image::ImageBuffer::from_fn(
        (map.dimensions.width * 32) as u32,
        (map.dimensions.height * 32) as u32,
        |x, y| {
            let c = bitmap[(x + y * (map.dimensions.width as u32 * 32)) as usize];
            image::Rgb([c.r, c.g, c.b])
        },
    );
    println!("Overall: Thing took {}ms", sw.elapsed_ms());
    let texels = img.into_raw();

    let mut surface = luminance_glfw::GlfwSurface::new(
        luminance_glfw::WindowDim::Windowed(900, 600),
        "Hello, world!",
        luminance_glfw::WindowOpt::default(),
    )
    .expect("GLFW surface creation");
    let tex: Texture<Dim2, NormRGB8UI> = luminance::texture::Texture::new(
        &mut surface,
        [
            (map.dimensions.width * 32) as u32,
            (map.dimensions.height * 32) as u32,
        ],
        0,
        luminance::texture::Sampler::default(),
    )
    .expect("luminance texture creation");

    tex.upload_raw(GenMipmaps::No, &texels).unwrap();

    // set the uniform interface to our type so that we can read textures from the shader
    let program = luminance::shader::program::Program::<(), (), ShaderInterface>::from_strings(
        None, VS, None, FS,
    )
    .expect("program creation")
    .ignore_warnings();

    // we’ll use an attributeless render here to display a quad on the screen (two triangles); there
    // are over ways to cover the whole screen but this is easier for you to understand; the
    // TriangleFan creates triangles by connecting the third (and next) vertex to the first one
    let tess = TessBuilder::new(&mut surface)
        .set_vertex_nb(4)
        .set_mode(Mode::TriangleFan)
        .build()
        .unwrap();

    let mut back_buffer = surface.back_buffer().unwrap();
    let render_st =
        &RenderState::default().set_blending((Equation::Additive, Factor::SrcAlpha, Factor::Zero));
    let mut resize = false;

    println!("rendering!");

    'app: loop {
        for event in surface.poll_events() {
            match event {
                WindowEvent::Close | WindowEvent::Key(Key::Escape, _, Action::Release, _) => {
                    break 'app
                }

                WindowEvent::FramebufferSize(..) => {
                    resize = true;
                }

                _ => (),
            }
        }

        if resize {
            back_buffer = surface.back_buffer().unwrap();
            resize = false;
        }

        // here, we need to bind the pipeline variable; it will enable us to bind the texture to the GPU
        // and use it in the shader
        surface.pipeline_builder().pipeline(
            &back_buffer,
            &PipelineState::default(),
            |pipeline, mut shd_gate| {
                // bind our fancy texture to the GPU: it gives us a bound texture we can use with the shader
                let bound_tex = pipeline.bind_texture(&tex);

                shd_gate.shade(&program, |iface, mut rdr_gate| {
                    // update the texture; strictly speaking, this update doesn’t do much: it just tells the GPU
                    // to use the texture passed as argument (no allocation or copy is performed)
                    iface.tex.update(&bound_tex);

                    rdr_gate.render(render_st, |mut tess_gate| {
                        // render the tessellation to the surface the regular way and let the vertex shader’s
                        // magic do the rest!
                        tess_gate.render(&tess);
                    });
                });
            },
        );

        surface.swap_buffers();
    }
}