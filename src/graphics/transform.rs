#[derive(Debug, Clone)]
pub struct Transform {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
    pub rotation: cgmath::Rad<f32>,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: cgmath::vec2(0.0, 0.0),
            scale: cgmath::vec2(1.0, 1.0),
            rotation: cgmath::Rad(0.0),
        }
    }
}

impl Transform {
    pub fn as_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::from_translation(self.position.extend(0.0))
            * cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, 1.0)
            * cgmath::Matrix4::from_angle_z(self.rotation)
    }
}
