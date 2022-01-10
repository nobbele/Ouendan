#![feature(drain_filter)]

use std::collections::HashMap;

use futures::task::SpawnExt;
use num_traits::NumCast;
use wgpu_glyph::{ab_glyph, GlyphBrushBuilder, Section, Text};
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::{
    game::{
        screen::{playing::PlayingScreen, GameLoadingResource, GameScreen, Screen, Updatable},
        GameContext, GameResources,
    },
    graphics::Renderable,
    job::spawn_job,
};

pub mod game;
pub mod graphics;
pub mod job;
pub mod math;

pub type ArcLock<T> = std::sync::Arc<std::sync::RwLock<T>>;

#[derive(Debug, Clone, Copy)]
pub struct Rect<T> {
    pub position: cgmath::Vector2<T>,
    pub size: cgmath::Vector2<T>,
}

impl<T> Rect<T> {
    pub fn new(x: T, y: T, w: T, h: T) -> Self {
        Rect {
            position: cgmath::vec2(x, y),
            size: cgmath::vec2(w, h),
        }
    }
    pub fn cast<U>(self) -> Rect<U>
    where
        T: NumCast + Copy,
        U: NumCast + Copy,
    {
        Rect {
            position: self.position.cast::<U>().unwrap(),
            size: self.size.cast::<U>().unwrap(),
        }
    }
}

fn main() {
    dotenv::dotenv().ok();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let ctx = std::sync::Arc::new(GameContext::new(
        graphics::context::Context::new(&window),
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

    let ui_font =
        ab_glyph::FontArc::try_from_slice(include_bytes!("../Roboto-Regular.ttf")).unwrap();
    let mut glyph_brush =
        GlyphBrushBuilder::using_font(ui_font).build(&gfx.device, gfx.surface_format);
    let mut staging_belt = wgpu::util::StagingBelt::new(1024);

    let mut local_pool = futures::executor::LocalPool::new();
    let local_spawner = local_pool.spawner();

    let progress = std::sync::Arc::new(std::sync::atomic::AtomicU8::new(0));

    let mut load_game_resource_job = spawn_job({
        let ctx = ctx.clone();
        let progress = progress.clone();
        move || {
            const PROGRESS: u8 = 100 / 2;

            let gfx = &ctx.gfx;

            let playfield = graphics::Texture::new(
                &gfx,
                include_bytes!("../resources/ui/playfield.png"),
                wgpu::TextureFormat::Rgba8Unorm,
            );

            progress.fetch_add(PROGRESS, std::sync::atomic::Ordering::SeqCst);

            let mut map = HashMap::new();
            map.insert(
                "tinted".to_owned(),
                graphics::texture::RawTextureData::from_raw_image(include_bytes!(
                    "../resources/circle/tinted.png"
                )),
            );
            map.insert(
                "overlay".to_owned(),
                graphics::texture::RawTextureData::from_raw_image(include_bytes!(
                    "../resources/circle/overlay.png"
                )),
            );
            map.insert(
                "track".to_owned(),
                graphics::texture::RawTextureData::from_raw_image(include_bytes!(
                    "../resources/circle/track.png"
                )),
            );
            map.insert(
                "approach".to_owned(),
                graphics::texture::RawTextureData::from_raw_image(include_bytes!(
                    "../resources/circle/approach.png"
                )),
            );

            let hitobject_atlas = game::graphics::atlas::Atlas::new(
                &gfx,
                map.iter()
                    .map(|(key, value)| (key, value))
                    .collect::<Vec<_>>()
                    .as_slice(),
                wgpu::TextureFormat::Rgba8Unorm,
            );

            progress.fetch_add(PROGRESS, std::sync::atomic::Ordering::SeqCst);

            GameResources {
                hitobject_atlas,
                playfield: std::sync::Arc::new(playfield),
            }
        }
    });

    let mut current_screen: Option<GameScreen> = None;
    let mut next_scene_resource: Option<GameLoadingResource> = Some(GameLoadingResource::Playing(
        PlayingScreen::load(ctx.clone()),
    ));
    //let mut next_scene_resource: Option<GameLoadingResource> = None;

    /*let (_depth_texture, depth_view, _depth_sampler) = {
        let size = wgpu::Extent3d {
            // 2.
            width: gfx.dimensions.x,
            height: gfx.dimensions.y,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        };
        let texture = gfx.device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = gfx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });

        (texture, view, sampler)
    };*/

    event_loop.run(move |event, _target, control_flow| match event {
        winit::event::Event::WindowEvent { event, .. } => match event {
            winit::event::WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            winit::event::WindowEvent::KeyboardInput { input, .. } => {
                if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Escape) {
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
                        /*depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &depth_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        }),*/
                        depth_stencil_attachment: None,
                    });

                render_pass.set_pipeline(&pipeline.pipeline);
                render_pass.set_bind_group(0, &gfx.proj_bind_group, &[]);
                match &current_screen {
                    Some(s) => match s {
                        GameScreen::Playing(s) => s.render(&mut render_pass),
                    },
                    None => {}
                }
            }

            if current_screen.is_none() {
                glyph_brush.queue(Section {
                    screen_position: (gfx.dimensions.x as f32 / 2.0, gfx.dimensions.y as f32 / 2.0),
                    bounds: (gfx.dimensions.x as f32, gfx.dimensions.y as f32),
                    text: vec![Text::new(&format!(
                        "Loading... {}%",
                        progress.load(std::sync::atomic::Ordering::SeqCst)
                    ))
                    .with_color([1.0, 1.0, 1.0, 1.0])
                    .with_scale(40.0)],
                    layout: wgpu_glyph::Layout::default_single_line()
                        .h_align(wgpu_glyph::HorizontalAlign::Center)
                        .v_align(wgpu_glyph::VerticalAlign::Center),
                    ..Section::default()
                });
            }

            glyph_brush
                .draw_queued(
                    &gfx.device,
                    &mut staging_belt,
                    &mut command_encoder,
                    &frame_view,
                    gfx.dimensions.x,
                    gfx.dimensions.y,
                )
                .unwrap();

            staging_belt.finish();

            gfx.queue.submit(std::iter::once(command_encoder.finish()));
            frame.present();

            local_spawner.spawn(staging_belt.recall()).unwrap();
            local_pool.run_until_stalled();
        }
        _ => {}
    })
}
