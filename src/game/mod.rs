use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

use kira::{instance::handle::InstanceHandle, manager::AudioManager};
use resources::{Resource, Resources};

use crate::{
    graphics::{ArcTexture, GraphicsContext, Renderable, SpriteBatch, Transform},
    job::{spawn_job, JobHandle},
    math,
};

use self::{
    chart::{Chart, ChartData},
    hitcircle_batch::HitCircleBatch,
    screen::{Screen, Updatable},
};

pub mod chart;
pub mod hitcircle_batch;
pub mod screen;

pub struct GameResources {
    pub tinted_circle: ArcTexture,
    pub overlay_circle: ArcTexture,
    pub approach_circle: ArcTexture,
    pub playfield: ArcTexture,
}

struct Song(pub InstanceHandle);

pub struct ChartProgress {
    pub passed_index: usize,
}

pub struct GameContext {
    pub gfx: GraphicsContext,
    pub audio: Mutex<RefCell<AudioManager>>,
    pub resources: Resources,
}

impl GameContext {
    pub fn new(gfx: GraphicsContext, audio: AudioManager) -> Self {
        let mut resources = Resources::new();
        resources.insert::<Option<Song>>(None);
        resources.insert::<Option<Chart>>(None);
        resources.insert::<Option<ChartData>>(None);
        resources.insert::<Option<ChartProgress>>(None);
        resources.insert::<Option<Arc<GameResources>>>(None);
        GameContext {
            resources,
            gfx,
            audio: Mutex::new(RefCell::new(audio)),
        }
    }

    pub fn set_game_resources(&self, res: GameResources) {
        *self
            .resources
            .get_mut::<Option<Arc<GameResources>>>()
            .unwrap() = Some(Arc::new(res));
    }

    pub fn game_resources(&self) -> Arc<GameResources> {
        self.resources
            .get::<Option<Arc<GameResources>>>()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone()
    }

    pub fn set_song(&self, song: InstanceHandle) {
        *self.resources.get_mut::<Option<Song>>().unwrap() = Some(Song(song));
    }

    pub fn song(&self) -> Option<InstanceHandle> {
        self.get_raw_opt::<Song>().as_ref().map(|s| s.0.clone())
    }

    pub fn set_chart(&self, chart: Chart) {
        *self.resources.get_mut::<Option<Chart>>().unwrap() = Some(chart);
    }

    pub fn chart(&self) -> resources::Ref<Option<Chart>> {
        self.get_raw_opt::<Chart>()
    }

    pub fn set_chart_data(&self, chart_data: ChartData) {
        *self.resources.get_mut::<Option<ChartData>>().unwrap() = Some(chart_data);
    }

    pub fn chart_data(&self) -> resources::Ref<Option<ChartData>> {
        self.get_raw_opt::<ChartData>()
    }

    pub fn set_chart_progress(&self, chart_progress: ChartProgress) {
        *self.resources.get_mut::<Option<ChartProgress>>().unwrap() = Some(chart_progress);
    }

    pub fn chart_progress(&self) -> resources::Ref<Option<ChartProgress>> {
        self.get_raw_opt::<ChartProgress>()
    }

    fn get_raw_opt<T: Resource>(&self) -> resources::Ref<Option<T>> {
        self.resources.get::<Option<T>>().unwrap()
    }
}

pub struct PlayingResources {
    sound: kira::sound::Sound,
    beatmap: osu_parser::Beatmap,
}

pub struct PlayingScreen {
    hitcircle_batch: HitCircleBatch,
    playfield_batch: SpriteBatch,
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
            .play(kira::instance::InstanceSettings::default())
            .unwrap();

        ctx.set_song(instance_handle);
        ctx.set_chart(Chart {
            title: loading_res.beatmap.info.metadata.title.clone(),
            modifiers: chart::Modifiers { approach_rate: 5.5 },
        });
        ctx.set_chart_data(ChartData {
            objects: loading_res
                .beatmap
                .hit_objects
                .iter()
                .map(|hit_object| chart::HitObject {
                    position: cgmath::vec2(
                        math::remap(0.0, 640.0, -640.0, 640.0, hit_object.position.0 as f32),
                        math::remap(0.0, 480.0, -480.0, 480.0, hit_object.position.1 as f32),
                    ),
                    time: (hit_object.time as f32) / 1000.0,
                })
                .collect(),
        });
        ctx.set_chart_progress(ChartProgress { passed_index: 0 });

        println!(
            "Playing chart '{}'",
            ctx.chart().as_ref().unwrap().title.clone()
        );

        let mut hitcircle_batch = HitCircleBatch::new(&ctx.gfx, 64);
        hitcircle_batch.set_view(Transform {
            position: cgmath::vec2(
                ctx.gfx.dimensions.x as f32 / 2.0,
                ctx.gfx.dimensions.y as f32 / 2.0,
            ),
            scale: cgmath::vec2(0.5, 0.5),
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
        }
    }
}

impl Updatable for PlayingScreen {
    fn update(&mut self, ctx: &GameContext) {
        self.hitcircle_batch.update(ctx);
        self.playfield_batch.update(&ctx.gfx);
    }
}

impl Renderable for PlayingScreen {
    fn render<'data>(&'data self, pass: &mut wgpu::RenderPass<'data>) {
        self.hitcircle_batch.render(pass);
        self.playfield_batch.render(pass);
    }
}
