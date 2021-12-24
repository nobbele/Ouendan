use lyon::lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers,
};
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub mod graphics;
pub mod math;

fn main() {
    dotenv::dotenv().ok();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let gfx = graphics::Context::new(&window);
    let shader = graphics::Shader::new(
        &gfx,
        include_str!("graphics/shaders/shader.wgsl"),
        "vs_main",
        "fs_main",
    );
    let shader_pipeline = graphics::Pipeline::new(&gfx, &shader);

    let mut geometry: VertexBuffers<graphics::Vertex, u16> = VertexBuffers::new();

    let mut tess = FillTessellator::new();
    let radius = 150.0;
    tess.tessellate_circle(
        lyon::math::point(0.0, 0.0),
        radius,
        &FillOptions::default().with_tolerance(0.001),
        &mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| graphics::Vertex {
            position: cgmath::vec2(vertex.position().x, vertex.position().y),
            uv: cgmath::vec2(
                math::remap(-radius, radius, 0.0, 1.0, vertex.position().x),
                math::remap(-radius, radius, 1.0, 0.0, vertex.position().y),
            ),
        }),
    )
    .unwrap();

    let vertex_buffer = graphics::Buffer::new_with_alignable_data(
        &gfx,
        &geometry.vertices,
        wgpu::BufferUsages::VERTEX,
    );
    let index_buffer =
        graphics::Buffer::new_with_data::<u16>(&gfx, &geometry.indices, wgpu::BufferUsages::INDEX);

    let texture = graphics::Texture::new(
        &gfx,
        include_bytes!("../happy-tree.png"),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    let view = graphics::Transform {
        position: cgmath::vec2(gfx.dimensions.x as f32 / 2.0, gfx.dimensions.y as f32 / 2.0),
        scale: cgmath::vec2(1.0, 1.0),
    }
    .as_matrix(&gfx);

    let view_buffer =
        graphics::Buffer::new_with_alignable_data(&gfx, &[view], wgpu::BufferUsages::UNIFORM);

    let view_bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &gfx.view_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: view_buffer.buffer.as_entire_binding(),
        }],
        label: Some("camera_bind_group"),
    });

    event_loop.run(move |event, _target, control_flow| match event {
        winit::event::Event::WindowEvent { event, .. } => match event {
            winit::event::WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        },
        winit::event::Event::MainEventsCleared => {
            let frame = gfx.surface.get_current_texture().unwrap();
            let frame_view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut command_encoder = gfx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
            {
                let mut render_pass =
                    command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachment {
                            view: &frame_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });

                render_pass.set_pipeline(&shader_pipeline.pipeline);
                render_pass.set_bind_group(0, &texture.bind_group, &[]);
                render_pass.set_bind_group(1, &view_bind_group, &[]);
                render_pass.set_vertex_buffer(0, vertex_buffer.buffer.slice(..));
                render_pass
                    .set_index_buffer(index_buffer.buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..index_buffer.element_count as u32, 0, 0..1);
            }

            gfx.queue.submit(std::iter::once(command_encoder.finish()));
            frame.present();
        }
        _ => {}
    })
}
