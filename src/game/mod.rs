use self::{
    chart::{Chart, ChartData},
    graphics::atlas::Atlas,
};
use kira::{instance::handle::InstanceHandle, manager::AudioManager};
use ogfx::{ArcTexture, GraphicsContext};
use resources::{Resource, Resources};
use std::{
    cell::RefCell,
    sync::{Arc, Mutex},
};

pub mod chart;
pub mod graphics;
pub mod screen;

pub struct GameResources {
    pub hitobject_atlas: Atlas<String>,
    pub playfield: ArcTexture,
}

struct Song(pub InstanceHandle);

pub struct ChartProgress {
    pub passed_index: usize,
}

pub struct GameContext {
    pub gfx: Arc<GraphicsContext>,
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
            gfx: Arc::new(gfx),
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
