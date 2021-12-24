use crate::graphics;

pub struct Transform {
    pub position: cgmath::Vector2<f32>,
    pub scale: cgmath::Vector2<f32>,
}

impl Transform {
    pub fn as_matrix(&self, gfx: &graphics::Context) -> cgmath::Matrix4<f32> {
        #[rustfmt::skip]
        pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0,
        );

        OPENGL_TO_WGPU_MATRIX
            * cgmath::Matrix4::from_translation(cgmath::vec3(-1.0, 1.0, 0.0))
            * cgmath::Matrix4::from_nonuniform_scale(
                1.0 / gfx.dimensions.x as f32,
                1.0 / gfx.dimensions.y as f32,
                1.0,
            )
            * cgmath::Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, 1.0)
            * cgmath::Matrix4::from_translation(cgmath::vec3(
                self.position.x * 2.0,
                self.position.y * -2.0,
                0.0,
            ))
    }
}
