#[derive(Clone, Copy)]
pub enum CurveType {
    Perfect,
    Bezier,
    Linear,
}

pub struct Slider {
    pub control_points: Vec<cgmath::Vector2<f32>>,
    pub curve_type: CurveType,
    pub repeat: u32,
}

pub enum HitObjectData {
    Circle,
    Slider(Slider),
}

pub struct HitObject {
    pub position: cgmath::Vector2<f32>,
    pub time: f32,
    pub data: HitObjectData,
}

pub struct ChartData {
    pub objects: Vec<HitObject>,
}

pub struct Modifiers {
    pub approach_rate: f32,
}

impl Modifiers {
    pub fn approach_seconds(&self) -> f32 {
        1.0 - self.approach_rate * 0.1
    }
}

pub struct Chart {
    pub title: String,
    pub modifiers: Modifiers,
}
