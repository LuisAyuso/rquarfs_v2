use anyhow::{Context, Result};
use glium::*;

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
        let vertex_buffer = glium::VertexBuffer::persistent(facade, &shape).unwrap();
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

        RedTriangle {
            vertices: vertex_buffer,
            program: program,
        }
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
            .context("could not read opengl buffer")?;

        data[0].position[0] = (delta.as_nanos() as f32).cos();
        self.vertices.write(&data);

        Ok(())
    }
}

fn main() {
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new();
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let mut inabox = Box::new(RedTriangle::new(&display));
    let mut last_time = std::time::Instant::now();

    event_loop.run(move |ev, _, control_flow| {
        let now = std::time::Instant::now();
        let delta = now - last_time;
        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        inabox.update(delta).context("must update").unwrap();
        inabox.render(&mut target).context("render error").unwrap();
        target.finish().unwrap();

        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);
        match ev {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                _ => return,
            },
            _ => (),
        }
    });
}
