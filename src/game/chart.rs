/*pub struct Circle {}

pub enum HitObjectType {
    Circle(Circle),
}*/

pub struct HitObject {
    pub position: cgmath::Vector2<f32>,
    pub time: f32,
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
