use glam::{Mat4, Vec2, Vec3A};
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

    pub fn forward(&self) -> Vec3A {
        let (x_sin, x_cos) = self.rotation.x.sin_cos();
        let (y_sin, y_cos) = self.rotation.y.sin_cos();
        Vec3A::new(y_cos * x_cos, x_sin, y_sin * x_cos)
    }

    fn right(&self) -> Vec3A {
        self.forward().cross(Vec3A::Y).normalize()
    }

    fn up(&self) -> Vec3A {
        self.right().cross(self.forward()).normalize()
    }

    pub fn matrix(&self) -> Mat4 {
        Mat4::perspective_rh(Self::FOV, self.aspect_ratio, Self::Z_NEAR, Self::Z_FAR)
            * Mat4::look_to_rh(
            self.position.into(),
            self.forward().into(),
            self.up().into(),
        )
    }

    pub fn update(&mut self, keys: &[KeyCode]) {
        let mut translation = Vec3A::ZERO;

        let right = self.right();
        let forward = self.forward().with_y(0.0).normalize();

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
