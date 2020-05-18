#[macro_use(c)]
extern crate cute;

use starcraft_assets;

use std::io::Cursor;
use std::sync::Arc;

use openbw::{self, third_party};

use image;
use luminance;

use luminance::blending::{Equation, Factor};
use luminance::context::GraphicsContext as _;
use luminance::pipeline::{BoundTexture, PipelineState};
use luminance::pixel::{NormRGB8UI, NormUnsigned};
use luminance::render_state::RenderState;
use luminance::shader::program::Uniform;
use luminance::tess::{Mode, TessBuilder};
use luminance::texture::{Dim2, GenMipmaps, Texture};
use luminance_derive::{Semantics, UniformInterface, Vertex};
use luminance_glfw::{Action, Key, Surface, WindowEvent};

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

#[derive(Copy, Clone, Debug, Semantics)]
pub enum VertexSemantics {
    #[sem(name = "position", repr = "[f32; 2]", wrapper = "VertexPosition")]
    Position,
}

#[derive(Vertex)]
#[vertex(sem = "VertexSemantics")]
pub struct Vertex {
    position: VertexPosition,
}

fn main() {
    use stopwatch::Stopwatch;
    let sw = Stopwatch::start_new();

    let map = starcraft_assets::map::Map::from_mpq_file("(2)Destination.scx").unwrap();

    let unified_mpq_archive = starcraft_assets::mpq::UnifiedMPQArchive::from_existing(vec![
        third_party::mpq::ReadonlyArchive::open("StarDat.mpq").unwrap(),
        third_party::mpq::ReadonlyArchive::open("BrooDat.mpq").unwrap(),
    ]);

    let asset_loader = starcraft_assets::loader::AssetLoader::new(
        map.tileset.clone().into(),
        &unified_mpq_archive,
    );
    let assets = starcraft_assets::Assets::from(Arc::new(asset_loader)).unwrap();

    println!("Overall: Thing took {}ms", sw.elapsed_ms());

    let mut surface = luminance_glfw::GlfwSurface::new(
        luminance_glfw::WindowDim::Windowed(800, 600),
        "Open BW",
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

    let sw = Stopwatch::start_new();

    tex.upload_raw(
        GenMipmaps::Yes,
        openbw::ui::generate_bitmap(&map.dimensions, &map.mega_tile_ids, &assets)
            .map(|bitmap| {
                image::ImageBuffer::from_fn(
                    (map.dimensions.width * 32) as u32,
                    (map.dimensions.height * 32) as u32,
                    |x, y| {
                        let c = bitmap[(x
                            + (((map.dimensions.height * 32) as u32 - 1) - y)
                                * (map.dimensions.width as u32 * 32))
                            as usize];
                        image::Rgb(c)
                    },
                )
            })
            .unwrap()
            .into_raw()
            .as_slice(),
    )
    .unwrap();

    println!("Overall: Thing took {}ms", sw.elapsed_ms());

    // set the uniform interface to our type so that we can read textures from the shader
    let program = luminance::shader::program::Program::<(), (), ShaderInterface>::from_strings(
        None, VS, None, FS,
    )
    .expect("program creation")
    .ignore_warnings();

    const VERTICES: [Vertex; 4] = [
        Vertex {
            position: VertexPosition::new([-1., -1.]),
        },
        Vertex {
            position: VertexPosition::new([1., -1.]),
        },
        Vertex {
            position: VertexPosition::new([1., 1.]),
        },
        Vertex {
            position: VertexPosition::new([-1., 1.]),
        },
    ];

    // we’ll use an attributeless render here to display a quad on the screen (two triangles); there
    // are over ways to cover the whole screen but this is easier for you to understand; the
    // TriangleFan creates triangles by connecting the third (and next) vertex to the first one
    let tess = TessBuilder::new(&mut surface)
        .add_vertices(VERTICES)
        .set_mode(Mode::TriangleFan)
        .build()
        .unwrap();

    let mut back_buffer = surface.back_buffer().unwrap();
    let render_state =
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

                shd_gate.shade(&program, |program, mut rdr_gate| {
                    // update the texture; strictly speaking, this update doesn’t do much: it just tells the GPU
                    // to use the texture passed as argument (no allocation or copy is performed)
                    program.tex.update(&bound_tex);

                    rdr_gate.render(render_state, |mut tess_gate| {
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
