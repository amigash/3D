use glam::{Vec2, Vec3A};

pub struct Draw {
    length: usize,
    depth_buffer: Vec<f32>,
}

impl Draw {
    pub fn new(width: usize, height: usize) -> Self {
        Draw {
            length: width,
            depth_buffer: vec![f32::MAX; width * height],
        }
    }

    fn pixel(&mut self, frame: &mut [u8], x: usize, y: usize, z: f32, rgb: [u8; 3]) {
        let index = x + y * self.length;
        if z >= self.depth_buffer[index] {
            return;
        }
        let rgba = [rgb[0], rgb[1], rgb[2], 255];
        if let Some(slice) = frame.get_mut(4 * index..4 * index + 4) {
            slice.copy_from_slice(&rgba);
        }
        self.depth_buffer[index] = z;
    }

    fn bounding_box(triangle: &[Vec3A; 3]) -> [usize; 4] {
        triangle
            .iter()
            .fold(
                [f32::MAX, f32::MIN, f32::MAX, f32::MIN],
                |[x_min, x_max, y_min, y_max], e| {
                    [
                        x_min.min(e.x),
                        x_max.max(e.x),
                        y_min.min(e.y),
                        y_max.max(e.y),
                    ]
                },
            )
            .map(|n| n.round() as usize)
    }
    
    pub fn fill_triangle(&mut self, frame: &mut [u8], triangle: [Vec3A; 3], rgb: [u8; 3]) {
        let [a, b, c] = triangle.map(Vec3A::truncate);
        let z_coordinates = Vec3A::from_array(triangle.map(|point| point.z));
        let [x_min, x_max, y_min, y_max] = Self::bounding_box(&triangle);
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
                    self.pixel(frame, x, y, z, rgb);
                }
            }
        }
    }

    pub fn clear_depth_buffer(&mut self) {
        self.depth_buffer.fill(f32::MAX);
    }
}
