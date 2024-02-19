use std::f32::consts::{FRAC_PI_2, TAU};
use glam::{Mat4, Vec2, Vec3};
use win_loop::winit::keyboard::KeyCode;


const SPEED: f32 = 0.1;
const SENSITIVITY: f32 = 0.003;
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 100.0;
const FOV: f32 = FRAC_PI_2;

pub struct Camera {
    pub position: Vec3,
    rotation: Vec2,
    pub aspect_ratio: f32
}

impl Camera {
    pub fn new(position: Vec3, rotation: Vec2, aspect_ratio: f32) -> Self {
        Camera {
            position,
            rotation,
            aspect_ratio
        }
    }

    fn forward(&self) -> Vec3 {
        let (x_sin, x_cos) = self.rotation.x.sin_cos();
        let (y_sin, y_cos) = self.rotation.y.sin_cos();
        Vec3::new(y_cos * x_cos, x_sin, y_sin * x_cos)
    }

    fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    pub fn matrix(&self) -> Mat4 {
        let forward = self.forward();
        let right = self.right();
        let up = right.cross(forward).normalize();
        Mat4::perspective_rh(FOV, self.aspect_ratio, Z_NEAR, Z_FAR) * Mat4::look_to_rh(self.position, forward, up)
    }

    pub fn update(&mut self, keys: &[KeyCode]) {
        let mut translation = Vec3::ZERO;

        let right = self.right();
        let forward = right.cross(-Vec3::Y).normalize(); // "flat" forward vector -- not affected by pitch

        for key in keys {
            match key {
                KeyCode::KeyW => translation += forward,
                KeyCode::KeyS => translation -= forward,
                KeyCode::KeyA => translation -= right,
                KeyCode::KeyD => translation += right,
                KeyCode::Space => translation.y += 1.0,
                KeyCode::ShiftLeft => translation.y -= 1.0,
                _ => {}
            }
        }

        self.position += translation * SPEED;
    }

    pub fn update_rotation(&mut self, delta: Vec2) {
        self.rotation += delta * SENSITIVITY;
        self.rotation.x = self.rotation.x.clamp(0.99 * -FRAC_PI_2, 0.99 * FRAC_PI_2);
        self.rotation.y = self.rotation.y.rem_euclid(TAU);
    }
}