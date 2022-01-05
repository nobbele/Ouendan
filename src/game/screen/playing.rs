use osu_types::SpecificHitObject;

use crate::{
    game::{
        chart::{self, Chart, ChartData},
        graphics::{hitcircle_batch::HitCircleBatch, slider},
        ChartProgress, GameContext,
    },
    graphics::{Renderable, SpriteBatch, Transform},
    job::{spawn_job, JobHandle},
    math,
};

use super::{Screen, Updatable};

pub struct PlayingResources {
    sound: kira::sound::Sound,
    beatmap: osu_parser::Beatmap,
}

pub struct PlayingScreen {
    hitcircle_batch: HitCircleBatch,
    playfield_batch: SpriteBatch,

    visible_sliders: Vec<(usize, slider::Slider)>,
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
                osu_parser::BeatmapParseOptions::default(),
            )
            .unwrap();
            PlayingResources { sound, beatmap }
        })
    }

    fn new(ctx: &GameContext, loading_res: PlayingResources) -> Self {
        let game_resources = ctx.game_resources();

        let mut sound_handle = ctx
            .audio
            .lock()
            .unwrap()
            .borrow_mut()
            .add_sound(loading_res.sound)
            .unwrap();
        let instance_handle = sound_handle
            .play(kira::instance::InstanceSettings::default().playback_rate(1.0))
            .unwrap();

        ctx.set_song(instance_handle);
        ctx.set_chart(Chart {
            title: loading_res.beatmap.info.metadata.title.clone(),
            modifiers: chart::Modifiers { approach_rate: 5.5 },
        });
        let opx_per_secs = loading_res
            .beatmap
            .timing_points
            .iter()
            .scan(0.0, |bpm, tp| {
                Some((
                    tp.time,
                    if tp.uninherited {
                        *bpm = tp.beat_length;
                        let bps = *bpm / 60.0;
                        (100.0 * bps) / 1.0
                    } else {
                        let multiplier = -1.0 / (tp.beat_length / 100.0);
                        let sv = loading_res.beatmap.info.difficulty.slider_multiplier * multiplier;
                        let bps = *bpm / 60.0;
                        (100.0 * bps) / sv
                    },
                ))
            })
            .collect::<Vec<_>>();
        ctx.set_chart_data(ChartData {
            objects: loading_res
                .beatmap
                .hit_objects
                .iter()
                .map(|hit_object| chart::HitObject {
                    position: cgmath::vec2(
                        hit_object.position.0 as f32,
                        hit_object.position.1 as f32,
                    ),
                    time: (hit_object.time as f32) / 1000.0,
                    data: match &hit_object.specific {
                        SpecificHitObject::Circle => chart::HitObjectData::Circle,
                        SpecificHitObject::Slider {
                            curve_type,
                            curve_points,
                            slides,
                            length,
                        } => {
                            let opx_per_sec = opx_per_secs
                                .iter()
                                .find(|p| p.0 >= hit_object.time as i32)
                                .unwrap()
                                .1;
                            chart::HitObjectData::Slider(chart::Slider {
                                control_points: curve_points
                                    .iter()
                                    .map(|p| cgmath::vec2(p.x as f32, p.y as f32))
                                    .collect::<Vec<_>>(),
                                curve_type: match curve_type {
                                    osu_types::CurveType::Bezier => chart::CurveType::Bezier,
                                    osu_types::CurveType::Perfect => chart::CurveType::Perfect,
                                    osu_types::CurveType::Linear => chart::CurveType::Linear,
                                    osu_types::CurveType::Catmull => todo!(),
                                },
                                repeat: (*slides as u32 - 1),
                                velocity: opx_per_sec,
                                length: *length,
                            })
                        }
                        SpecificHitObject::Spinner { end_time: _ } => todo!(),
                        SpecificHitObject::ManiaHold {} => todo!(),
                    },
                })
                .collect(),
        });
        ctx.set_chart_progress(ChartProgress { passed_index: 0 });

        println!(
            "Playing chart '{}'",
            ctx.chart().as_ref().unwrap().title.clone()
        );

        let mut hitcircle_batch = HitCircleBatch::new(
            &ctx.gfx,
            game_resources.tinted_circle.clone(),
            game_resources.overlay_circle.clone(),
            game_resources.approach_circle.clone(),
            64,
        );
        hitcircle_batch.set_view(Transform {
            position: cgmath::vec2(
                ctx.gfx.dimensions.x as f32 / 2.0 - 640.0 / 2.0,
                ctx.gfx.dimensions.y as f32 / 2.0 - 480.0 / 2.0,
            ),
            scale: cgmath::vec2(1.0, 1.0),
            rotation: cgmath::Rad(0.0),
        });

        let mut playfield_batch = SpriteBatch::new(&ctx.gfx, game_resources.playfield.clone(), 1);
        *playfield_batch.get_view_mut() = Transform {
            position: cgmath::vec2(
                ctx.gfx.dimensions.x as f32 / 2.0,
                ctx.gfx.dimensions.y as f32 / 2.0,
            ),
            scale: cgmath::vec2(
                ctx.gfx.dimensions.x as f32 / 2732.0,
                ctx.gfx.dimensions.y as f32 / 1572.0,
            ),
            rotation: cgmath::Rad(0.0),
        };
        let _playfield_entry = playfield_batch.insert(Transform::default());

        PlayingScreen {
            hitcircle_batch,
            playfield_batch,

            visible_sliders: Vec::new(),
        }
    }
}

impl Updatable for PlayingScreen {
    fn update(&mut self, ctx: &GameContext) {
        let gfx = &ctx.gfx;
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
        let chart_progress = chart_progress.as_ref().unwrap();

        let song_position = song.position() as f32;

        let mut display_objects = chart_data.objects[chart_progress.passed_index..]
            .iter()
            .enumerate()
            .skip_while(|(_, obj)| {
                song_position < obj.time - chart.modifiers.approach_seconds()
                    || song_position > obj.end_time()
            })
            .take_while(|(_, obj)| obj.time - chart.modifiers.approach_seconds() < song_position)
            .map(|(idx, _)| chart_progress.passed_index + idx)
            .collect::<Vec<_>>();

        let mut to_remove = vec![];

        for entry in &self.hitcircle_batch.keys {
            match display_objects.binary_search(&entry.index) {
                Ok(idx) => {
                    display_objects.remove(idx);
                }
                Err(_) => {
                    to_remove.push(*entry);
                    continue;
                }
            }

            let trans = self
                .hitcircle_batch
                .approach
                .get_mut(entry.approach)
                .unwrap();
            let hitobject = &chart_data.objects[entry.index];
            let scale = math::clamped_remap(
                hitobject.time - chart.modifiers.approach_seconds(),
                hitobject.time,
                1.0,
                0.25,
                song_position,
            );
            trans.scale.x = scale;
            trans.scale.y = scale;
        }

        for (_object_index, _slider_entry) in &self.visible_sliders {}

        for entry in to_remove {
            let hitobject = &chart_data.objects[entry.index];
            match hitobject.data {
                chart::HitObjectData::Circle => {
                    self.hitcircle_batch.remove(entry);
                }
                chart::HitObjectData::Slider(_) => {
                    self.hitcircle_batch.remove(entry);
                    match self
                        .visible_sliders
                        .binary_search_by_key(&entry.index, |s| s.0)
                    {
                        Ok(idx) => {
                            self.visible_sliders.remove(idx);
                        }
                        Err(_) => {}
                    }
                }
            }
        }

        for display_object in display_objects {
            let hitobject = &chart_data.objects[display_object];
            match &hitobject.data {
                chart::HitObjectData::Circle => {
                    self.hitcircle_batch
                        .insert(hitobject.position, display_object);
                }
                chart::HitObjectData::Slider(slider) => {
                    self.hitcircle_batch
                        .insert(hitobject.position, display_object);
                    self.visible_sliders.push((
                        display_object,
                        slider::Slider::new(
                            gfx,
                            slider.curve_type,
                            hitobject.position,
                            &slider.control_points,
                            ctx.game_resources().slider_track.clone(),
                        ),
                    ));
                }
            }
        }

        self.hitcircle_batch.update(ctx);
        self.playfield_batch.update(&ctx.gfx);
    }
}

impl Renderable for PlayingScreen {
    fn render<'data>(&'data self, pass: &mut wgpu::RenderPass<'data>) {
        for (_, slider) in &self.visible_sliders {
            slider.render(pass);
        }

        self.hitcircle_batch.render(pass);
        self.playfield_batch.render(pass);
    }
}
