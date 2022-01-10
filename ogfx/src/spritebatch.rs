use crevice::std140::{AsStd140, Std140};
use slotmap::SlotMap;

use crate::{transform, ArcTexture, Buffer, GraphicsContext, RenderContext, Renderable, Transform};

pub type SpriteIdx = slotmap::DefaultKey;

pub struct SpriteBatch {
    pub texture: ArcTexture,
    pub instance_buffer: wgpu::Buffer,
    pub view_buffer: Buffer,
    pub view_binding: wgpu::BindGroup,

    view: Transform,
    transforms: SlotMap<SpriteIdx, Transform>,
    dirty: bool,
}

impl SpriteBatch {
    pub fn new(gfx: &GraphicsContext, texture: ArcTexture, capacity: usize) -> Self {
        let instance_buffer = gfx.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: transform::RawTransform::packed_size() * capacity as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let view = Transform::default();
        let view_buffer = Buffer::new_with_alignable_data(
            gfx,
            &[view.as_matrix()],
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        );
        let view_binding = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &gfx.view_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_buffer.buffer.as_entire_binding(),
            }],
            label: None,
        });

        SpriteBatch {
            texture,
            instance_buffer,
            view_buffer,
            view_binding,
            view,

            transforms: SlotMap::with_capacity(capacity),
            dirty: false,
        }
    }

    pub fn get_view_mut(&mut self) -> &mut Transform {
        self.dirty = true;
        &mut self.view
    }

    pub fn remove(&mut self, key: SpriteIdx) {
        self.dirty = true;
        self.transforms.remove(key);
    }

    pub fn insert(&mut self, transform: Transform) -> SpriteIdx {
        self.dirty = true;
        self.transforms.insert(transform)
    }

    pub fn get(&self, key: SpriteIdx) -> Option<&Transform> {
        self.transforms.get(key)
    }

    pub fn get_mut(&mut self, key: SpriteIdx) -> Option<&mut Transform> {
        self.dirty = true;
        self.transforms.get_mut(key)
    }

    fn refresh_gpu_buffer(&mut self, gfx: &GraphicsContext) {
        gfx.queue.write_buffer(
            &self.view_buffer.buffer,
            0,
            self.view.as_matrix().as_std140().as_bytes(),
        );
        for (idx, transform) in self.into_iter().enumerate() {
            gfx.queue.write_buffer(
                &self.instance_buffer,
                transform::RawTransform::packed_size() * idx as u64,
                transform.as_matrix().as_std140().as_bytes(),
            );
        }
    }

    pub fn update(&mut self, gfx: &GraphicsContext) {
        if self.dirty {
            self.refresh_gpu_buffer(gfx);
            self.dirty = false;
        }
    }
}

impl<'a> IntoIterator for &'a SpriteBatch {
    type Item = &'a Transform;

    type IntoIter = slotmap::basic::Values<'a, SpriteIdx, Transform>;

    fn into_iter(self) -> Self::IntoIter {
        self.transforms.values()
    }
}

impl Renderable for SpriteBatch {
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
        pass.draw_indexed(
            0..self.texture.index_buffer.element_count as _,
            0,
            0..self.transforms.len() as _,
        );
    }
}
