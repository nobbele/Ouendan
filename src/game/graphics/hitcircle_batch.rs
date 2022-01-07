use crate::{
    game::GameContext,
    graphics::{
        self, spritebatch::SpriteIdx, ArcTexture, GraphicsContext, Renderable, SpriteBatch,
        Transform,
    },
};

#[derive(Clone, Copy)]
pub struct HitCircleEntry {
    #[allow(dead_code)]
    pub tinted: SpriteIdx,
    #[allow(dead_code)]
    pub overlay: SpriteIdx,
    pub approach: SpriteIdx,
    pub index: usize,
}

// Maybe make a triple-image batch instead of 3 separate batches?
pub struct HitCircleBatch {
    pub tinted: SpriteBatch,
    pub overlay: SpriteBatch,
    pub approach: SpriteBatch,
    pub keys: Vec<HitCircleEntry>,
}

impl HitCircleBatch {
    pub fn new(
        gfx: &GraphicsContext,
        tinted: ArcTexture,
        overlay: ArcTexture,
        approach: ArcTexture,
        capacity: usize,
    ) -> Self {
        let tinted = SpriteBatch::new(gfx, tinted, capacity);
        let overlay = SpriteBatch::new(gfx, overlay, capacity);
        let approach = SpriteBatch::new(gfx, approach, capacity);
        HitCircleBatch {
            tinted,
            overlay,
            approach,
            keys: Vec::new(),
        }
    }

    pub fn set_view(&mut self, view: graphics::Transform) {
        *self.tinted.get_view_mut() = view.clone();
        *self.overlay.get_view_mut() = view.clone();
        *self.approach.get_view_mut() = view;
    }

    pub fn remove(&mut self, entry: HitCircleEntry) {
        self.tinted.remove(entry.tinted);
        self.overlay.remove(entry.overlay);
        self.approach.remove(entry.approach);
        let keys_index = self
            .keys
            .iter()
            .position(|key| entry.index == key.index)
            .unwrap();
        self.keys.remove(keys_index);
    }

    pub fn insert(&mut self, position: cgmath::Vector2<f32>, index: usize) {
        let trans = Transform {
            position: cgmath::vec2(position.x, position.y),
            layer: 0,
            scale: cgmath::vec2(0.25, 0.25),
            rotation: cgmath::Rad(0.0),
        };
        let tinted = self.tinted.insert(trans.clone());
        let overlay = self.overlay.insert(trans.clone());
        let approach = self.approach.insert(trans);

        self.keys.push(HitCircleEntry {
            tinted,
            overlay,
            approach,
            index,
        });
    }

    pub fn update(&mut self, ctx: &GameContext) {
        self.tinted.update(&ctx.gfx);
        self.overlay.update(&ctx.gfx);
        self.approach.update(&ctx.gfx);
    }
}

impl Renderable for HitCircleBatch {
    fn render<'data>(&'data self, pass: &mut wgpu::RenderPass<'data>) {
        self.tinted.render(pass);
        self.overlay.render(pass);
        self.approach.render(pass);
    }
}
