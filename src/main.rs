use nannou::geom::Tri;
use nannou::prelude::*;

const SPEED: f32 = 0.1;
const SENSITIVITY: f32 = 0.03;

fn main() {
    nannou::app(model).update(update).run();
}

struct Camera {
    position: Point3,
    rotation: Vec2,
}

impl Camera {
    fn new(position: Point3, rotation: Vec2) -> Self {
        Camera { position, rotation }
    }

    fn forward(&self) -> Vec3 {
        let (x_sin, x_cos) = self.rotation.x.sin_cos();
        let (y_sin, y_cos) = self.rotation.y.sin_cos();
        Vec3::new(y_cos * x_cos, x_sin, y_sin * x_cos)
    }

    fn right(&self) -> Vec3 {
        self.forward().cross(Vec3::Y).normalize()
    }

    fn matrix(&self) -> Mat4 {
        let forward = self.forward();
        let right = self.right();
        let up = right.cross(forward).normalize();
        Mat4::look_to_rh(self.position, forward, up)
    }
}

struct Model {
    mesh: Mesh,
    projection_matrix: Mat4,
    camera: Camera,
}

fn model(app: &App) -> Model {
    app.new_window().size(800, 800).view(view).build().unwrap();

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
    let camera = Camera::new(position, rotation);

    Model {
        mesh,
        projection_matrix,
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
    let view_matrix = model.camera.matrix();

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
    let mut rotation = Point2::ZERO;
    let mut translation = Vec3::ZERO;

    let right = model.camera.right();
    let forward = right.cross(-Vec3::Y).normalize(); // "flat" forward vector -- not affected by pitch

    for key in app.keys.down.iter() {
        match key {
            Key::W => translation += forward,
            Key::S => translation -= forward,
            Key::A => translation -= right,
            Key::D => translation += right,
            Key::Space => translation.y += 1.0,
            Key::LShift => translation.y -= 1.0,
            Key::Left => rotation.y -= 1.0,
            Key::Right => rotation.y += 1.0,
            Key::Up => rotation.x += 1.0,
            Key::Down => rotation.x -= 1.0,
            _ => {}
        }
    }

    model.camera.position += translation * SPEED;
    model.camera.rotation += rotation * SENSITIVITY;
    model.camera.rotation.x = model
        .camera
        .rotation
        .x
        .clamp(-f32::FRAC_PI_2(), f32::FRAC_PI_2());
    model.camera.rotation.y = model.camera.rotation.y.rem_euclid(f32::TAU());
}
