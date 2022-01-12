use self::{
    chart::{ChartData, ChartInfo},
    graphics::atlas::Atlas,
};
use kira::{instance::handle::InstanceHandle, manager::AudioManager};
use ogfx::{ArcTexture, GraphicsContext};
use resources::{Resource, Resources};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

pub mod chart;
pub mod graphics;
pub mod screen;
pub mod ui;

pub struct GameResources {
    pub hitobject_atlas: Atlas<String>,
    pub playfield: ArcTexture,
}

struct Song(pub InstanceHandle);

#[derive(Copy, Clone)]
pub struct ChartProgress {
    pub passed_index: usize,

    pub combo: u32,
    pub progress: f32,
}

pub struct GameContext {
    pub gfx: Arc<GraphicsContext>,
    pub audio: Mutex<AudioManager>,
    pub resources: Resources,
    pub game_resources: Arc<Mutex<Option<GameResources>>>,

    pub dirty: AtomicBool,
}

impl GameContext {
    pub fn new(gfx: GraphicsContext, audio: AudioManager) -> Self {
        let mut resources = Resources::new();
        resources.insert::<Option<Song>>(None);
        resources.insert::<Option<ChartInfo>>(None);
        resources.insert::<Option<ChartData>>(None);
        resources.insert::<Option<ChartProgress>>(None);
        GameContext {
            resources,
            gfx: Arc::new(gfx),
            audio: Mutex::new(audio),
            game_resources: Arc::new(Mutex::new(None)),
            dirty: AtomicBool::new(true),
        }
    }

    pub fn set_song(&self, song: InstanceHandle) {
        self.dirty.store(true, Ordering::SeqCst);
        *self.resources.get_mut::<Option<Song>>().unwrap() = Some(Song(song));
    }

    pub fn song(&self) -> Option<InstanceHandle> {
        self.get_raw_opt::<Song>().as_ref().map(|s| s.0.clone())
    }

    pub fn set_chart_info(&self, chart: ChartInfo) {
        self.dirty.store(true, Ordering::SeqCst);
        *self.resources.get_mut::<Option<ChartInfo>>().unwrap() = Some(chart);
    }

    pub fn chart(&self) -> resources::Ref<Option<ChartInfo>> {
        self.get_raw_opt::<ChartInfo>()
    }

    pub fn set_chart_data(&self, chart_data: ChartData) {
        self.dirty.store(true, Ordering::SeqCst);
        *self.resources.get_mut::<Option<ChartData>>().unwrap() = Some(chart_data);
    }

    pub fn chart_data(&self) -> resources::Ref<Option<ChartData>> {
        self.get_raw_opt::<ChartData>()
    }

    pub fn set_chart_progress(&self, chart_progress: ChartProgress) {
        self.dirty.store(true, Ordering::SeqCst);
        *self.resources.get_mut::<Option<ChartProgress>>().unwrap() = Some(chart_progress);
    }

    pub fn chart_progress(&self) -> Option<ChartProgress> {
        self.get_raw_opt::<ChartProgress>().as_ref().map(|&s| s)
    }

    fn get_raw_opt<T: Resource>(&self) -> resources::Ref<Option<T>> {
        self.resources.get::<Option<T>>().unwrap()
    }
}
