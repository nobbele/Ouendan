use super::{GameContext, PlayingScreen};
use crate::job::JobHandle;

pub trait Updatable {
    fn update(&mut self, ctx: &GameContext);
}

pub trait Screen {
    type LoadingResource;

    fn load(ctx: std::sync::Arc<GameContext>) -> JobHandle<Self::LoadingResource>;
    fn new(ctx: &GameContext, loading_res: Self::LoadingResource) -> Self;
}

pub enum GameScreen {
    Playing(PlayingScreen),
}

pub enum GameLoadingResource {
    Playing(JobHandle<<PlayingScreen as Screen>::LoadingResource>),
}
