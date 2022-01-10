use std::{
    cell::Cell,
    sync::{Arc, Mutex},
};

use crate::graphics;
use crevice::std140::{AsStd140, Std140};
use wgpu::util::DeviceExt;
use winit::window::Window;

pub struct Context {
    pub surface: wgpu::Surface,
    pub surface_format: wgpu::TextureFormat,
    pub queue: wgpu::Queue,
    pub device: wgpu::Device,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,

    pub view_bind_group_layout: wgpu::BindGroupLayout,
    pub identity_view_buffer: Arc<wgpu::Buffer>,
    pub identity_view_binding: Arc<wgpu::BindGroup>,

    pub proj_bind_group_layout: wgpu::BindGroupLayout,
    pub proj_transform: Mutex<Cell<graphics::Transform>>,
    pub proj_buffer: wgpu::Buffer,
    pub proj_bind_group: wgpu::BindGroup,

    pub aspect_ratio: f32,
    pub dimensions: cgmath::Vector2<u32>,
}

impl Context {
    pub fn new(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))
        .unwrap();

        let surface_format = surface.get_preferred_format(&adapter).unwrap();
        let inner_size = window.inner_size();

        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: inner_size.width,
                height: inner_size.height,
                present_mode: wgpu::PresentMode::Mailbox,
            },
        );

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: None,
            });

        let proj_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let view_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: None,
            });

        let view_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: graphics::Transform::default()
                .as_matrix()
                .as_std140()
                .as_bytes(),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let view_binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &view_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let dimensions = cgmath::vec2(inner_size.width, inner_size.height);

        let proj_transform = graphics::Transform::default();

        let mut raw = proj_transform.as_matrix();
        raw.matrix = raw.matrix * new_projection_matrix(dimensions.cast().unwrap());
        let proj_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: raw.as_std140().as_bytes(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let proj_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &proj_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: proj_buffer.as_entire_binding(),
            }],
            label: None,
        });

        Context {
            surface,
            surface_format,
            queue,
            device,
            texture_bind_group_layout,
            proj_bind_group_layout,
            view_bind_group_layout,
            identity_view_buffer: Arc::new(view_buffer),
            identity_view_binding: Arc::new(view_binding),
            proj_transform: Mutex::new(Cell::new(proj_transform)),
            proj_buffer,
            proj_bind_group,
            aspect_ratio: inner_size.width as f32 / inner_size.height as f32,
            dimensions,
        }
    }

    pub fn set_projection_transform(&self, transform: graphics::Transform) {
        self.proj_transform.lock().unwrap().set(transform);
        let mut raw = self.proj_transform.lock().unwrap().get().as_matrix();
        raw.matrix = raw.matrix * new_projection_matrix(self.dimensions.cast().unwrap());
        self.queue
            .write_buffer(&self.proj_buffer, 0, raw.as_std140().as_bytes());
    }
    pub fn get_projection_transform(&self) -> graphics::Transform {
        self.proj_transform.lock().unwrap().get()
    }
}

fn new_projection_matrix(dimensions: cgmath::Vector2<f32>) -> cgmath::Matrix4<f32> {
    #[rustfmt::skip]
    pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.0,
        0.0, 0.0, 0.5, 1.0,
    );
    let proj = OPENGL_TO_WGPU_MATRIX
        * cgmath::ortho(
            0.0,
            dimensions.x as f32,
            dimensions.y as f32,
            0.0,
            -1.0,
            1.0,
        );
    proj
}
