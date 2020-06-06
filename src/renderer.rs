use std::borrow::Cow;

use glium::index::{NoIndices, PrimitiveType};
use glium::texture::{ClientFormat, RawImage2d, Texture2dDataSource};
use glium::texture::{MipmapsOption, UncompressedUintFormat, UnsignedTexture2d};
use glium::{implement_vertex, uniform, DrawParameters, Program, Rect, Surface, VertexBuffer};

use crate::display as chip8_display;

const VERTEX_SHADER_SOURCE: &str = include_str!("shaders/default.vert");
const FRAGMENT_SHADER_SOURCE: &str = include_str!("shaders/default.frag");

pub struct Renderer<'a> {
    display: glium::Display,
    shader_program: Program,
    vertex_buffer: VertexBuffer<Vertex>,
    draw_params: DrawParameters<'a>,
    initial_render: bool,
}

impl<'a> Renderer<'a> {
    pub fn new(display: glium::Display) -> Self {
        let vertices = [
            Vertex {
                position: [-1.0, -1.0],
            },
            Vertex {
                position: [-1.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0],
            },
            Vertex {
                position: [1.0, 1.0],
            },
        ];
        let vertex_buffer = VertexBuffer::new(&display, &vertices).unwrap();
        let shader_program =
            Program::from_source(&display, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE, None)
                .unwrap();
        Self {
            display,
            vertex_buffer,
            shader_program,
            draw_params: DrawParameters::default(),
            initial_render: true,
        }
    }

    pub fn render(&mut self, chip8_display: &chip8_display::Display) {
        let mut frame = self.display.draw();
        frame.clear_color(0.0, 0.0, 0.0, 1.0);
        let texture = UnsignedTexture2d::with_format(
            &self.display,
            chip8_display.clone(),
            UncompressedUintFormat::U8,
            MipmapsOption::NoMipmap,
        )
        .unwrap();
        frame
            .draw(
                &self.vertex_buffer,
                NoIndices(PrimitiveType::TriangleStrip),
                &self.shader_program,
                &uniform! { texture: &texture },
                &self.draw_params,
            )
            .unwrap();
        frame.finish().unwrap();
        if self.initial_render {
            self.display.gl_window().window().set_visible(true);
            self.initial_render = false;
        }
    }

    pub fn set_viewport(&mut self, viewport: Rect) {
        self.draw_params.viewport = Some(viewport);
    }
}

#[derive(Debug, Clone, Copy)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

impl<'a> Texture2dDataSource<'a> for chip8_display::Display {
    type Data = u8;

    fn into_raw(self) -> RawImage2d<'a, Self::Data> {
        let (width, height) = self.dimensions();
        let data: Vec<_> = self
            .into_inner()
            .into_iter()
            .map(|v| if v { 255 } else { 0 })
            .collect();
        RawImage2d {
            width: width as u32,
            height: height as u32,
            format: ClientFormat::U8,
            data: Cow::from(data),
        }
    }
}
