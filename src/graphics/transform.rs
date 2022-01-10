use crevice::std140::AsStd140;

use crate::Rect;

#[derive(AsStd140)]
pub struct RawTransform {
    pub matrix: cgmath::Matrix4<f32>,
    pub source: cgmath::Vector4<f32>,
}

impl RawTransform {
    pub fn packed_size() -> wgpu::BufferAddress {
        use std::mem::size_of;
        (size_of::<cgmath::Matrix4<f32>>() + size_of::<cgmath::Vector4<f32>>()) as _
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: cgmath::Vector2<f32>,
    pub layer: u32,
    pub scale: cgmath::Vector2<f32>,
    pub rotation: cgmath::Rad<f32>,
    pub source: Rect<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: cgmath::vec2(0.0, 0.0),
            layer: 0,
            scale: cgmath::vec2(1.0, 1.0),
            rotation: cgmath::Rad(0.0),
            source: Rect::new(0.0, 0.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub fn as_matrix(&self) -> RawTransform {
        let matrix = cgmath::Matrix4::from_translation(
            self.position.extend(self.layer as f32 / u16::MAX as f32),
        ) * cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, 1.0)
            * cgmath::Matrix4::from_angle_z(self.rotation);
        let source = cgmath::vec4(
            self.source.position.x,
            self.source.position.y,
            self.source.size.x,
            self.source.size.y,
        );
        RawTransform { matrix, source }
    }
}
