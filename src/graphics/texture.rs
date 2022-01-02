use std::sync::Arc;

use image::GenericImageView;
use wgpu::util::DeviceExt;

use crate::graphics;

use super::{ArcTexture, GraphicsContext};

pub struct Texture {
    // Do these need to be kept alive?
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,

    pub bind_group: wgpu::BindGroup,

    // Could be reused but would just be annoying to deal with
    pub vertex_buffer: graphics::Buffer,
    pub index_buffer: graphics::Buffer,
}

impl Texture {
    pub fn new(
        gfx: &GraphicsContext,
        image_data: &[u8],
        format: wgpu::TextureFormat,
    ) -> ArcTexture {
        let img = image::load_from_memory(image_data).unwrap();
        let rgba = img.as_rgba8().unwrap();
        let dimensions = img.dimensions();

        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
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
            rgba,
        );
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

        let half_extent = (dimensions.0 as f32 / 2.0, dimensions.1 as f32 / 2.0);
        let vertex_buffer = graphics::Buffer::new_with_data(
            gfx,
            &[
                graphics::Vertex {
                    position: cgmath::vec2(-half_extent.0, half_extent.1),
                    uv: cgmath::vec2(0.0, 1.0),
                },
                graphics::Vertex {
                    position: cgmath::vec2(-half_extent.0, -half_extent.1),
                    uv: cgmath::vec2(0.0, 0.0),
                },
                graphics::Vertex {
                    position: cgmath::vec2(half_extent.0, -half_extent.1),
                    uv: cgmath::vec2(1.0, 0.0),
                },
                graphics::Vertex {
                    position: cgmath::vec2(half_extent.0, half_extent.1),
                    uv: cgmath::vec2(1.0, 1.0),
                },
            ],
            wgpu::BufferUsages::VERTEX,
        );
        let index_buffer = graphics::Buffer::new_with_data::<u16>(
            gfx,
            &[0, 1, 2, 2, 3, 0],
            wgpu::BufferUsages::INDEX,
        );

        Arc::new(Texture {
            texture,
            view,
            sampler,
            bind_group,
            vertex_buffer,
            index_buffer,
        })
    }
}
