use crate::{
    graphics::{self, spritebatch::SpriteIdx, GraphicsContext, Renderable, SpriteBatch, Transform},
    math,
};

use super::GameContext;

#[derive(Clone, Copy)]
pub struct HitCircleEntry {
    #[allow(dead_code)]
    tinted: SpriteIdx,
    #[allow(dead_code)]
    overlay: SpriteIdx,
    approach: SpriteIdx,
    index: usize,
}

// Maybe make a triple-image batch instead of 3 separate batches?
pub struct HitCircleBatch {
    tinted: SpriteBatch,
    overlay: SpriteBatch,
    approach: SpriteBatch,
    keys: Vec<HitCircleEntry>,
    p: f32,
}

impl HitCircleBatch {
    pub fn new(gfx: &GraphicsContext, capacity: usize) -> Self {
        let tinted_texture = graphics::Texture::new(
            gfx,
            include_bytes!("../../resources/circle/tinted.png"),
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );
        let overlay_texture = graphics::Texture::new(
            gfx,
            include_bytes!("../../resources/circle/overlay.png"),
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );

        let approach_texture = graphics::Texture::new(
            gfx,
            include_bytes!("../../resources/circle/approach.png"),
            wgpu::TextureFormat::Rgba8UnormSrgb,
        );

        let tinted = SpriteBatch::new(gfx, tinted_texture, capacity);
        let overlay = SpriteBatch::new(gfx, overlay_texture, capacity);
        let approach = SpriteBatch::new(gfx, approach_texture, capacity);
        HitCircleBatch {
            tinted,
            overlay,
            approach,
            keys: Vec::new(),
            p: 0.0,
        }
    }

    pub fn set_view(&mut self, view: graphics::Transform) {
        *self.tinted.get_view_mut() = view.clone();
        *self.overlay.get_view_mut() = view.clone();
        *self.approach.get_view_mut() = view;
    }

    fn remove(&mut self, entry: HitCircleEntry) {
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

    fn insert(&mut self, position: cgmath::Vector2<f32>, index: usize) {
        let trans = Transform {
            position,
            scale: cgmath::vec2(0.5, 0.5),
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
                    || song_position > obj.time
            })
            .take_while(|(_, obj)| obj.time - chart.modifiers.approach_seconds() < song_position)
            .map(|(idx, _)| chart_progress.passed_index + idx)
            .collect::<Vec<_>>();

        let mut to_remove = vec![];

        for entry in &self.keys {
            match display_objects.binary_search(&entry.index) {
                Ok(idx) => {
                    display_objects.remove(idx);
                }
                Err(_) => {
                    to_remove.push(*entry);
                    continue;
                }
            }

            let trans = self.approach.get_mut(entry.approach).unwrap();
            let hitobject = &chart_data.objects[entry.index];
            let scale = math::clamped_remap(
                hitobject.time - chart.modifiers.approach_seconds(),
                hitobject.time,
                2.0,
                0.5,
                song_position,
            );
            trans.scale.x = scale;
            trans.scale.y = scale;
            self.tinted.update(gfx);
        }

        for entry in to_remove {
            self.remove(entry);
        }

        for display_object in display_objects {
            let hitobject = &chart_data.objects[display_object];
            self.insert(hitobject.position, display_object);
        }

        self.tinted.update(gfx);
        self.overlay.update(gfx);
        self.approach.update(gfx);

        self.p += 0.01;
    }
}

impl Renderable for HitCircleBatch {
    fn render<'data>(&'data self, pass: &mut wgpu::RenderPass<'data>) {
        self.tinted.render(pass);
        self.overlay.render(pass);
        self.approach.render(pass);
    }
}
