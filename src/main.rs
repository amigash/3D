#![warn(clippy::pedantic)]
extern crate core;

mod draw;
mod camera;
mod triangle;
mod mesh;

use std::{
    sync::Arc,
    time::{
        Duration,
        Instant
    },
    f32::consts::TAU,
    fs::File
};
use glam::{ivec3, IVec3, vec2, Vec2, Vec3, vec3};
use pixels::{Pixels, SurfaceTexture};
use win_loop::{
    App, Context, InputState, start,
    anyhow::Result,
    winit::{
        event::{Event, WindowEvent},
        dpi::PhysicalSize,
        event_loop::EventLoop,
        keyboard::NamedKey,
        window::WindowBuilder,
        event::DeviceEvent,
        window::Window,
        keyboard::KeyCode,
        window::{CursorGrabMode, Fullscreen}
    }
};
use crate::{triangle::Triangle, camera::Camera, draw::Draw};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const SCALE: u32 = 4;

struct Application {
    mesh: Vec<Triangle>,
    pixels: Pixels,
    window: Arc<Window>,
    scale: u32,
    time: Instant,
    camera: Camera,
    draw: Draw
}


impl App for Application {
    fn update(&mut self, ctx: &mut Context) -> Result<()> {

        if ctx.input.is_logical_key_pressed(NamedKey::Escape) {
            ctx.exit();
        }

        let keys: Vec<KeyCode> = ctx.input.physical_keys()
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
            ivec3(inner_size.width, inner_size.height, 0) / (SCALE as i32)
        };

        let matrix = self.camera.matrix();
        let scale_factor = 0.5 * size.as_vec3();

        let transform = |point: &Vec3| {
            let homogeneous = point.extend(1.0);
            let projected = matrix * homogeneous;
            let perspective_divided = projected / projected.w;
            let flipped = perspective_divided.truncate() * vec3(1.0, -1.0, 1.0);
            let centered = flipped + 1.0;
            let scaled = centered * scale_factor;
            scaled
        };

        let transform_triangle = |triangle: &Triangle| {
            Triangle {
                a: transform(&triangle.a),
                b: transform(&triangle.b),
                c: transform(&triangle.c),
                normal: transform(&triangle.normal)
            }
        };


        let is_on_screen = |point: IVec3| {
            point.x > 0 && point.y > 0 && point.x < size.x && point.y < size.y
        };

        let is_on_screen_triangle = |triangle: &Triangle| {
            [triangle.a, triangle.b, triangle.c].iter().all(|vertex| is_on_screen(vertex.as_ivec3()))
        };

        let is_visible = |triangle: &&Triangle| {
            triangle.normal.dot(self.camera.position - triangle.centroid()) >= 0.0
        };

        let time = self.time.elapsed().as_secs_f32();
        let rgb: Vec<u8> = (0..3).map(|i| ((TAU * (time + i as f32 / 3.0)).sin() * 127.5 + 127.5).round() as u8).collect();
        let rgba = [rgb[0], rgb[1], rgb[2], 255];

        for tri in self.mesh.iter()
            .filter(is_visible)
            .map(transform_triangle)
            .filter(is_on_screen_triangle)
        {
            self.draw.triangle(
                tri.a.round().as_ivec3().truncate(),
                tri.b.round().as_ivec3().truncate(),
                tri.c.round().as_ivec3().truncate(),
                rgba);
        }

        for (axis, color) in [(Vec3::X, [255, 0, 0, 255]), (Vec3::Y, [0, 255, 0, 255]), (Vec3::Z, [0, 0, 255, 255])] {
            let origin = transform(&Vec3::ZERO).round().as_ivec3();
            let transformed = transform(&axis).round().as_ivec3();
            if is_on_screen(origin) && is_on_screen(transformed) {
                self.draw.line(
                    origin.truncate(),
                    transformed.truncate(),
                    color
                )
            }
        }

        self.draw.pixel(
            (size / 2).truncate(),
            [255, 255, 255, 255]
        );

        self.draw.copy_to_frame(self.pixels.frame_mut());
        self.pixels.render()?;

        Ok(())
    }

    fn handle(&mut self, event: &Event<()>) -> Result<()> {
        match event {
            Event::WindowEvent {event, .. } => {
                match event {
                    WindowEvent::Resized(size) => {
                        self.pixels.resize_surface(size.width, size.height)?;
                        let (width, height) = (size.width / self.scale, size.height / self.scale);
                        self.pixels.resize_buffer(width, height)?;
                        self.draw = Draw::new(width as usize, height as usize);
                        self.camera.aspect_ratio = width as f32 / height as f32;
                    },
                    _ => {}
                }
            }
            Event::DeviceEvent {event, .. } =>
                match event {
                    DeviceEvent::MouseMotion { delta: (dx, dy)} => {
                        self.camera.update_rotation(vec2(-*dy as f32, *dx as f32));
                    }
                    _ => ()
                }
            _ => {}
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let mesh = mesh::load_from_obj_file(File::open("assets/teapot.obj")?)?;

    let event_loop = EventLoop::new()?;

    let window = Arc::new(WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .build(&event_loop)?);

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

    let draw = Draw::new(pixel_buffer_size.width as usize, pixel_buffer_size.height as usize);

    let time = Instant::now();

    let app = Application {
        mesh,
        pixels,
        window: window.clone(),
        scale: SCALE,
        time,
        camera: Camera::new(Vec3::ZERO, Vec2::ZERO, 0.0),
        draw
    };

    start(
        event_loop,
        window,
        app,
        target_frame_time,
        max_frame_time,
    )
}
