use crate::triangle::Triangle;
use glam::{Vec2, Vec3A};

pub struct Draw {
    width: usize,
    height: usize,
    depth_buffer: Vec<f32>,
}

impl Draw {
    pub fn new(width: usize, height: usize) -> Self {
        Draw {
            width,
            height,
            depth_buffer: vec![0.0; width * height],
        }
    }

    fn pixel(&mut self, frame: &mut [u8], x: usize, y: usize, z: f32, rgb: [u8; 3]) {
        let index = x + y * self.width;
        if z.recip() < self.depth_buffer[index] {
            return;
        }
        let rgba = [rgb[0], rgb[1], rgb[2], 255];
        if let Some(slice) = frame.get_mut(4 * index..4 * index + 4) {
            slice.copy_from_slice(&rgba);
        }
        self.depth_buffer[index] = z.recip();
    }

    fn bounding_box(&self, vertices: &[Vec3A; 3]) -> [usize; 4] {
        vertices
            .iter()
            .fold(
                [(self.width as f32) - 1.0, 0.0, (self.height - 1) as f32, 0.0],
                |[x_min, x_max, y_min, y_max], e| {
                    [
                        x_min.min(e.x).max(0.0),
                        x_max.max(e.x).min((self.width as f32) - 1.0),
                        y_min.min(e.y).max(0.0),
                        y_max.max(e.y).min((self.height as f32) - 1.0),
                    ]
                },
            )
            .map(|n| n.round() as usize)
    }

    pub fn fill_triangle(&mut self, frame: &mut [u8], triangle: &Triangle) {
        let vertices = triangle.vertices();
        let [a, b, c] = vertices.map(Vec3A::truncate);
        let z_coordinates = Vec3A::from_array(vertices.map(|point| point.z));
        let [x_min, x_max, y_min, y_max] = self.bounding_box(&vertices);
        let [c_b, b_a, a_c] = [c.perp_dot(b), b.perp_dot(a), a.perp_dot(c)];
        let inverse_area = (c_b + b_a + a_c).recip();

        for y in y_min..=y_max {
            for x in x_min..=x_max {
                let point = Vec2::new(x as f32, y as f32) + 0.5;
                let [p_a, p_b, p_c] = [a, b, c].map(|vertex| point.perp_dot(vertex));
                let sub_triangle_areas =
                    Vec3A::new(p_c + c_b - p_b, p_a + a_c - p_c, p_b + b_a - p_a);
                let weights = inverse_area * sub_triangle_areas;
                if weights
                    .as_ref()
                    .iter()
                    .all(|weight| weight.is_sign_positive())
                {
                    let z = weights.dot(z_coordinates);
                    let rgb = (Vec3A::splat(255.0) * weights).to_array().map(|n| n as _);
                    self.pixel(frame, x, y, z, rgb);
                }
            }
        }
    }

    pub fn clear_depth_buffer(&mut self) {
        self.depth_buffer.fill(0.0);
    }
}
