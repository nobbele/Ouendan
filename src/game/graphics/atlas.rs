use std::num::NonZeroU32;

use crate::graphics::{texture::RawTextureData, ArcTexture, GraphicsContext};
use atlas_packer::PackSolver;

pub struct AtlasSubTexture {
    position: cgmath::Vector2<f32>,
    size: cgmath::Vector2<f32>,
    atlas: ArcTexture,
}

pub struct Atlas {}

impl Atlas {
    pub fn new(
        gfx: &GraphicsContext,
        textures: &[&RawTextureData],
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let rects = textures
            .iter()
            .map(|tex| tex.size.into())
            .collect::<Vec<_>>();
        let solver = PackSolver::new(&rects);
        let pack = solver.solve();
        let texture_dimension = cgmath::vec2(
            pack.dimensions.x.next_power_of_two(),
            pack.dimensions.y.next_power_of_two(),
        );
        let texture = gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: pack.dimensions.x,
                height: pack.dimensions.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        });
        for (idx, &raw) in textures.iter().enumerate() {
            gfx.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 1,
                    origin: wgpu::Origin3d::default(),
                    aspect: wgpu::TextureAspect::All,
                },
                &raw.data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(NonZeroU32::new(raw.size.x * 4).unwrap()),
                    rows_per_image: Some(NonZeroU32::new(raw.size.y).unwrap()),
                },
                wgpu::Extent3d {
                    width: raw.size.x,
                    height: raw.size.y,
                    depth_or_array_layers: 1,
                },
            )
        }

        Atlas {}
    }
}
