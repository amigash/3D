<<<<<<< HEAD
mod camera;
mod mesh;

use std::fs::File;
use camera::Camera;
use nannou::{
    geom::Tri,
    prelude::*
};
use nannou::winit::event::DeviceEvent;
use nannou::winit::window::CursorGrabMode;
use crate::mesh::mesh_from_obj_file;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;
fn main() {
    nannou::app(model).update(update).event(event).run();
}

struct Model {
    mesh: Vec<Tri>,
    camera: Camera,
}

fn surface_normal(tri: &Tri) -> Vec3 {
    let a = tri[1] - tri[0];
    let b = tri[2] - tri[0];
    a.cross(b).normalize()
}

fn model(app: &App) -> Model {
    app
        .new_window()
        .decorations(false)
        .fullscreen()
        .size(WIDTH, HEIGHT)
        .view(view)
        .build()
        .unwrap();

    let window = app.main_window();
    window.set_cursor_visible(false);
    window.winit_window().set_cursor_grab(CursorGrabMode::Confined).unwrap(); // nannou's window only allows Locked and None

    let mesh = mesh_from_obj_file(File::open("assets/teapot.obj").unwrap()).unwrap();
    let position = Point3::ZERO;
    let rotation = Vec2::ZERO;
    let camera = Camera::new(position, rotation, WIDTH as f32 / HEIGHT as f32);

    Model {
        mesh,
        camera,
    }
}


fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    let matrix = model.camera.matrix();
    let scale_factor = 0.5 * app.window_rect().wh().extend(1.0);

    let transform = |point: &Point3| {
        let homogeneous = point.extend(1.0);
        let projected = matrix * homogeneous;
        let perspective_divided = if projected.w == 0.0 {
            projected
        } else {
            projected / projected.w
        };
        let scaled = scale_factor * perspective_divided.truncate();
        scaled
    };

    let is_visible = |tri: &&Tri| {
        let normal = surface_normal(tri);
        let view_vector = model.camera.position - tri.centroid();
        normal.dot(view_vector) >= 0.0
    };

    let draw_axis = |axis: Vec3, color: Srgb<u8>| {
        draw.polyline()
            .color(color)
            .points([Vec3::ZERO, axis].iter().map(transform));
    };

    for tri in model.mesh.iter().filter(is_visible) {
        draw.polyline()
            .points(tri.iter().map(transform))
            .color(WHITE);

        draw.line().xyz(tri.centroid().into());
    }

    draw_axis(Vec3::X, RED);
    draw_axis(Vec3::Y, GREEN);
    draw_axis(Vec3::Z, BLUE);

    // crosshair
    draw.ellipse()
        .wh(Vec2::ZERO)
        .color(WHITE)
        .radius(1.5);

    draw.to_frame(app, &frame).unwrap();
}

fn update(app: &App, model: &mut Model, _update: Update) {
    model.camera.update(&app.keys);
}

fn event(_app: &App, model: &mut Model, event: Event) {
    match event {
        Event::WindowEvent {
            simple: Some(Resized(size)),
            ..
        } => model.camera.aspect_ratio = size.x / size.y,
        Event::DeviceEvent(_, DeviceEvent::MouseMotion { delta: (dx, dy) }, ..) => {
            model.camera.update_rotation(vec2(-dy as f32, dx as f32));
        }
        _ => {}
    }
}
=======
#![warn(clippy::pedantic)]
extern crate core;

mod camera;
mod draw;
mod mesh;
mod triangle;
mod line;

use crate::{camera::Camera, draw::Draw, triangle::Triangle};
use glam::{ivec2, vec2, vec3a, Vec3A, Vec4};
use pixels::{Pixels, SurfaceTexture};
use std::{
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

fn rgba_from_normal(triangle: &Triangle, light: Vec3A) -> [u8; 4] {
    let intensity = triangle.normal.dot(light).max(0.01);
    let color = (intensity * 255.0) as u8;
    [color, color, color, 255]
}

impl App for Application {
    fn update(&mut self, ctx: &mut Context) -> Result<()> {
        if ctx.input.is_logical_key_pressed(NamedKey::Escape) {
            ctx.exit();
        }

        self.mesh.sort_unstable_by_key(|t| t.centroid().z as i32);

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
            ivec2(inner_size.width, inner_size.height) / (SCALE as i32)
        };


        let matrix = self.camera.matrix();
        let scale_factor = Vec3A::from((0.5 * size.as_vec2()).extend(1.0));

        let project = |point: Vec3A| matrix * point.extend(1.0);
        let ahead_of = |point: &Vec4| point.z > 0.01;

        let transform = |point: Vec4| {
            let perspective_divided= Vec3A::from(point / point.w);
            let flipped = perspective_divided * vec3a(1.0, -1.0, 1.0);
            let centered = flipped + vec3a(1.0, 1.0, 0.0);
            let scaled = centered * scale_factor;
            scaled
        };

        let is_on_screen =
            |point: &Vec3A| point.x > 0.0 && point.y > 0.0 && point.x < size.x as f32 && point.y < size.y as f32;

        let is_visible = |triangle: &&Triangle| {
            triangle
                .normal
                .dot(self.camera.position - triangle.centroid())
                >= 0.0
        };

        let angle = (10.0 * self.time.elapsed().as_secs_f32().sin()).to_radians();
        let rotated_light = glam::Mat3::from_rotation_z(angle) * Vec3A::Y;

        for triangle in self
            .mesh
            .iter()
            .filter(is_visible)
        {
            let points = triangle.points.map(project);
            if !points.iter().all(ahead_of) { continue; }
            let transformed_points = points.map(transform);
            if !transformed_points.iter().all(is_on_screen) { continue; }

            self.draw.fill_triangle(transformed_points, rgba_from_normal(triangle, rotated_light));
        }

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
    let mesh = mesh::load_from_obj_file(File::open("assets/heavy.obj")?)?;

    let event_loop = EventLoop::new()?;

    let window = Arc::new(
        WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
            .with_fullscreen(Some(Fullscreen::Borderless(None)))
            .build(&event_loop)?,
    );

    window.set_cursor_grab(CursorGrabMode::Confined)?;
    window.set_cursor_visible(false);

    let target_frame_time = Duration::from_secs_f32(1. / 144.); // 144 fps
    let max_frame_time = Duration::from_secs_f32(0.1);

    let [width, height] = [WIDTH / SCALE, HEIGHT / SCALE];
    let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, &window);

    let pixels = Pixels::new(
        width,
        height,
        surface_texture,
    )?;

    let draw = Draw::new(
        width as usize,
        height as usize,
    );

    let time = Instant::now();

    let app = Application {
        mesh,
        pixels,
        window: window.clone(),
        scale: SCALE,
        time,
        camera: Camera::new(vec3a(0.0, 2.5, 5.0), vec2(0.0, -std::f32::consts::FRAC_PI_2), 0.0),
        draw,
    };

    start(event_loop, window, app, target_frame_time, max_frame_time)
}
>>>>>>> new-repo/main
