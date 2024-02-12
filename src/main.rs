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

    let transform = |point: &Point3| -> Point3 {
        let projected = model.camera.project(*point);
        let scaled = 0.5 * app.window_rect().wh().extend(1.0) * projected;
        scaled
    };

    for tri in &model.mesh {
        draw.polyline()
            .points(tri.iter().map(transform))
            .color(WHITE);
    }

    let draw_axis = |axis: Vec3, color: Srgb<u8>| {
        draw.polyline()
            .color(color)
            .points([Vec3::ZERO, axis].iter().map(transform));
    };

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
