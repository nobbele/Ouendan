use super::GraphicsContext;
use crate::{Buffer, Vertex};
use image::GenericImageView;
use wgpu::util::DeviceExt;

pub struct RawTextureData {
    pub data: Vec<u8>,
    pub size: cgmath::Vector2<u32>,
}

impl RawTextureData {
    pub fn from_raw_image(image_data: &[u8]) -> Self {
        let img = image::load_from_memory(image_data).unwrap();
        let dimensions = img.dimensions();

        RawTextureData {
            data: img.into_rgba8().into_raw(),
            size: cgmath::vec2(dimensions.0, dimensions.1),
        }
    }
}

pub struct RawTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,

    pub bind_group: wgpu::BindGroup,
}

impl RawTexture {
    pub fn from_wgpu_texture(gfx: &GraphicsContext, texture: wgpu::Texture) -> Self {
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = gfx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &gfx.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        RawTexture {
            texture,
            view,
            sampler,
            bind_group,
        }
    }
    pub fn from_rgba8(
        gfx: &GraphicsContext,
        data: &[u8],
        size: cgmath::Vector2<u32>,
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture_size = wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        };
        let texture = gfx.device.create_texture_with_data(
            &gfx.queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING,
            },
            data,
        );

        RawTexture::from_wgpu_texture(gfx, texture)
    }
}

pub struct Texture {
    pub raw: RawTexture,
    pub size: cgmath::Vector2<u32>,

    // Could be reused but would just be annoying to deal with
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
}

impl Texture {
    pub fn from_raw_texture(
        gfx: &GraphicsContext,
        raw: RawTexture,
        size: cgmath::Vector2<u32>,
    ) -> Self {
        let half_extent = size.cast::<f32>().unwrap() / 2.0;
        let vertex_buffer = Buffer::new_with_data(
            gfx,
            &[
                Vertex {
                    position: cgmath::vec2(-half_extent.x, half_extent.y),
                    uv: cgmath::vec2(0.0, 1.0),
                },
                Vertex {
                    position: cgmath::vec2(-half_extent.x, -half_extent.y),
                    uv: cgmath::vec2(0.0, 0.0),
                },
                Vertex {
                    position: cgmath::vec2(half_extent.x, -half_extent.y),
                    uv: cgmath::vec2(1.0, 0.0),
                },
                Vertex {
                    position: cgmath::vec2(half_extent.x, half_extent.y),
                    uv: cgmath::vec2(1.0, 1.0),
                },
            ],
            wgpu::BufferUsages::VERTEX,
        );
        let index_buffer =
            Buffer::new_with_data::<u16>(gfx, &[0, 1, 2, 2, 3, 0], wgpu::BufferUsages::INDEX);

        Texture {
            raw,
            size,
            vertex_buffer,
            index_buffer,
        }
    }

    pub fn new(gfx: &GraphicsContext, image_data: &[u8], format: wgpu::TextureFormat) -> Self {
        let tex_data = RawTextureData::from_raw_image(image_data);
        let raw = RawTexture::from_rgba8(gfx, &tex_data.data, tex_data.size, format);

        Texture::from_raw_texture(gfx, raw, tex_data.size)
    }
}
