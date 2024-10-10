use glam::{Mat4, Vec2, Vec3, Vec3A, Vec4};
use std::f32::consts::{FRAC_PI_2, TAU};
use win_loop::winit::keyboard::KeyCode;

pub struct Camera {
    pub position: Vec3A,
    rotation: Vec2,
    pub aspect_ratio: f32,
}

impl Camera {
    const Z_NEAR: f32 = 0.1;
    const Z_FAR: f32 = 500.0;
    const SPEED: f32 = 0.1;
    const SENSITIVITY: f32 = 0.003;
    const FOV: f32 = FRAC_PI_2;

    pub fn new(position: Vec3A, rotation: Vec2) -> Self {
        Camera {
            position,
            rotation,
            aspect_ratio: 0.0,
        }
    }

    fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(Self::FOV, self.aspect_ratio, Self::Z_NEAR, Self::Z_FAR)
    }

    fn rotation_matrix(&self) -> Mat4 {
        let (sin_x, cos_x) = self.rotation.x.sin_cos();
        let (sin_y, cos_y) = self.rotation.y.sin_cos();

        Mat4::from_cols(
            Vec4::new(cos_y, -sin_x * sin_y, -cos_x * sin_y, 0.0),
            Vec4::new(0.0, cos_x, -sin_x, 0.0),
            Vec4::new(sin_y, cos_y * sin_x, cos_x * cos_y, 0.0),
            Vec4::W,
        )
    }

    fn translation_matrix(&self) -> Mat4 {
        Mat4::from_translation(Vec3::from(-self.position))
    }

    fn view_matrix(&self) -> Mat4 {
        self.rotation_matrix() * self.translation_matrix()
    }

    pub fn view_projection_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    pub fn update(&mut self, keys: &[KeyCode]) {
        let mut translation = Vec3A::ZERO;

        let camera_matrix = self.view_matrix().inverse();
        let right = Vec3A::from_vec4(camera_matrix.col(0));
        let forward = -Vec3A::from_vec4(camera_matrix.col(2)).with_y(0.0);

        for key in keys {
            match key {
                KeyCode::KeyW => translation += forward,
                KeyCode::KeyS => translation -= forward,
                KeyCode::KeyA => translation -= right,
                KeyCode::KeyD => translation += right,
                KeyCode::Space => translation.y += 1.0,
                KeyCode::ShiftLeft => translation.y -= 1.0,
                _ => (),
            }
        }

        self.position += translation * Self::SPEED;
    }

    pub fn update_rotation(&mut self, delta: Vec2) {
        self.rotation += delta * Self::SENSITIVITY;
        self.rotation.x = self.rotation.x.clamp(0.99 * -FRAC_PI_2, 0.99 * FRAC_PI_2);
        self.rotation.y = self.rotation.y.rem_euclid(TAU);
    }
}
