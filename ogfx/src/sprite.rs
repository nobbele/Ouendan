use crevice::std140::{AsStd140, Std140};
use wgpu::util::DeviceExt;

use super::{GraphicsContext, Renderable};
use crate::{ArcTexture, RenderContext, Transform};
use std::sync::Arc;

pub struct Sprite {
    texture: ArcTexture,
    transform: Transform,
    instance_buffer: wgpu::Buffer,

    view_binding: Arc<wgpu::BindGroup>,

    dirty: bool,
}

impl Sprite {
    pub fn new(gfx: &GraphicsContext, texture: ArcTexture, transform: Transform) -> Self {
        let instance_buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: transform.as_matrix().as_std140().as_bytes(),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        Sprite {
            texture,
            transform,
            instance_buffer,

            view_binding: gfx.identity_view_binding.clone(),

            dirty: false,
        }
    }

    pub fn get_transform_mut(&mut self) -> &mut Transform {
        self.dirty = true;
        &mut self.transform
    }

    pub fn update(&mut self, gfx: &GraphicsContext) {
        if self.dirty {
            gfx.queue.write_buffer(
                &self.instance_buffer,
                0,
                self.transform.as_matrix().as_std140().as_bytes(),
            );
            self.dirty = false;
        }
    }
}

impl Renderable for Sprite {
    fn render<'data>(
        &'data self,
        _rctx: &RenderContext<'data>,
        pass: &mut wgpu::RenderPass<'data>,
    ) {
        pass.set_bind_group(1, &self.view_binding, &[]);
        pass.set_bind_group(2, &self.texture.raw.bind_group, &[]);
        pass.set_vertex_buffer(0, self.texture.vertex_buffer.buffer.slice(..));
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.set_index_buffer(
            self.texture.index_buffer.buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        pass.draw_indexed(0..self.texture.index_buffer.element_count as _, 0, 0..1);
    }
}
