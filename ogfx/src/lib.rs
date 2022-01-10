use num_traits::NumCast;

pub mod context;
pub type GraphicsContext = context::Context;
pub type RenderContext<'a> = context::RenderContext<'a>;

pub mod pipeline;

pub use pipeline::Pipeline;

pub mod shader;
pub use shader::Shader;

pub mod vertex;
pub use vertex::Vertex;

pub mod buffer;
pub use buffer::Buffer;

pub mod texture;
pub use texture::Texture;
pub type ArcTexture = std::sync::Arc<Texture>;

pub mod transform;
pub use transform::Transform;

pub mod spritebatch;
pub use spritebatch::SpriteBatch;

pub mod sprite;
pub use sprite::Sprite;

use transform::RawTransform;

#[derive(Debug, Clone, Copy)]
pub struct Rect<T> {
    pub position: cgmath::Vector2<T>,
    pub size: cgmath::Vector2<T>,
}

impl<T> Rect<T> {
    pub fn new(x: T, y: T, w: T, h: T) -> Self {
        Rect {
            position: cgmath::vec2(x, y),
            size: cgmath::vec2(w, h),
        }
    }
    pub fn cast<U>(self) -> Rect<U>
    where
        T: NumCast + Copy,
        U: NumCast + Copy,
    {
        Rect {
            position: self.position.cast::<U>().unwrap(),
            size: self.size.cast::<U>().unwrap(),
        }
    }
}

pub trait Renderable {
    fn render<'data>(&'data self, rctx: &RenderContext<'data>, pass: &mut wgpu::RenderPass<'data>);
}

pub fn instance_matrix_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    use std::mem;
    wgpu::VertexBufferLayout {
        array_stride: RawTransform::packed_size(),
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                shader_location: 4,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                shader_location: 6,
                format: wgpu::VertexFormat::Float32x4,
            },
        ],
    }
}
