#![warn(clippy::pedantic)]
extern crate core;

mod camera;
mod draw;
mod mesh;
mod triangle;

use crate::{camera::Camera, draw::Draw, triangle::Triangle};
use glam::{ivec2, vec2, vec3a, IVec3, Vec2, Vec3A, Vec4};
use pixels::{Pixels, SurfaceTexture};
use std::{
    f32::consts::TAU,
    fs::File,
    sync::Arc,
    time::{Duration, Instant},
};
use win_loop::{
    anyhow::Result,
    start,
    winit::{
        dpi::PhysicalSize,
        event::{DeviceEvent, Event, WindowEvent},
        event_loop::EventLoop,
        keyboard::{KeyCode, NamedKey},
        window::{CursorGrabMode, Fullscreen, Window, WindowBuilder},
    },
    App, Context, InputState,
};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const SCALE: u32 = 1;

struct Application {
    mesh: Vec<Triangle>,
    pixels: Pixels,
    window: Arc<Window>,
    scale: u32,
    time: Instant,
    camera: Camera,
    draw: Draw,
}

impl App for Application {
    fn update(&mut self, ctx: &mut Context) -> Result<()> {
        if ctx.input.is_logical_key_pressed(NamedKey::Escape) {
            ctx.exit();
        }

        let keys: Vec<KeyCode> = ctx
            .input
            .physical_keys()
            .iter()
            .filter(|(_, input_state)| matches!(input_state, InputState::Down))
            .map(|(&key_code, _)| key_code)
            .collect();
        self.camera.update(&keys);

        Ok(())
    }

    fn render(&mut self, _blending_factor: f64) -> Result<()> {
        let size = {
            let inner_size = self.window.inner_size().cast::<i32>();
            (ivec2(inner_size.width, inner_size.height) / (SCALE as i32)).extend(1)
        };

        let time = self.time.elapsed().as_secs_f32();
        let rgb: Vec<u8> = (0..3)
            .map(|i| ((TAU * (time + i as f32 / 3.0)).sin() * 127.5 + 127.5).round() as u8)
            .collect();
        let rgba = [rgb[0], rgb[1], rgb[2], 255];

        let matrix = self.camera.matrix();
        let scale_factor = 0.5 * size.as_vec3a();

        let project = |point: Vec3A| matrix * point.extend(1.0);
        let ahead_of = |point: &Vec4| point.z > 0.01;

        let transform = |point: Vec4| {
            let perspective_divided: Vec3A = ((point / point.w).truncate()).into();
            let flipped = perspective_divided * vec3a(1.0, -1.0, 1.0);
            let centered = flipped + 1.0;
            let scaled = centered * scale_factor;
            scaled.round().as_ivec3()
        };

        let is_on_screen =
            |point: &IVec3| point.x > 0 && point.y > 0 && point.x < size.x && point.y < size.y;

        let is_visible = |triangle: &&Triangle| {
            triangle
                .normal
                .dot(self.camera.position - triangle.centroid())
                >= 0.0
        };

        for points in self
            .mesh
            .iter()
            .filter(is_visible)
            .map(|triangle| triangle.points.map(project))
            .filter(|p| p.iter().all(ahead_of))
            .map(|v| v.map(transform))
            .filter(|p| p.iter().all(is_on_screen))
        {
            self.draw.triangle(points, rgba);
        }

        let projected_origin = project(Vec3A::ZERO);
        if ahead_of(&projected_origin) {
            let transformed_origin = transform(projected_origin);
            if is_on_screen(&transformed_origin) {
                for (axis, color) in [
                    (Vec3A::X, [255, 0, 0, 255]),
                    (Vec3A::Y, [0, 255, 0, 255]),
                    (Vec3A::Z, [0, 0, 255, 255]),
                ] {
                    let projected_axis = project(axis);
                    if ahead_of(&projected_axis) {
                        let transformed_axis = transform(projected_axis);
                        if is_on_screen(&transformed_axis) {
                            self.draw.line(transformed_axis, transformed_origin, color);
                        }
                    }
                }
            }
        }

        self.draw.pixel(size / 2, [255, 255, 255, 255]);

        self.draw.copy_to_frame(self.pixels.frame_mut());
        self.pixels.render()?;

        Ok(())
    }

    fn handle(&mut self, event: &Event<()>) -> Result<()> {
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                self.pixels.resize_surface(size.width, size.height)?;
                let (width, height) = (size.width / self.scale, size.height / self.scale);
                self.pixels.resize_buffer(width, height)?;
                self.draw = Draw::new(width as usize, height as usize);
                self.camera.aspect_ratio = width as f32 / height as f32;
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta: (dx, dy) },
                ..
            } => {
                self.camera.update_rotation(vec2(-*dy as f32, *dx as f32));
            }
            _ => {}
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let mesh = mesh::load_from_obj_file(File::open("assets/sentry.obj")?)?;

    let event_loop = EventLoop::new()?;

    let window = Arc::new(
        WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
            .with_fullscreen(Some(Fullscreen::Borderless(None)))
            .build(&event_loop)?,
    );

    window.set_cursor_grab(CursorGrabMode::Confined)?;
    window.set_cursor_visible(false);

    let target_frame_time = Duration::from_secs_f32(1. / 120.); // 120 fps
    let max_frame_time = Duration::from_secs_f32(0.1);

    let pixel_buffer_size = PhysicalSize::new(WIDTH / SCALE, HEIGHT / SCALE);
    let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, &window);

    let pixels = Pixels::new(
        pixel_buffer_size.width,
        pixel_buffer_size.height,
        surface_texture,
    )?;

    let draw = Draw::new(
        pixel_buffer_size.width as usize,
        pixel_buffer_size.height as usize,
    );

    let time = Instant::now();

    let app = Application {
        mesh,
        pixels,
        window: window.clone(),
        scale: SCALE,
        time,
        camera: Camera::new(Vec3A::ZERO, Vec2::ZERO, 0.0),
        draw,
    };

    start(event_loop, window, app, target_frame_time, max_frame_time)
}
