#![warn(clippy::pedantic)]
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
use glam::{Vec2, vec2, Vec3};
use pixels::*;
use win_loop::{
    *,
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
use crate::{draw::*, triangle::*, camera::*};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const SCALE: u32 = 4;

const fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

struct Application {
    mesh: Vec<Triangle>,
    pixels: Pixels,
    window: Arc<Window>,
    scale: u32,
    time: Instant,
    camera: Camera,
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
        let frame = self.pixels.frame_mut();
        let size = self.window.inner_size();
        clear(frame);

        let matrix = self.camera.matrix();
        let scale_factor = 0.5 * vec3(size.width as f32, size.height as f32,0.0) / (SCALE as f32);

        let transform = |point: &Vec3| {
            let homogeneous = point.extend(1.0);
            let projected = matrix * homogeneous;
            let perspective_divided = if projected.w == 0.0 {
                projected
            } else {
                projected / projected.w
            };
            let flipped = vec3(perspective_divided.x, -perspective_divided.y, perspective_divided.z);
            let scaled = scale_factor * flipped;
            let centered = scaled + 0.5 * vec3(size.width as f32, size.height as f32, 0.0) / (SCALE as f32);
            centered
        };

        let transform_triangle = |triangle: &Triangle| {
            Triangle {
                a: transform(&triangle.a),
                b: transform(&triangle.b),
                c: transform(&triangle.c),
            }
        };

        let is_visible = |triangle: &&Triangle| {
            let normal = triangle.surface_normal();
            let view_vector = self.camera.position - triangle.centroid();
            normal.dot(view_vector) >= 0.0
        };

        let draw_axis = |frame_: &mut [u8] ,axis: Vec3, color: [u8; 4]| {
            let origin = transform(&Vec3::ZERO);
            let transformed = transform(&axis);
            line(
                frame_,
                (size.width / SCALE) as i32, (size.height / SCALE) as i32,
                origin.x as i32, origin.y as i32,
                transformed.x as i32, transformed.y as i32,
                color
            )
        };

        let time = self.time.elapsed().as_secs_f32();
        let rgb: Vec<u8> = (0..3).map(|i| ((TAU * (time + i as f32 / 3.0)).sin() * 127.5 + 127.5).round() as u8).collect();
        let rgba = [rgb[0], rgb[1], rgb[2], 255];

        for tri in self.mesh.iter()
            .filter(is_visible)
            .map(transform_triangle)
        {
            triangle(
                frame,
                (size.width / SCALE) as i32, (size.height / SCALE) as i32,
                tri.a.x as i32, tri.a.y as i32,
                tri.b.x as i32, tri.b.y as i32,
                tri.c.x as i32, tri.c.y as i32,
                rgba);
        }
        draw_axis(frame, Vec3::X, [255, 0, 0, 255]);
        draw_axis(frame, Vec3::Y, [0, 255, 0, 255]);
        draw_axis(frame, Vec3::Z, [0, 0, 255, 255]);

        pixel(
            frame,
            (size.width / SCALE) as i32,
            (size.width / (2 * SCALE)) as i32, (size.height / (2 * SCALE)) as i32,
            [255, 255, 255, 255]
        );

        self.pixels.render()?;

        Ok(())
    }

    fn handle(&mut self, event: &Event<()>) -> Result<()> {
        match event {
            Event::WindowEvent {event, .. } => {
                match event {
                    WindowEvent::Resized(size) => {
                        self.pixels.resize_surface(size.width, size.height)?;
                        self.pixels.resize_buffer(size.width / self.scale, size.height / self.scale)?;
                        self.camera.aspect_ratio = size.width as f32 / size.height as f32;
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

    let target_frame_time = Duration::from_secs_f32(1. / 120.); // 60 fps
    let max_frame_time = Duration::from_secs_f32(0.1);

    let pixel_buffer_size = PhysicalSize::new(WIDTH / SCALE, HEIGHT / SCALE);
    let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, &window);

    let pixels = Pixels::new(
        pixel_buffer_size.width,
        pixel_buffer_size.height,
        surface_texture,
    )?;

    let time = Instant::now();

    let app = Application {
        mesh,
        pixels,
        window: window.clone(),
        scale: SCALE,
        time,
        camera: Camera::new(vec3(0.0, 0.0, 0.0), Vec2::ZERO, 0.0)
    };

    start(
        event_loop,
        window,
        app,
        target_frame_time,
        max_frame_time,
    )
}
