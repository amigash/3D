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
    projection_matrix: Mat4,
}

impl Camera {
    fn new(position: Point3, rotation: Vec2) -> Self {
        Camera {
            position,
            rotation,
            projection_matrix: Mat4::perspective_rh(f32::FRAC_PI_2(), 1.0, 0.1, 100.0)
        }
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
        self.projection_matrix * Mat4::look_to_rh(self.position, forward, up)
    }

    fn update(&mut self, app: &App) {
        let mut rotation = Point2::ZERO;
        let mut translation = Vec3::ZERO;

        let right = self.right();
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

        self.position += translation * SPEED;
        self.rotation += rotation * SENSITIVITY;
        self.rotation.x = self.rotation.x.clamp(0.99 * -f32::FRAC_PI_2(), 0.99 * f32::FRAC_PI_2());
        self.rotation.y = self.rotation.y.rem_euclid(f32::TAU());
    }
}

struct Model {
    mesh: Mesh,
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
