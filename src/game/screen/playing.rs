use ogfx::{Buffer, Rect, RenderContext, Renderable, Sprite, Transform};
use slotmap::SlotMap;

use crate::{
    game::{chart, graphics::slider, ChartProgress, GameContext, LogLayer},
    job::{spawn_job, JobHandle},
    llog, math,
};

use super::{Screen, Updatable};

pub struct PlayingResources {
    sound: kira::sound::Sound,
    beatmap: osu_parser::Beatmap,
}

#[derive(Clone, Copy)]
pub enum VisibleHitObjectRef {
    Circle {
        tinted: slotmap::DefaultKey,
        overlay: slotmap::DefaultKey,
        approach: slotmap::DefaultKey,
    },
    Slider {
        tinted: slotmap::DefaultKey,
        overlay: slotmap::DefaultKey,
        approach: slotmap::DefaultKey,
        slider: slotmap::DefaultKey,
    },
    Spinner,
}

#[derive(Clone, Copy)]
pub struct VisibleHitObject {
    hitobject_index: usize,
    refs: VisibleHitObjectRef,
}

pub struct PlayingScreen {
    playfield: Sprite,
    tinted: SlotMap<slotmap::DefaultKey, Sprite>,
    overlay: SlotMap<slotmap::DefaultKey, Sprite>,
    slider_bodies: SlotMap<slotmap::DefaultKey, slider::Slider>,
    approach: SlotMap<slotmap::DefaultKey, Sprite>,

    visible_objects: SlotMap<slotmap::DefaultKey, VisibleHitObject>,
    #[allow(dead_code)]
    playfield_projection_buffer: Buffer,
    playfield_projection_binding: wgpu::BindGroup,

    end_time: f32,
}

impl Screen for PlayingScreen {
    type LoadingResource = PlayingResources;

    fn load(_ctx: std::sync::Arc<GameContext>) -> JobHandle<PlayingResources> {
        spawn_job(|| {
            let sound = kira::sound::Sound::from_file(
                "Mynarco Addiction.mp3",
                kira::sound::SoundSettings::default(),
            )
            .unwrap();
            let beatmap = osu_parser::load_file(
                "positive MAD-crew - Mynarco Addiction (Okoratu) [Ex].osu",
                //"positive MAD-crew - Mynarco Addiction (Okoratu) [test].osu",
                //"positive MAD-crew - Mynarco Addiction (Okoratu) [corner].osu",
                osu_parser::BeatmapParseOptions::default(),
            )
            .unwrap();
            PlayingResources { sound, beatmap }
        })
    }

    fn new(ctx: &GameContext, loading_res: PlayingResources) -> Self {
        let mut sound_handle = ctx
            .audio
            .lock()
            .unwrap()
            .add_sound(loading_res.sound)
            .unwrap();
        let instance_handle = sound_handle
            .play(kira::instance::InstanceSettings::default().playback_rate(1.0))
            .unwrap();

        ctx.set_song(instance_handle);
        let (chart_info, chart_data) = chart::load_osu_beatmap(&loading_res.beatmap);

        let end_time = chart_data
            .objects
            .iter()
            .map(|el| el.end_time())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        ctx.set_chart_info(chart_info);
        ctx.set_chart_data(chart_data);
        ctx.set_chart_progress(ChartProgress {
            pending_start: 0,
            combo: 0,
            progress: 0.0,
        });

        println!("Playing chart '{:#?}'", ctx.chart().as_ref().unwrap());

        let game_resources = ctx.game_resources.lock().unwrap();
        let game_resources = game_resources.as_ref().unwrap();

        let playfield = Sprite::new(
            &ctx.gfx,
            game_resources.playfield.clone(),
            Transform {
                position: cgmath::vec2(
                    ctx.gfx.dimensions.x as f32 / 2.0,
                    ctx.gfx.dimensions.y as f32 / 2.0,
                ),
                layer: 0,
                scale: cgmath::vec2(
                    ctx.gfx.dimensions.x as f32 / 2732.0,
                    ctx.gfx.dimensions.y as f32 / 1572.0,
                ),
                rotation: cgmath::Rad(0.0),
                source: Rect::new(0.0, 0.0, 1.0, 1.0),
            },
        );

        let playfield_projection_buffer = Buffer::new_with_alignable_data(
            &ctx.gfx,
            &[ctx.gfx.new_projection_transform(Transform {
                position: cgmath::vec2(
                    ctx.gfx.dimensions.x as f32 / 2.0,
                    ctx.gfx.dimensions.y as f32 / 2.0,
                ),
                scale: cgmath::vec2(1.18, 1.18),
                ..Default::default()
            })],
            wgpu::BufferUsages::UNIFORM,
        );
        let playfield_projection_binding =
            ctx.gfx
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &ctx.gfx.proj_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: playfield_projection_buffer.buffer.as_entire_binding(),
                    }],
                    label: None,
                });

        PlayingScreen {
            playfield,

            tinted: SlotMap::new(),
            overlay: SlotMap::new(),
            slider_bodies: SlotMap::new(),
            approach: SlotMap::new(),

            visible_objects: SlotMap::new(),

            playfield_projection_buffer,
            playfield_projection_binding,

            end_time,
        }
    }
}

impl Updatable for PlayingScreen {
    fn update(&mut self, ctx: &GameContext) {
        let song = ctx.song();
        let chart = ctx.chart();
        let chart_data = ctx.chart_data();
        let chart_progress = ctx.chart_progress();
        if song.is_none() || chart.is_none() || chart_data.is_none() || chart_progress.is_none() {
            return;
        }
        let song = song.unwrap();
        let chart = chart.as_ref().unwrap();
        let chart_data = chart_data.as_ref().unwrap();
        let mut chart_progress = chart_progress.unwrap();

        let song_position = song.position() as f32;

        let display_objects = {
            if let Some(start_idx) = chart_data.objects[chart_progress.pending_start..]
                .iter()
                .position(|obj| song_position >= obj.time - chart.modifiers.approach_seconds())
            {
                let start = start_idx + chart_progress.pending_start;
                // This end index refers to the index of the object after the last one to be displayed.
                // This has to pass at least one object, the start one.
                let end = chart_data.objects[start..]
                    .iter()
                    .position(|obj| song_position < obj.end_time())
                    .unwrap()
                    + start
                    + 1;
                start..end.min(chart_data.objects.len())
            } else {
                chart_progress.pending_start..chart_progress.pending_start
            }
        };
        chart_progress.pending_start = display_objects.end;

        let active_object = {
            let mut it = chart_data
                .objects
                .iter()
                .enumerate()
                .filter(|(_, obj)| song_position >= obj.time && song_position < obj.end_time())
                .map(|(idx, _)| idx);
            it.next().map(|v| {
                let next = it.next();
                assert_eq!(
                    next,
                    None,
                    "Multiple active objects. (2B!) ({}ms)",
                    chart_data.objects[next.unwrap()].time * 1000.0
                );
                v
            })
        };

        if let Some(i) = active_object {
            llog!(ctx, LogLayer::Playfield, "acitve_object (index {})", i);
        }

        let mut to_remove = vec![];

        for (idx, visible_hitobject) in self.visible_objects.iter() {
            let hitobject = &chart_data.objects[visible_hitobject.hitobject_index];
            if song_position > hitobject.end_time() {
                llog!(
                    ctx,
                    LogLayer::Playfield,
                    "Despawning! (time={}ms end_time={}ms) at {}",
                    hitobject.time * 1000.0,
                    hitobject.end_time() * 1000.0,
                    song_position
                );
                to_remove.push(idx);
                continue;
            }
            if let VisibleHitObjectRef::Circle { approach, .. }
            | VisibleHitObjectRef::Slider { approach, .. } = visible_hitobject.refs
            {
                let scale = math::clamped_remap(
                    hitobject.time - chart.modifiers.approach_seconds(),
                    hitobject.time,
                    0.5,
                    0.125,
                    song_position,
                );
                self.approach[approach].get_transform_mut().scale = cgmath::vec2(scale, scale);
                self.approach[approach].update(&ctx.gfx);
            }
        }

        for idx in to_remove {
            chart_progress.combo += 1;

            let visible_hitobject = self.visible_objects.remove(idx).unwrap();
            match visible_hitobject.refs {
                VisibleHitObjectRef::Circle {
                    tinted,
                    overlay,
                    approach,
                } => {
                    self.tinted.remove(tinted);
                    self.overlay.remove(overlay);
                    self.approach.remove(approach);
                }
                VisibleHitObjectRef::Slider {
                    tinted,
                    overlay,
                    approach,
                    slider,
                } => {
                    self.tinted.remove(tinted);
                    self.overlay.remove(overlay);
                    self.approach.remove(approach);
                    self.slider_bodies.remove(slider);
                }
                _ => panic!(),
            }
        }

        let game_resources = ctx.game_resources.lock().unwrap();
        let game_resources = game_resources.as_ref().unwrap();

        for display_object in display_objects {
            let hitobject = &chart_data.objects[display_object];
            llog!(
                ctx,
                LogLayer::Playfield,
                "Spawning! (time={}ms end_time={}ms) at {}",
                hitobject.time * 1000.0,
                hitobject.end_time() * 1000.0,
                song_position
            );
            let trans = Transform {
                position: cgmath::vec2(hitobject.position.x, hitobject.position.y),
                scale: cgmath::vec2(0.125, 0.125),
                ..Default::default()
            };
            let tinted = self.tinted.insert(Sprite::new(
                &ctx.gfx,
                game_resources.hitobject_atlas.texture.clone(),
                Transform {
                    source: game_resources.hitobject_atlas.sub_textures["tinted"].cast(),
                    ..trans
                },
            ));
            let overlay = self.overlay.insert(Sprite::new(
                &ctx.gfx,
                game_resources.hitobject_atlas.texture.clone(),
                Transform {
                    source: game_resources.hitobject_atlas.sub_textures["overlay"].cast(),
                    ..trans
                },
            ));
            let approach = self.approach.insert(Sprite::new(
                &ctx.gfx,
                game_resources.hitobject_atlas.texture.clone(),
                Transform {
                    source: game_resources.hitobject_atlas.sub_textures["approach"].cast(),
                    ..trans
                },
            ));
            match &hitobject.data {
                chart::HitObjectData::Circle => {
                    self.visible_objects.insert(VisibleHitObject {
                        hitobject_index: display_object,
                        refs: VisibleHitObjectRef::Circle {
                            tinted,
                            overlay,
                            approach,
                        },
                    });
                }
                chart::HitObjectData::Slider(slider) => {
                    let slider = self.slider_bodies.insert(slider::Slider::new(
                        &ctx.gfx,
                        slider.curve_type,
                        hitobject.position,
                        &slider.control_points,
                        &game_resources.hitobject_atlas,
                        "track",
                    ));
                    self.visible_objects.insert(VisibleHitObject {
                        hitobject_index: display_object,
                        refs: VisibleHitObjectRef::Slider {
                            tinted,
                            overlay,
                            approach,
                            slider,
                        },
                    });
                }
            }
        }

        chart_progress.progress = song_position / self.end_time;

        ctx.set_chart_progress(chart_progress);
    }
}

impl Renderable for PlayingScreen {
    fn render<'data>(&'data self, rctx: &RenderContext<'data>, pass: &mut wgpu::RenderPass<'data>) {
        rctx.with_projection(&self.playfield_projection_binding, pass, |pass| {
            for visible_hitobject in self.visible_objects.values().copied() {
                match visible_hitobject.refs {
                    VisibleHitObjectRef::Circle {
                        tinted,
                        overlay,
                        approach: _,
                    } => {
                        self.tinted[tinted].render(rctx, pass);
                        self.overlay[overlay].render(rctx, pass);
                    }
                    VisibleHitObjectRef::Slider {
                        tinted,
                        overlay,
                        slider,
                        approach: _,
                    } => {
                        self.slider_bodies[slider].render(rctx, pass);
                        self.tinted[tinted].render(rctx, pass);
                        self.overlay[overlay].render(rctx, pass);
                    }
                    _ => panic!(),
                }
            }

            for visible_hitobject in self.visible_objects.values().copied() {
                match visible_hitobject.refs {
                    VisibleHitObjectRef::Circle { approach, .. } => {
                        self.approach[approach].render(rctx, pass);
                    }
                    VisibleHitObjectRef::Slider { approach, .. } => {
                        self.approach[approach].render(rctx, pass);
                    }
                    _ => panic!(),
                }
            }
        });

        self.playfield.render(rctx, pass);
    }
}
