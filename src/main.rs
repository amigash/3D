#![warn(clippy::pedantic)]
mod draw;
mod camera;

use glam::Vec3;
use crate::{
    draw::*,
};

use pix_win_loop::{
    start, App, Context, Duration, InputState, NamedKey, PhysicalSize, Pixels, Result,
    WindowBuilder,
};


const WIDTH: u32 = 40;
const HEIGHT: u32 = 30;
const SCALE: u32 = 10;

struct Triangle {
    a: Vec3,
    b: Vec3,
    c: Vec3
}

impl IntoIterator for Triangle {
    type Item = Vec3;
    type IntoIter = std::vec::IntoIter<Vec3>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self.a, self.b, self.c].into_iter()
    }
}

const fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

const fn tri(a: Vec3, b: Vec3, c: Vec3, ) -> Triangle {
    Triangle { a, b, c }
}

impl Triangle {
    fn surface_normal(&self) -> Vec3 {
        let a = self.b - self.a;
        let b = self.c - self.a;
        a.cross(b).normalize()
    }
}

struct Application {
    x_shift: i32,
    y_shift: i32,
    mesh: Vec<Triangle>
}

impl App for Application {
    fn update(&mut self, ctx: &mut Context) -> Result<()> {

        if ctx.input.is_logical_key_pressed(NamedKey::Escape) {
            ctx.exit();
            return Ok(())
        }
        for (key, input_state) in ctx.input.logical_keys() {
            let InputState::Pressed = input_state else {
                continue;
            };
            match key {
                NamedKey::ArrowUp => self.y_shift -= 1,
                NamedKey::ArrowDown => self.y_shift += 1,
                NamedKey::ArrowLeft => self.x_shift -= 1,
                NamedKey::ArrowRight => self.x_shift += 1,
                _ => {}
            }
        }

        Ok(())
    }

    fn render(&mut self, pixels: &mut Pixels, _blending_factor: f64) -> Result<()> {
        let frame = pixels.frame_mut();
        let triangle = ((10 + self.x_shift, 10 + self.y_shift), (30, 15), (10, 15));
        clear(frame);
        draw_triangle(frame,
                      WIDTH as i32, HEIGHT as i32,
                      triangle.0.0, triangle.0.1,
                      triangle.1.0, triangle.1.1,
                      triangle.2.0, triangle.2.1,
                      [255, 255, 255, 255]);
        pixels.render()?;
        Ok(())
    }
}

fn main() -> Result<()> {
    let mesh = vec![
        // Front
        tri(vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(1.0, 1.0, 0.0)),
        tri(vec3(0.0, 0.0, 0.0), vec3(1.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0)),
        // Back
        tri(vec3(0.0, 0.0, 1.0), vec3(1.0, 1.0, 1.0), vec3(0.0, 1.0, 1.0)),
        tri(vec3(0.0, 0.0, 1.0), vec3(1.0, 0.0, 1.0), vec3(1.0, 1.0, 1.0)),
        // Top
        tri(vec3(0.0, 1.0, 0.0), vec3(0.0, 1.0, 1.0), vec3(1.0, 1.0, 1.0)),
        tri(vec3(0.0, 1.0, 0.0), vec3(1.0, 1.0, 1.0), vec3(1.0, 1.0, 0.0)),
        // Bottom
        tri(vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 1.0), vec3(0.0, 0.0, 1.0)),
        tri(vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), vec3(1.0, 0.0, 1.0)),
        // Left
        tri(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 1.0), vec3(0.0, 1.0, 1.0)),
        tri(vec3(0.0, 0.0, 0.0), vec3(0.0, 1.0, 1.0), vec3(0.0, 1.0, 0.0)),
        // Right
        tri(vec3(1.0, 0.0, 0.0), vec3(1.0, 1.0, 1.0), vec3(1.0, 0.0, 1.0)),
        tri(vec3(1.0, 0.0, 0.0), vec3(1.0, 1.0, 0.0), vec3(1.0, 1.0, 1.0)),
    ];

    let window_builder = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(WIDTH * SCALE, HEIGHT * SCALE))
        .with_min_inner_size(PhysicalSize::new(WIDTH, HEIGHT));

    let pixel_buffer_size = PhysicalSize::new(WIDTH, HEIGHT);
    let target_frame_time = Duration::from_secs_f32(1. / 120.); // 120 fps
    let max_frame_time = Duration::from_secs_f32(0.1);

    start(
        window_builder,
        Application {
            x_shift: 0,
            y_shift: 0,
            mesh
        },
        pixel_buffer_size,
        target_frame_time,
        max_frame_time,
    )
}
