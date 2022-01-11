#![feature(drain_filter)]

use std::collections::HashMap;

use futures::task::SpawnExt;
use iced_winit::winit;
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
    job::spawn_job,
};
use ogfx::{Buffer, Renderable, Transform};

pub mod game;
//pub mod graphics;
pub mod job;
pub mod math;

pub type ArcLock<T> = std::sync::Arc<std::sync::RwLock<T>>;

pub struct Controls {
    pub background_color: iced::Color,
    sliders: [iced::slider::State; 3],
}

#[derive(Debug, Clone)]
pub enum Message {
    BackgroundColorChanged(iced::Color),
}

impl Controls {
    pub fn new() -> Controls {
        Controls {
            background_color: iced::Color::BLACK,
            sliders: Default::default(),
        }
    }
}

impl iced_winit::Program for Controls {
    type Renderer = iced_wgpu::Renderer;
    type Message = Message;

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Message::BackgroundColorChanged(color) => {
                self.background_color = color;
            }
        }

        iced::Command::none()
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        let [r, g, b] = &mut self.sliders;
        let background_color = self.background_color;

        let sliders = iced::Row::new()
            .width(iced::Length::Units(500))
            .spacing(20)
            .push(
                iced::Slider::new(r, 0.0..=1.0, background_color.r, move |r| {
                    Message::BackgroundColorChanged(iced::Color {
                        r,
                        ..background_color
                    })
                })
                .step(0.01),
            )
            .push(
                iced::Slider::new(g, 0.0..=1.0, background_color.g, move |g| {
                    Message::BackgroundColorChanged(iced::Color {
                        g,
                        ..background_color
                    })
                })
                .step(0.01),
            )
            .push(
                iced::Slider::new(b, 0.0..=1.0, background_color.b, move |b| {
                    Message::BackgroundColorChanged(iced::Color {
                        b,
                        ..background_color
                    })
                })
                .step(0.01),
            );

        iced::Row::new()
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .align_items(iced::Alignment::End)
            .push(
                iced::Column::new()
                    .width(iced::Length::Fill)
                    .align_items(iced::Alignment::End)
                    .push(
                        iced::Column::new()
                            .padding(10)
                            .spacing(10)
                            .push(iced::Text::new("Background color").color(iced::Color::WHITE))
                            .push(sliders)
                            .push(
                                iced::Text::new(format!("{:?}", background_color))
                                    .size(14)
                                    .color(iced::Color::WHITE),
                            ),
                    ),
            )
            .into()
    }
}

fn main() {
    dotenv::dotenv().ok();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::<u32> {
            width: 1024,
            height: 576,
        })
        .build(&event_loop)
        .unwrap();

    let physical_size = window.inner_size();
    let viewport = iced_wgpu::Viewport::with_physical_size(
        iced::Size::new(physical_size.width, physical_size.height),
        window.scale_factor(),
    );
    let mut cursor_position = winit::dpi::PhysicalPosition::new(-1.0, -1.0);
    let mut modifiers = winit::event::ModifiersState::default();
    let mut clipboard = iced_winit::Clipboard::connect(&window);

    let ctx = std::sync::Arc::new(GameContext::new(
        ogfx::context::Context::new(
            &window,
            cgmath::vec2(physical_size.width, physical_size.height),
        ),
        kira::manager::AudioManager::new(kira::manager::AudioManagerSettings::default()).unwrap(),
    ));
    let gfx = &ctx.gfx;

    let shader = ogfx::Shader::new(
        &gfx,
        include_str!("../ogfx/src/shaders/shader.wgsl"),
        "vs_main",
        "fs_main",
    );
    let pipeline = ogfx::Pipeline::new(&gfx, &shader);

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

            let playfield = ogfx::Texture::new(
                &gfx,
                include_bytes!("../resources/ui/playfield.png"),
                wgpu::TextureFormat::Rgba8Unorm,
            );

            progress.fetch_add(PROGRESS, std::sync::atomic::Ordering::SeqCst);

            let mut map = HashMap::new();
            map.insert(
                "tinted".to_owned(),
                ogfx::texture::RawTextureData::from_raw_image(include_bytes!(
                    "../resources/circle/tinted.png"
                )),
            );
            map.insert(
                "overlay".to_owned(),
                ogfx::texture::RawTextureData::from_raw_image(include_bytes!(
                    "../resources/circle/overlay.png"
                )),
            );
            map.insert(
                "track".to_owned(),
                ogfx::texture::RawTextureData::from_raw_image(include_bytes!(
                    "../resources/circle/track.png"
                )),
            );
            map.insert(
                "approach".to_owned(),
                ogfx::texture::RawTextureData::from_raw_image(include_bytes!(
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

    let proj_buffer = Buffer::new_with_alignable_data(
        gfx,
        &[gfx.new_projection_transform(Transform::default())],
        wgpu::BufferUsages::UNIFORM,
    );

    let proj_bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &gfx.proj_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: proj_buffer.buffer.as_entire_binding(),
        }],
        label: None,
    });

    let controls = Controls::new();
    let mut debug = iced_winit::Debug::new();
    let mut renderer = iced_wgpu::Renderer::new(iced_wgpu::Backend::new(
        &gfx.device,
        iced_wgpu::Settings::default(),
        gfx.surface_format,
    ));

    let mut state = iced_winit::program::State::new(
        controls,
        viewport.logical_size(),
        &mut renderer,
        &mut debug,
    );

    event_loop.run(move |event, _target, control_flow| match event {
        winit::event::Event::WindowEvent { event, .. } => {
            match event {
                winit::event::WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                winit::event::WindowEvent::KeyboardInput { input, .. } => {
                    if input.virtual_keycode == Some(winit::event::VirtualKeyCode::Escape)
                        && input.state == winit::event::ElementState::Pressed
                    {
                        let mut song = ctx.song().unwrap();
                        match song.state() {
                            kira::instance::InstanceState::Playing => {
                                song.pause(kira::instance::PauseInstanceSettings {
                                    fade_tween: None,
                                })
                                .unwrap();
                            }
                            kira::instance::InstanceState::Paused(_) => song
                                .resume(kira::instance::ResumeInstanceSettings {
                                    fade_tween: None,
                                    rewind_to_pause_position: false,
                                })
                                .unwrap(),
                            _ => panic!(),
                        }
                    }
                }
                winit::event::WindowEvent::CursorMoved { position, .. } => {
                    cursor_position = position;
                }
                winit::event::WindowEvent::ModifiersChanged(new_modifiers) => {
                    modifiers = new_modifiers;
                }
                _ => {}
            }
            if let Some(event) =
                iced_winit::conversion::window_event(&event, window.scale_factor(), modifiers)
            {
                state.queue_event(event);
            }
        }
        winit::event::Event::MainEventsCleared => {
            if !state.is_queue_empty() {
                let _ = state.update(
                    viewport.logical_size(),
                    iced_winit::conversion::cursor_position(
                        cursor_position,
                        viewport.scale_factor(),
                    ),
                    &mut renderer,
                    &mut clipboard,
                    &mut debug,
                );

                window.request_redraw();
            }

            let program = state.program();

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
                                    /*r: 0.1,
                                    g: 0.2,
                                    b: 0.3,*/
                                    r: program.background_color.r as _,
                                    g: program.background_color.g as _,
                                    b: program.background_color.b as _,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });

                render_pass.set_pipeline(&pipeline.pipeline);
                let rctx = ogfx::context::RenderContext::new();
                rctx.with_initial_projection(&proj_bind_group, &mut render_pass, |pass| {
                    match &current_screen {
                        Some(s) => match s {
                            GameScreen::Playing(s) => s.render(&rctx, pass),
                        },
                        None => {}
                    }
                });
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

            renderer.with_primitives(|backend, primitive| {
                backend.present(
                    &gfx.device,
                    &mut staging_belt,
                    &mut command_encoder,
                    &frame_view,
                    primitive,
                    &viewport,
                    &debug.overlay(),
                );
            });

            staging_belt.finish();

            gfx.queue.submit(std::iter::once(command_encoder.finish()));
            frame.present();

            local_spawner.spawn(staging_belt.recall()).unwrap();
            local_pool.run_until_stalled();
        }
        _ => {}
    })
}
