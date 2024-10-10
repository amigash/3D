#![feature(iter_partition_in_place)]

mod camera;
mod draw;
mod mesh;

use crate::{camera::Camera, draw::Draw};
use glam::{vec2, vec3a, vec4, Vec2, Vec3A, Vec4};
use itertools::Itertools;
use pixels::{Pixels, SurfaceTexture};
use std::{fs::File, sync::Arc, time::Duration, time::Instant};
use win_loop::{
    anyhow::Result,
    start,
    winit::{
        dpi::PhysicalSize,
        event::{DeviceEvent, Event, WindowEvent},
        event_loop::EventLoop,
        keyboard::{KeyCode, NamedKey},
        window::{CursorGrabMode, Fullscreen, WindowBuilder},
    },
    App, Context, InputState,
};

const WIDTH: u32 = 2560;
const HEIGHT: u32 = 1600;
const SCALE: u32 = 2;

const CLIPPING_PLANES: [Vec4; 6] = [
    vec4(0.0, 0.0, 1.0, 1.0),  // Near
    vec4(0.0, 0.0, -1.0, 1.0), // Far
    vec4(1.0, 0.0, 0.0, 1.0),  // Left
    vec4(-1.0, 0.0, 0.0, 1.0), // Right
    vec4(0.0, -1.0, 0.0, 1.0), // Top
    vec4(0.0, 1.0, 0.0, 1.0),  // Bottom
];

struct Application {
    mesh: Vec<([Vec3A; 3], Vec3A)>,
    pixels: Pixels,
    scale: u32,
    camera: Camera,
    draw: Draw,
    size: Vec2,
    time: Instant,
}

fn clip(points: &mut Vec<([Vec4; 3], Vec3A)>) {
    for plane in CLIPPING_PLANES {
        let mut i = 0;
        let mut length = points.len();
        while i < length {
            let (mut triangle, normal) = points[i];
            let inside = triangle
                .iter_mut()
                .partition_in_place(|point| point.dot(plane).is_sign_positive());
            match inside {
                1 | 2 => {
                    let [a, b, c] = [1, 2, 3].map(|j| triangle[(3 + j - inside) % 3]);
                    let [ab, ac] = [b, c].map(|point| {
                        a + (point - a) * plane.dot(a) / (plane.dot(a) - plane.dot(point))
                    });
                    if inside == 1 {
                        points[i] = ([ac, a, ab], normal);
                    } else {
                        points[i] = ([ac, b, ab], normal);
                        points.push(([c, b, ac], normal));
                        length += 1;
                    }
                    i += 1;
                }
                3 => i += 1,
                _ => {
                    points.swap_remove(i);
                    length -= 1;
                }
            }
        }
    }
}

impl Application {
    fn transform(&self, point: Vec4) -> Vec3A {
        let perspective_divided = Vec3A::from_vec4(point / point.w);
        let flipped = perspective_divided * vec3a(1.0, -1.0, 1.0);
        let centered = flipped + vec3a(1.0, 1.0, 0.0);
        centered * Vec3A::from((0.5 * (self.size - Vec2::ONE)).extend(1.0))
    }

    fn rgb_from_normal(&self, normal: Vec3A) -> [u8; 3] {
        let speed = 0.25;
        let angle = self.time.elapsed().as_secs_f32() * speed;
        let light_direction = vec3a(angle.cos(), angle.sin(), 0.0);
        let intensity = normal.dot(light_direction).max(0.01);
        let color = (intensity * 255.0) as u8;
        [color; 3]
    }
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
        let mut view_space = self
            .mesh
            .iter()
            .sorted_by(|(a, _), (b, _)| a[0].z.total_cmp(&b[0].z))
            .filter(|&&(triangle, normal)| {
                normal
                    .dot(self.camera.position - triangle[0])
                    .is_sign_positive()
            })
            .map(|&(triangle, normal)| {
                (
                    triangle.map(|p| self.camera.view_projection_matrix() * p.extend(1.0)),
                    normal,
                )
            })
            .collect();
        clip(&mut view_space);

        self.pixels.frame_mut().fill(0);
        for (triangle, normal) in view_space {
            let screen_space = triangle.map(|point| self.transform(point));
            let rgb = self.rgb_from_normal(normal);
            self.draw
                .fill_triangle(self.pixels.frame_mut(), screen_space, rgb);
        }
        self.draw.clear_depth_buffer();

        self.pixels.render()?;

        Ok(())
    }

    fn handle(&mut self, event: &Event<()>) -> Result<()> {
        match *event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                let (width, height) = (size.width / self.scale, size.height / self.scale);
                self.pixels.resize_surface(size.width, size.height)?;
                self.pixels.resize_buffer(width, height)?;
                self.draw = Draw::new(width as usize, height as usize);
                self.camera.aspect_ratio = width as f32 / height as f32;
                self.size = vec2(width as f32, height as f32);
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta: (dx, dy) },
                ..
            } => self.camera.update_rotation(vec2(-dy as f32, dx as f32)),
            _ => (),
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;

    let window = Arc::new(
        WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
            .with_fullscreen(Some(Fullscreen::Borderless(None)))
            .build(&event_loop)?,
    );

    window.set_cursor_grab(CursorGrabMode::Confined)?;
    window.set_cursor_visible(false);

    let [width, height] = [WIDTH / SCALE, HEIGHT / SCALE];

    let app = Application {
        mesh: mesh::load_from_obj_file(File::open("assets/teapot.obj")?)?,
        pixels: Pixels::new(width, height, SurfaceTexture::new(WIDTH, HEIGHT, &window))?,
        scale: SCALE,
        camera: Camera::new(vec3a(0.0, 2.5, 5.0), vec2(0.0, 0.0)),
        draw: Draw::new(width as usize, height as usize),
        size: vec2(WIDTH as f32, HEIGHT as f32),
        time: Instant::now(),
    };

    let target_frame_time = Duration::from_secs_f32(1. / 144.); // 144 fps
    let max_frame_time = Duration::from_secs_f32(0.1);

    start(event_loop, window, app, target_frame_time, max_frame_time)
}
