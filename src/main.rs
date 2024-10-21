#![feature(iter_partition_in_place)]

mod camera;
mod draw;
mod mesh;
mod geometry;

use crate::geometry::{ProjectedTriangle, ProjectedVertex};
use crate::{
    camera::Camera,
    draw::Draw,
    mesh::ObjectData,
    geometry::{Triangle, Vertex},
};
use glam::{Vec2, Vec3A, Vec4};
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

const OBJECT_PATH: &str = "assets/coconut/Wii_Coconut_Mall.obj";
const CLEAR_COLOR: [u8; 4] = [110, 177, 255, 255];
const WIDTH: u32 = 2560;
const HEIGHT: u32 = 1600;
const SCALE: u32 = 4;
const TARGET_FRAME_TIME_SECONDS: f32 = 1.0 / 144.0;
const MAX_FRAME_TIME_SECONDS: f32 = 0.1;
const CAMERA_POSITION: Vec3A = Vec3A::new(0.0, 2.5, 5.0);
const CAMERA_ROTATION: Vec2 = Vec2::ZERO;

const CLIPPING_PLANES: [Vec4; 5] = [
    Vec4::new(0.0, 0.0, 1.0, 1.0),  // Near
    Vec4::new(1.0, 0.0, 0.0, 1.0),  // Left
    Vec4::new(-1.0, 0.0, 0.0, 1.0), // Right
    Vec4::new(0.0, -1.0, 0.0, 1.0), // Top
    Vec4::new(0.0, 1.0, 0.0, 1.0),  // Bottom
];

fn intersection(plane: Vec4, a: ProjectedVertex, b: ProjectedVertex) -> ProjectedVertex {
    let s = plane.dot(a.position) / (plane.dot(a.position) - plane.dot(b.position));
    a.lerp(b, s)
}

fn clip(triangles: &mut Vec<ProjectedTriangle>) {
    for plane in CLIPPING_PLANES {
        let mut i = 0;
        let mut length = triangles.len();

        while i < length {
            let mut triangle = triangles[i].clone();
            let inside = triangle
                .vertices
                .iter_mut()
                .partition_in_place(|point| point.position.dot(plane).is_sign_positive());
            match inside {
                1 | 2 => {
                    let [a, b, c] = [4, 5, 6].map(|j| triangle.vertices[(j - inside) % 3]);
                    let [ab, ac] = [b, c].map(|point| intersection(plane, a, point));

                    if inside == 1 {
                        triangles[i].vertices = [ac, a, ab];
                        // triangles[i].texture_name = "cyan".to_string();
                    } else {
                        triangles[i].vertices = [ac, b, ab];
                        // triangles[i].texture_name = "yellow".to_string();
                        triangle.vertices = [ac, b, c];
                        // triangle.texture_name = "magenta".to_string();
                        triangles.insert(i, triangle);
                        length += 1;
                    }
                    i += 1;
                }
                3 => i += 1,
                _ => {
                    triangles.swap_remove(i);
                    length -= 1;
                }
            }
        }
    }
}

struct Application {
    mesh: Vec<Triangle>,
    pixels: Pixels,
    scale: u32,
    camera: Camera,
    draw: Draw,
    size: Vec2,
}

impl Application {
    fn clear_screen(&mut self) {
        for pixels in self.pixels.frame_mut().chunks_exact_mut(4) {
            pixels.copy_from_slice(&CLEAR_COLOR);
        }
    }
}

impl App for Application {
    fn update(&mut self, ctx: &mut Context) -> Result<()> {
        // Keeps the mesh sorted so that closer triangles are drawn first, resulting in fewer draw calls.
        let position = self.camera.position;
        self.mesh
            .sort_unstable_by_key(|triangle| position.distance(triangle.centroid) as i32);

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

        let project = |triangle: &Triangle| ProjectedTriangle {
            vertices: triangle.vertices.map(|vertex| ProjectedVertex {
                position: view_projection_matrix * vertex.position.extend(1.0),
                normal: vertex.normal,
                texture: vertex.texture,
            }),
            normal: triangle.normal,
            texture_name: triangle.texture_name.clone(),
            centroid: triangle.centroid,
        };

        let divide_and_scale = |triangle: ProjectedTriangle| Triangle {
            vertices: triangle.vertices.map(|vertex| {
                let perspective_divided = Vec3A::from_vec4(vertex.position / vertex.position.w);
                let flipped = perspective_divided.with_y(-perspective_divided.y);
                let centered = flipped + Vec3A::new(1.0, 1.0, 0.0);
                let position = centered * Vec3A::from((0.5 * size).extend(1.0));
                let texture =
                    Vec3A::new(vertex.texture.x, vertex.texture.y, 1.0) / vertex.position.w;
                Vertex {
                    position,
                    normal: vertex.normal,
                    texture,
                }
            }),
            normal: triangle.normal,
            texture_name: triangle.texture_name.clone(),
            centroid: triangle.centroid,
        };

        let is_facing_camera = |triangle: &&Triangle| {
            triangle
                .normal
                .dot(camera_position - triangle.centroid)
                .is_sign_positive()
        };

        self.clear_screen();
        let mut projected: Vec<_> = self
            .mesh
            .iter()
            .filter(is_facing_camera)
            .map(project)
            .collect();
        clip(&mut projected);
        for triangle in projected.into_iter().map(divide_and_scale) {
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
                self.draw = Draw::new(width as usize, height as usize, self.draw.textures.clone());
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

    let ObjectData {
        triangles: mesh,
        textures,
    } = mesh::load_from_obj_file(OBJECT_PATH)?;

    let app = Application {
        mesh,
        pixels: Pixels::new(width, height, SurfaceTexture::new(WIDTH, HEIGHT, &window))?,
        scale: SCALE,
        camera: Camera::new(CAMERA_POSITION, CAMERA_ROTATION),
        draw: Draw::new(width as usize, height as usize, textures),
        size: Vec2::new(WIDTH as f32, HEIGHT as f32),
    };

    let target_frame_time = Duration::from_secs_f32(TARGET_FRAME_TIME_SECONDS);
    let max_frame_time = Duration::from_secs_f32(MAX_FRAME_TIME_SECONDS);

    start(event_loop, window, app, target_frame_time, max_frame_time)
}
