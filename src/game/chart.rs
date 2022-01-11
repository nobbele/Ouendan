use crate::math;

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
    pub velocity: f32,
    pub length: f32,
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

impl HitObject {
    pub fn end_time(&self) -> f32 {
        match &self.data {
            HitObjectData::Circle => self.time,
            HitObjectData::Slider(s) => {
                self.time + (s.length / s.velocity) * (s.repeat as f32 + 1.0)
            }
        }
    }
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

impl std::fmt::Debug for Modifiers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Modifiers")
            .field(
                "approach_rate",
                &format!(
                    "{:.2} ({:.0} ms)",
                    self.approach_rate,
                    self.approach_seconds() * 1000.0
                ),
            )
            .finish()
    }
}

#[derive(Debug)]
pub struct ChartInfo {
    pub title: String,
    pub modifiers: Modifiers,
}

fn ar_from_secs(sec: f32) -> f32 {
    (1.0 - sec) * 10.0
}

fn difficulty_range(difficulty: f32, min: f32, mid: f32, max: f32) -> f32 {
    if difficulty > 5.0 {
        mid + (max - mid) * (difficulty - 5.0) / 5.0
    } else {
        mid - (mid - min) * (5.0 - difficulty) / 5.0
    }
}

fn osu_ar_to_secs(ar: f32) -> f32 {
    difficulty_range(ar, 1.800, 1.200, 0.450)
}

#[test]
fn test_difficulty_maps() {
    assert!(osu_ar_to_secs(9.0) - 0.600 <= f32::EPSILON);
    assert!(osu_ar_to_secs(6.0) - 1.050 <= f32::EPSILON);
    assert!(osu_ar_to_secs(3.0) - 1.440 <= f32::EPSILON);
}

pub fn load_osu_beatmap(beatmap: &osu_parser::Beatmap) -> (ChartInfo, ChartData) {
    let info = ChartInfo {
        title: beatmap.info.metadata.title.clone(),
        modifiers: Modifiers {
            approach_rate: ar_from_secs(osu_ar_to_secs(beatmap.info.difficulty.ar)),
        },
    };
    let opx_per_secs = beatmap
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
                    let sv = beatmap.info.difficulty.slider_multiplier * multiplier;
                    let bps = *bpm / 60.0;
                    (100.0 * bps) / sv
                },
            ))
        })
        .collect::<Vec<_>>();
    fn opx_to_oepx(x: i16, y: i16) -> cgmath::Vector2<f32> {
        cgmath::vec2(
            math::remap(0.0, 512.0, -320.0, 320.0, x as f32),
            math::remap(0.0, 384.0, -240.0, 240.0, y as f32),
        )
    }
    let data = ChartData {
        objects: beatmap
            .hit_objects
            .iter()
            .map(|hit_object| HitObject {
                position: opx_to_oepx(hit_object.position.0 as i16, hit_object.position.1 as i16),
                time: (hit_object.time as f32) / 1000.0,
                data: match &hit_object.specific {
                    osu_types::SpecificHitObject::Circle => HitObjectData::Circle,
                    osu_types::SpecificHitObject::Slider {
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
                        HitObjectData::Slider(Slider {
                            control_points: curve_points
                                .iter()
                                //.map(|p| cgmath::vec2(p.x as f32, p.y as f32))
                                .map(|p| opx_to_oepx(p.x, p.y))
                                .collect::<Vec<_>>(),
                            curve_type: match curve_type {
                                osu_types::CurveType::Bezier => CurveType::Bezier,
                                osu_types::CurveType::Perfect => CurveType::Perfect,
                                osu_types::CurveType::Linear => CurveType::Linear,
                                osu_types::CurveType::Catmull => todo!(),
                            },
                            repeat: (*slides as u32 - 1),
                            velocity: opx_per_sec,
                            length: *length,
                        })
                    }
                    osu_types::SpecificHitObject::Spinner { end_time: _ } => todo!(),
                    osu_types::SpecificHitObject::ManiaHold {} => todo!(),
                },
            })
            .collect(),
    };
    (info, data)
}
