#![feature(drain_filter)]

use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{
    game::{
        screen::{GameLoadingResource, GameScreen, Screen, Updatable},
        GameContext, GameResources, PlayingScreen,
    },
    graphics::Renderable,
    job::spawn_job,
};

pub mod game;
pub mod graphics;
pub mod job;
pub mod math;

pub type ArcLock<T> = std::sync::Arc<std::sync::RwLock<T>>;

fn main() {
    dotenv::dotenv().ok();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let ctx = std::sync::Arc::new(GameContext::new(
        std::sync::Arc::new(graphics::context::Context::new(&window)),
        kira::manager::AudioManager::new(kira::manager::AudioManagerSettings::default()).unwrap(),
    ));
    let gfx = &ctx.gfx;

    let shader = graphics::Shader::new(
        &gfx,
        include_str!("graphics/shaders/shader.wgsl"),
        "vs_main",
        "fs_main",
    );
    let pipeline = graphics::Pipeline::new(&gfx, &shader);

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
            gfx.dimensions.x as f32,
            gfx.dimensions.y as f32,
            0.0,
            -1.0,
            1.0,
        );

    let proj_buffer =
        graphics::Buffer::new_with_alignable_data(&gfx, &[proj], wgpu::BufferUsages::UNIFORM);

    let proj_bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &gfx.proj_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: proj_buffer.buffer.as_entire_binding(),
        }],
        label: None,
    });

    let mut load_game_resource_job = spawn_job({
        let ctx = ctx.clone();
        move || {
            let gfx = &ctx.gfx;
            let tinted_circle = graphics::Texture::new(
                &gfx,
                include_bytes!("../resources/circle/tinted.png"),
                wgpu::TextureFormat::Rgba8UnormSrgb,
            );
            let overlay_circle = graphics::Texture::new(
                &gfx,
                include_bytes!("../resources/circle/overlay.png"),
                wgpu::TextureFormat::Rgba8UnormSrgb,
            );

            let approach_circle = graphics::Texture::new(
                &gfx,
                include_bytes!("../resources/circle/approach.png"),
                wgpu::TextureFormat::Rgba8UnormSrgb,
            );
            let playfield = graphics::Texture::new(
                &gfx,
                include_bytes!("../resources/ui/playfield.png"),
                wgpu::TextureFormat::Rgba8Unorm,
            );

            GameResources {
                tinted_circle,
                overlay_circle,
                approach_circle,
                playfield,
            }
        }
    });

    drop(gfx);

    let mut current_screen: Option<GameScreen> = None;
    let mut next_scene_resource: Option<GameLoadingResource> = Some(GameLoadingResource::Playing(
        PlayingScreen::load(ctx.clone()),
    ));

    event_loop.run(move |event, _target, control_flow| match event {
        winit::event::Event::WindowEvent { event, .. } => match event {
            winit::event::WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            winit::event::WindowEvent::KeyboardInput { input, .. } => {
                if input.virtual_keycode.unwrap() == winit::event::VirtualKeyCode::Escape {
                    ctx.song()
                        .unwrap()
                        .pause(kira::instance::PauseInstanceSettings { fade_tween: None })
                        .unwrap();
                }
            }
            _ => {}
        },
        winit::event::Event::MainEventsCleared => {
            if load_game_resource_job.finished() {
                if let Some(game_loading_resource) = &mut next_scene_resource {
                    match game_loading_resource {
                        GameLoadingResource::Playing(r) => {
                            if let Some(resource) = r.poll().unwrap() {
                                current_screen =
                                    Some(GameScreen::Playing(PlayingScreen::new(&ctx, resource)));
                                next_scene_resource = None;
                            }
                        }
                    };
                }
            } else {
                if let Some(game_resources) = load_game_resource_job.poll().unwrap() {
                    ctx.set_game_resources(game_resources);
                }
            }

            match &mut current_screen {
                Some(s) => match s {
                    GameScreen::Playing(s) => s.update(&ctx),
                },
                None => {}
            }

            let gfx = &ctx.gfx;

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

                render_pass.set_pipeline(&pipeline.pipeline);
                render_pass.set_bind_group(0, &proj_bind_group, &[]);
                match &current_screen {
                    Some(s) => match s {
                        GameScreen::Playing(s) => s.render(&mut render_pass),
                    },
                    None => {}
                }
            }

            gfx.queue.submit(std::iter::once(command_encoder.finish()));
            frame.present();
        }
        _ => {}
    })
}
