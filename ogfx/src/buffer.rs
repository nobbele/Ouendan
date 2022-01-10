use crevice::std140::{AsStd140, Std140};
use wgpu::util::DeviceExt;

use super::GraphicsContext;

pub struct Buffer {
    pub buffer: wgpu::Buffer,
    pub element_size: usize,
    pub element_count: usize,
}

impl Buffer {
    pub fn new_with_data<T: bytemuck::Pod>(
        gfx: &GraphicsContext,
        data: &[T],
        usage: wgpu::BufferUsages,
    ) -> Self {
        let element_size = std::mem::size_of::<T>();
        let element_count = data.len();
        let buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(data),
                usage,
            });
        Buffer {
            buffer,
            element_size,
            element_count,
        }
    }

    pub fn new_with_alignable_data<T: AsStd140>(
        gfx: &GraphicsContext,
        data: &[T],
        usage: wgpu::BufferUsages,
    ) -> Self {
        let element_size = T::std140_size_static();
        let mut aligned_data = vec![0; element_size * data.len()];
        for (idx, el) in data.iter().enumerate() {
            aligned_data[element_size * idx..element_size * (idx + 1)]
                .copy_from_slice(el.as_std140().as_bytes());
        }

        let element_count = data.len();
        let buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&aligned_data),
                usage,
            });
        Buffer {
            buffer,
            element_size,
            element_count,
        }
    }
}
