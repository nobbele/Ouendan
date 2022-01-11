use crate::{transform::RawTransform, Transform};
use crevice::std140::{AsStd140, Std140};
use std::{cell::RefCell, sync::Arc};
use wgpu::util::DeviceExt;

pub struct RenderContext<'a> {
    projection_stack: RefCell<Vec<&'a wgpu::BindGroup>>,
}

impl<'a> RenderContext<'a> {
    pub fn new() -> Self {
        RenderContext {
            projection_stack: RefCell::new(Vec::new()),
        }
    }
    pub fn with_projection(
        &self,
        new: &'a wgpu::BindGroup,
        pass: &mut wgpu::RenderPass<'a>,
        f: impl Fn(&mut wgpu::RenderPass<'a>) -> (),
    ) {
        assert!(self.projection_stack.borrow().len() >= 1);
        self.projection_stack.borrow_mut().push(new);
        pass.set_bind_group(0, new, &[]);
        f(pass);
        self.projection_stack.borrow_mut().pop().unwrap();
        pass.set_bind_group(0, self.projection_stack.borrow().last().unwrap(), &[]);
    }

    pub fn with_initial_projection(
        &self,
        new: &'a wgpu::BindGroup,
        pass: &mut wgpu::RenderPass<'a>,
        f: impl Fn(&mut wgpu::RenderPass<'a>) -> (),
    ) {
        assert!(self.projection_stack.borrow().len() == 0);
        self.projection_stack.borrow_mut().push(new);
        pass.set_bind_group(0, new, &[]);
        f(pass);
        self.projection_stack.borrow_mut().pop().unwrap();
    }
}

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

    pub aspect_ratio: f32,
    pub dimensions: cgmath::Vector2<u32>,
}

impl Context {
    pub fn new(
        window: &impl raw_window_handle::HasRawWindowHandle,
        dimensions: cgmath::Vector2<u32>,
    ) -> Self {
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

        surface.configure(
            &device,
            &wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: dimensions.x,
                height: dimensions.y,
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
            contents: Transform::default().as_matrix().as_std140().as_bytes(),
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
            aspect_ratio: dimensions.x as f32 / dimensions.y as f32,
            dimensions,
        }
    }

    pub fn new_projection_transform(&self, transform: Transform) -> RawTransform {
        let mut raw = transform.as_matrix();
        raw.matrix = new_projection_matrix(self.dimensions.cast().unwrap()) * raw.matrix;
        raw
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
