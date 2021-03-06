use anyhow::{Context, Result};
use glium::{backend::Facade, *};
use image::{DynamicImage, EncodableLayout, GenericImageView};
use resource::{resource, resource_str};

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}
implement_vertex!(Vertex, position);

trait Renderable {
    fn update(&mut self, delta: std::time::Duration) -> Result<()>;
    fn render(&self, frame: &mut Frame) -> Result<(), glium::DrawError>;
    fn custom_render(
        &self,
        frame: &mut Frame,
        params: &glium::draw_parameters::DrawParameters,
    ) -> Result<(), glium::DrawError>;
}

struct RedTriangle {
    vertices: glium::vertex::VertexBuffer<Vertex>,
    program: glium::program::Program,
}

impl RedTriangle {
    fn new<F: glium::backend::Facade>(facade: &F) -> RedTriangle {
        let shape = vec![
            Vertex {
                position: [-0.5, -0.5],
            },
            Vertex {
                position: [0.0, 0.5],
            },
            Vertex {
                position: [0.5, -0.25],
            },
        ];
        let vertices = glium::VertexBuffer::new(facade, &shape).unwrap();
        let vertex_shader_src = r#"
    #version 140

    in vec2 position;

    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
    }
"#;
        let fragment_shader_src = r#"
    #version 140

    out vec4 color;

    void main() {
        color = vec4(1.0, 0.0, 0.0, 1.0);
    }
"#;
        let program =
            glium::Program::from_source(facade, vertex_shader_src, fragment_shader_src, None)
                .unwrap();

        RedTriangle { vertices, program }
    }
}

impl Renderable for RedTriangle {
    fn render(&self, frame: &mut Frame) -> Result<(), glium::DrawError> {
        self.custom_render(frame, &Default::default())
    }
    fn custom_render(
        &self,
        frame: &mut Frame,
        params: &glium::draw_parameters::DrawParameters,
    ) -> Result<(), glium::DrawError> {
        frame.draw(
            &self.vertices,
            glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
            &self.program,
            &glium::uniforms::EmptyUniforms,
            &params,
        )
    }

    fn update(&mut self, delta: std::time::Duration) -> Result<()> {
        let mut data = self
            .vertices
            .read()
            .context("could not read OpenGL buffer")?;

        data[0].position[0] = (delta.as_nanos() as f32).cos();
        self.vertices.write(&data);

        Ok(())
    }
}

fn load_image(raw_data: &[u8]) -> Result<image::DynamicImage> {
    use image::io::*;

    let reader = Reader::new(std::io::Cursor::new(raw_data))
        .with_guessed_format()
        .context("could not read buffer")?;

    assert_eq!(reader.format(), Some(image::ImageFormat::Png));
    reader.decode().context("must decode")
}

fn make_things_from_image<F: glium::backend::Facade>(
    facade: &F,
    _img: &DynamicImage,
) -> Result<()> {
    let _program = glium::program::ComputeShader::from_source(
        facade,
        r#"\
    #version 430
    layout(local_size_x = 1, local_size_y = 1, local_size_z = 1) in;
    layout(std140) buffer MyBlock {
        float power;
        vec4 values[4096/4];
    };
    void main() {
        vec4 val = values[gl_GlobalInvocationID.x];
        values[gl_GlobalInvocationID.x] = pow(val, vec4(power));
    }
"#,
    )
    .context("no compute shader")?;

    Ok(())
}

struct ImageQuad {
    texture: glium::texture::Texture2d,
    program: glium::program::Program,
}

impl ImageQuad {
    fn new<F: glium::backend::Facade>(facade: &F, img: &image::DynamicImage) -> Result<ImageQuad> {
        let vertex_shader_src = resource_str!("shaders/quad_vert.glsl");
        let fragment_shader_src = resource_str!("shaders/color_coords_frag.glsl");

        let program =
            glium::Program::from_source(facade, &vertex_shader_src, &fragment_shader_src, None)
                .context("no program")?;

        let image =
            glium::texture::RawImage2d::from_raw_rgba(img.to_rgba8().into_raw(), img.dimensions());
        let texture = glium::texture::Texture2d::with_mipmaps(
            facade,
            image,
            glium::texture::MipmapsOption::AutoGeneratedMipmaps,
        )
        .unwrap();

        Ok(ImageQuad { texture, program })
    }
}

impl Renderable for ImageQuad {
    fn update(&mut self, _delta: std::time::Duration) -> Result<()> {
        Ok(())
    }

    fn render(&self, frame: &mut Frame) -> Result<(), glium::DrawError> {
        self.custom_render(frame, &Default::default())
    }

    fn custom_render(
        &self,
        frame: &mut Frame,
        params: &glium::draw_parameters::DrawParameters,
    ) -> Result<(), glium::DrawError> {
        let uniforms = uniform! {
        image: &self.texture };

        frame.draw(
            glium::vertex::EmptyVertexAttributes { len: 6 },
            glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList),
            &self.program,
            &uniforms,
            params,
        )?;

        Ok(())
    }
}

fn main() -> Result<()> {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let mut inabox = Box::new(RedTriangle::new(&display));
    let mut last_time = std::time::Instant::now();

    let asset = resource!("assets/D18.png");
    let img = load_image(asset.as_bytes())?;
    make_things_from_image(&display, &img).unwrap();

    let quad = ImageQuad::new(&display, &img).expect("must construct");

    event_loop.run(move |ev, _, control_flow| {
        let now = std::time::Instant::now();
        let delta = now - last_time;
        last_time = now;
        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);

        let mut quad_params = glium::DrawParameters {
            ..Default::default()
        };
        quad_params.backface_culling =
            glium::draw_parameters::BackfaceCullingMode::CullCounterClockwise;

        quad_params.viewport = Some(glium::Rect {
            left: 0,
            bottom: 0,
            width: 100,
            height: 100,
        });
        //quad_params.viewport = Some(glium::Rect {left:0, bottom:0,width:100,height:100});

        quad.custom_render(&mut target, &quad_params).unwrap();
        //quad.render(&mut target ).unwrap();

        //        inabox.update(delta).context("must update").unwrap();
        inabox.render(&mut target).context("render error").unwrap();

        target.finish().unwrap();

        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        if let glutin::event::Event::WindowEvent { event, .. } = ev {
            match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit
                }
                glutin::event::WindowEvent::Resized(_) => (),
                glutin::event::WindowEvent::MouseInput { .. } => (),
                glutin::event::WindowEvent::MouseWheel { .. } => (),
                glutin::event::WindowEvent::KeyboardInput { .. } => (),

                _ => (),
            }
        }
    });
}
