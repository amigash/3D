mod camera;
use camera::Camera;
use nannou::{
    geom::Tri,
    prelude::*
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 800;
fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    mesh: Mesh,
    camera: Camera,
}

fn model(app: &App) -> Model {
    app.new_window().size(WIDTH, HEIGHT).view(view).build().unwrap();

    let mesh = Mesh {
        triangles: vec![
            // Front
            Tri([
                pt3(-1.0, -1.0, 1.0),
                pt3(1.0, -1.0, 1.0),
                pt3(1.0, 1.0, 1.0),
            ]),
            Tri([
                pt3(-1.0, -1.0, 1.0),
                pt3(1.0, 1.0, 1.0),
                pt3(-1.0, 1.0, 1.0),
            ]),
            // Back
            Tri([
                pt3(-1.0, -1.0, -1.0),
                pt3(1.0, 1.0, -1.0),
                pt3(1.0, -1.0, -1.0),
            ]),
            Tri([
                pt3(-1.0, -1.0, -1.0),
                pt3(-1.0, 1.0, -1.0),
                pt3(1.0, 1.0, -1.0),
            ]),
            // Right
            Tri([
                pt3(1.0, -1.0, -1.0),
                pt3(1.0, 1.0, -1.0),
                pt3(1.0, 1.0, 1.0),
            ]),
            Tri([
                pt3(1.0, -1.0, -1.0),
                pt3(1.0, 1.0, 1.0),
                pt3(1.0, -1.0, 1.0),
            ]),
            // Left
            Tri([
                pt3(-1.0, -1.0, -1.0),
                pt3(-1.0, 1.0, 1.0),
                pt3(-1.0, 1.0, -1.0),
            ]),
            Tri([
                pt3(-1.0, -1.0, -1.0),
                pt3(-1.0, -1.0, 1.0),
                pt3(-1.0, 1.0, 1.0),
            ]),
            // Top
            Tri([
                pt3(-1.0, 1.0, -1.0),
                pt3(-1.0, 1.0, 1.0),
                pt3(1.0, 1.0, 1.0),
            ]),
            Tri([
                pt3(-1.0, 1.0, -1.0),
                pt3(1.0, 1.0, 1.0),
                pt3(1.0, 1.0, -1.0),
            ]),
            // Bottom
            Tri([
                pt3(-1.0, -1.0, -1.0),
                pt3(1.0, -1.0, 1.0),
                pt3(-1.0, -1.0, 1.0),
            ]),
            Tri([
                pt3(-1.0, -1.0, -1.0),
                pt3(1.0, -1.0, -1.0),
                pt3(1.0, -1.0, 1.0),
            ]),
        ],
    };
    let position = Point3::ZERO;
    let rotation = Vec2::ZERO;
    let camera = Camera::new(position, rotation);

    Model {
        mesh,
        camera,
    }
}

struct Mesh {
    triangles: Vec<Tri>,
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    let scale = (0.5 * app.window_rect().wh()).extend(1.0);

    let transform = |point: &Point3| -> Point3 {
        let homogeneous = (*point).extend(1.0);
        let projected = model.camera.matrix() * homogeneous;
        let perspective_divided = if projected.w != 0.0 {
            projected / projected.w
        } else {
            projected
        };
        perspective_divided.truncate() * scale
    };

    for tri in &model.mesh.triangles {
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

    draw.to_frame(app, &frame).unwrap();
}

fn update(app: &App, model: &mut Model, _update: Update) {
    model.camera.update(app);
}
