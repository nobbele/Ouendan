use std::sync::Arc;

pub mod context;
pub type GraphicsContext = Arc<context::Context>;

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
pub type ArcTexture = Arc<Texture>;

pub mod transform;
pub use transform::Transform;

pub mod spritebatch;
pub use spritebatch::SpriteBatch;

pub trait Renderable {
    fn render<'data>(&'data self, pass: &mut wgpu::RenderPass<'data>);
}

pub fn instance_matrix_desc<'a>() -> wgpu::VertexBufferLayout<'a> {
    use std::mem;
    wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<cgmath::Matrix4<f32>>() as wgpu::BufferAddress,
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
        ],
    }
}