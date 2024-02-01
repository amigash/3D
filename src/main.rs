use nannou::geom::Tri;
use nannou::prelude::*;

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    mesh: Mesh,
    projection_matrix: Mat4,
    position: Point3,
    rotation: Vec2,
}

fn model(app: &App) -> Model {
    app.new_window()
        .size(800, 800)
        .view(view)
        .build()
        .unwrap();

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
    let projection_matrix = Mat4::perspective_rh(f32::FRAC_PI_2(), 1.0, 0.1, 100.0);
    let position = Point3::ZERO;
    let rotation = Vec2::ZERO;

    Model {
        mesh,
        projection_matrix,
        position,
        rotation,
    }
}

struct Mesh {
    triangles: Vec<Tri>,
}

fn view(app: &App, model: &Model, frame: Frame) {

    let draw = app.draw();
    draw.background().color(BLACK);

    let scale = (0.5 * app.window_rect().wh()).extend(1.0);
    let camera_rotation =
        Mat4::from_rotation_x(model.rotation.x) * Mat4::from_rotation_y(model.rotation.y);
    let camera_translation = Mat4::from_translation(-model.position);
    let view_matrix = camera_rotation.transpose() * camera_translation;

    let transform = |point: &Point3| -> Point3 {
        let homogeneous = Point3::from(*point).extend(1.0);
        let camera_transformed = view_matrix * homogeneous;
        let projected = model.projection_matrix * camera_transformed;
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

    draw.polyline()
        .points([Vec3::ZERO, Vec3::X].iter().map(transform))
        .color(RED);
    draw.polyline()
        .points([Vec3::ZERO, Vec3::Y].iter().map(transform))
        .color(GREEN);
    draw.polyline()
        .points([Vec3::ZERO, Vec3::Z].iter().map(transform))
        .color(BLUE);

    draw.to_frame(app, &frame).unwrap();
}

fn update(app: &App, model: &mut Model, _update: Update) {
    update_camera(app, model);
}

fn update_camera(app: &App, model: &mut Model) {
    let up = Vec3::Y;
    let forward = (Vec3::Z * -1.0).normalize();

    let mut translation = Vec3::ZERO;
    let mut rotation = Point2::ZERO;

    for key in app.keys.down.iter() {
        match key {
            Key::W => translation.z -= 0.1,
            Key::S => translation.z += 0.1,
            Key::A => translation.x -= 0.1,
            Key::D => translation.x += 0.1,
            Key::Space => translation.y += 0.1,
            Key::LShift => translation.y -= 0.1,
            Key::Left => rotation.y += 0.05,
            Key::Right => rotation.y -= 0.05,
            Key::Up => rotation.x += 0.05,
            Key::Down => rotation.x -= 0.05,
            _ => {}
        }
    }



}

