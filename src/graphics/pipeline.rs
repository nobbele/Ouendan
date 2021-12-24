use wgpu::{PipelineLayoutDescriptor, RenderPipelineDescriptor};

use crate::graphics;

use super::Shader;

pub struct Pipeline {
    pub pipeline: wgpu::RenderPipeline,
}

impl Pipeline {
    pub fn new(gfx: &graphics::Context, shader: &Shader) -> Self {
        let layout = gfx
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&gfx.texture_bind_group_layout, &gfx.view_bind_group_layout],
                push_constant_ranges: &[],
            });
        let pipeline = gfx
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader.module,
                    entry_point: shader.vs_name,
                    buffers: &[crate::graphics::Vertex::buffer_layout()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader.module,
                    entry_point: shader.fs_name,
                    targets: &[wgpu::ColorTargetState {
                        format: gfx.surface_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        Pipeline { pipeline }
    }
}
