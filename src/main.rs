mod camera;
mod draw;
mod mesh;
mod triangle;

use crate::triangle::Vertex;
use crate::{camera::Camera, draw::Draw, triangle::Triangle};
use glam::{Vec2, Vec3A};
use pixels::{Pixels, SurfaceTexture};
use std::{sync::Arc, time::Duration};
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

const OBJECT_PATH: &str = "assets/grass_block/grass_block.obj";
const CLEAR_COLOR: [u8; 4] = [110, 177, 255, 255];
const WIDTH: u32 = 2560;
const HEIGHT: u32 = 1600;
const SCALE: u32 = 2;
const TARGET_FRAME_TIME_SECONDS: f32 = 1.0 / 144.0;
const MAX_FRAME_TIME_SECONDS: f32 = 0.1;
const CAMERA_POSITION: Vec3A = Vec3A::new(0.0, 2.5, 5.0);
const CAMERA_ROTATION: Vec2 = Vec2::ZERO;

struct Application {
    mesh: Vec<Triangle>,
    pixels: Pixels,
    scale: u32,
    camera: Camera,
    draw: Draw,
    size: Vec2,
}

impl Application {
    fn clear(&mut self) {
        for pixels in self.pixels.frame_mut().chunks_exact_mut(4) {
            pixels.copy_from_slice(&CLEAR_COLOR);
        }
    }
}

impl App for Application {
    fn update(&mut self, ctx: &mut Context) -> Result<()> {
        // Keeps the mesh sorted so that closer triangles are drawn first, resulting in fewer draw calls.
        self.mesh.sort_unstable_by(|a, b| {
            a.vertices[0]
                .position
                .z
                .total_cmp(&b.vertices[0].position.z)
        });

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
        let size = self.size;
        let camera_position = self.camera.position;
        let view_projection_matrix = self.camera.view_projection_matrix();
        let forward = self.camera.forward();

        let transform = |triangle: &Triangle| {
            Triangle::from(triangle.vertices.map(|vertex| {
                let projected = view_projection_matrix * vertex.position.extend(1.0);
                let perspective_corrected_texture = vertex.texture.map(|texture| {
                    Vec3A::new(
                        texture.x,
                        texture.y,
                        1.0,
                    ) / projected.w
                });
                let perspective_divided = Vec3A::from_vec4(projected / projected.w);
                let flipped = perspective_divided.with_y(-perspective_divided.y);
                let centered = flipped + Vec3A::new(1.0, 1.0, 0.0);
                let position = centered * Vec3A::from((0.5 * size).extend(1.0));
                Vertex::new(position, vertex.normal, perspective_corrected_texture)
            }))
        };

        let is_facing_camera = |triangle: &Triangle| {
            triangle
                .normal
                .dot(camera_position - triangle.vertices[0].position)
                .is_sign_positive()
        };

        let is_ahead_of_camera = |triangle: &Triangle| {
            triangle.vertices.iter().all(|vertex| {
                forward
                    .dot(vertex.position - camera_position)
                    .is_sign_positive()
            })
        };

        let is_visible =
            |triangle: &&Triangle| is_facing_camera(triangle) && is_ahead_of_camera(triangle);

        self.clear();
        for triangle in self.mesh.iter().filter(is_visible).map(transform) {
            self.draw.fill_triangle(self.pixels.frame_mut(), &triangle);
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
                self.size = Vec2::new(width as f32, height as f32);
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta: (dx, dy) },
                ..
            } => self
                .camera
                .update_rotation(Vec2::new(-dy as f32, dx as f32)),
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
        mesh: mesh::load_from_obj_file(OBJECT_PATH)?,
        pixels: Pixels::new(width, height, SurfaceTexture::new(WIDTH, HEIGHT, &window))?,
        scale: SCALE,
        camera: Camera::new(CAMERA_POSITION, CAMERA_ROTATION),
        draw: Draw::new(width as usize, height as usize),
        size: Vec2::new(WIDTH as f32, HEIGHT as f32),
    };

    let target_frame_time = Duration::from_secs_f32(TARGET_FRAME_TIME_SECONDS);
    let max_frame_time = Duration::from_secs_f32(MAX_FRAME_TIME_SECONDS);

    start(event_loop, window, app, target_frame_time, max_frame_time)
}
