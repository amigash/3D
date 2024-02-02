use nannou::prelude::{Mat4LookTo, Mat4, Point2, Point3, Vec2, Vec3, Key, App};
use std::f32::consts::{FRAC_PI_2, TAU};
const SPEED: f32 = 0.1;
const SENSITIVITY: f32 = 0.03;
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 100.0;
const FOV: f32 = FRAC_PI_2;

pub struct Camera {
    position: Point3,
    rotation: Vec2,
    projection_matrix: Mat4,
}

impl Camera {
    pub fn new(position: Point3, rotation: Vec2) -> Self {
        Camera {
            position,
            rotation,
            projection_matrix: Mat4::perspective_rh(FOV, 1.0, Z_NEAR, Z_FAR)
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
        self.projection_matrix * Mat4::look_to_rh(self.position, forward, up)
    }

    pub fn update(&mut self, app: &App) {
        let mut rotation = Point2::ZERO;
        let mut translation = Vec3::ZERO;

        let right = self.right();
        let forward = right.cross(-Vec3::Y).normalize(); // "flat" forward vector -- not affected by pitch

        for key in app.keys.down.iter() {
            match key {
                Key::W => translation += forward,
                Key::S => translation -= forward,
                Key::A => translation -= right,
                Key::D => translation += right,
                Key::Space => translation.y += 1.0,
                Key::LShift => translation.y -= 1.0,
                Key::Left => rotation.y -= 1.0,
                Key::Right => rotation.y += 1.0,
                Key::Up => rotation.x += 1.0,
                Key::Down => rotation.x -= 1.0,
                _ => {}
            }
        }

        self.position += translation * SPEED;
        self.rotation += rotation * SENSITIVITY;
        self.rotation.x = self.rotation.x.clamp(0.99 * -FRAC_PI_2, 0.99 * FRAC_PI_2);
        self.rotation.y = self.rotation.y.rem_euclid(TAU);

        self.update_aspect_ratio(app);
    }

    fn update_aspect_ratio(&mut self, app: &App) {
        let window_rect = app.window_rect();
        let aspect_ratio = window_rect.w() / window_rect.h();
        self.projection_matrix = Mat4::perspective_rh(FOV, aspect_ratio, Z_NEAR, Z_FAR);
    }
}