#![warn(clippy::pedantic)]
mod draw;
mod camera;
mod triangle;

use std::{
    sync::Arc,
    time::Duration
};
use glam::Vec3;
use win_loop::{
    *,
    anyhow::Result,
    winit::{
        dpi::PhysicalSize,
        event_loop::EventLoop,
        keyboard::NamedKey,
        window::WindowBuilder
    }
};
use pixels::*;
use win_loop::winit::event::{Event, WindowEvent};
use win_loop::winit::window::Window;
use crate::{draw::*, triangle::*};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;
const SCALE: u32 = 5;

const fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

struct Application {
    mesh: Vec<Triangle>,
    pixels: Pixels,
    window: Arc<Window>,
    tri: Triangle,
    scale: u32
}


impl App for Application {
    fn update(&mut self, ctx: &mut Context) -> Result<()> {

        if ctx.input.is_logical_key_pressed(NamedKey::Escape) {
            ctx.exit();
        }
        for (key, input_state) in ctx.input.logical_keys() {
            let InputState::Down = input_state else {
                continue;
            };
            let mut movement = vec3(0.0, 0.0, 0.0);

            match key {
                NamedKey::ArrowUp => movement += vec3(0.0, -1.0, 0.0),
                NamedKey::ArrowDown => movement += vec3(0.0, 1.0, 0.0),
                NamedKey::ArrowLeft => movement += vec3(-1.0, 0.0, 0.0),
                NamedKey::ArrowRight => movement += vec3(1.0, 0.0, 0.0),
                _ => {}
            };
            self.tri += movement;
        }

        Ok(())
    }

    fn render(&mut self, _blending_factor: f64) -> Result<()> {
        let frame = self.pixels.frame_mut();
        let tri = &self.tri;
        let size = self.window.inner_size();
        clear(frame);
        triangle(frame,
                 (size.width / self.scale) as i32, (size.height / self.scale) as i32,
            tri.a.x as i32, tri.a.y as i32,
            tri.b.x as i32, tri.b.y as i32,
            tri.c.x as i32, tri.c.y as i32,
            [255, 255, 255, 255]);
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
                    },
                    _ => {}
                }
            }
            _ => {}
        }
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

    let event_loop = EventLoop::new()?;

    let window = Arc::new(WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
        .build(&event_loop)?);


    let target_frame_time = Duration::from_secs_f32(1. / 120.); // 60 fps
    let max_frame_time = Duration::from_secs_f32(0.1);

    let pixel_buffer_size = PhysicalSize::new(WIDTH / SCALE, HEIGHT / SCALE);
    let surface_texture = SurfaceTexture::new(window.inner_size().width, window.inner_size().height, &window);

    let pixels = Pixels::new(
        pixel_buffer_size.width,
        pixel_buffer_size.height,
        surface_texture,
    )?;

    let tri = tri(
        vec3(10.0, 10.0, 0.0),
        vec3(30.0, 15.0, 0.0),
        vec3(10.0, 15.0, 0.0)
    );


    let app = Application {
        mesh,
        pixels,
        window: window.clone(),
        tri,
        scale: SCALE
    };

    start(
        event_loop,
        window,
        app,
        target_frame_time,
        max_frame_time,
    )
}
