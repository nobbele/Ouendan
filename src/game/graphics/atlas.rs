use std::{collections::HashMap, hash::Hash, num::NonZeroU32};

use atlas_packer::PackSolver;
use ogfx::{self, texture::RawTextureData, GraphicsContext, Rect};

pub struct Atlas<T> {
    pub texture: ogfx::ArcTexture,
    pub sub_textures: HashMap<T, Rect<f32>>,
}

impl<T> Atlas<T>
where
    T: Clone + Hash + Eq,
{
    pub fn new(
        gfx: &GraphicsContext,
        textures: &[(&T, &RawTextureData)],
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let rects = textures
            .iter()
            .map(|tex| tex.1.size.into())
            .collect::<Vec<_>>();
        let solver = PackSolver::new(&rects);
        let pack = solver.solve();
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
        for (idx, &(_, raw)) in textures.iter().enumerate() {
            let pos = pack.output[pack.output.iter().position(|el| el.1 == idx).unwrap()].0;
            gfx.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: pos.x,
                        y: pos.y,
                        z: 0,
                    },
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

        Atlas {
            texture: std::sync::Arc::new(ogfx::Texture::from_raw_texture(
                gfx,
                ogfx::texture::RawTexture::from_wgpu_texture(gfx, texture),
                cgmath::vec2(pack.dimensions.x, pack.dimensions.y),
            )),
            sub_textures: pack
                .output
                .into_iter()
                .map(|(pos, idx)| {
                    (
                        (*textures[idx].0).clone(),
                        Rect {
                            position: cgmath::vec2(
                                pos.x as f32 / pack.dimensions.x as f32,
                                pos.y as f32 / pack.dimensions.y as f32,
                            ),
                            size: cgmath::vec2(
                                rects[idx].x as f32 / pack.dimensions.x as f32,
                                rects[idx].y as f32 / pack.dimensions.y as f32,
                            ),
                        },
                    )
                })
                .collect(),
        }
    }
}
