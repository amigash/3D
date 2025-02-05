use crate::{geometry::Triangle, mesh::Texture};
use glam::{FloatExt, Vec2, Vec3A};
use std::{collections::HashMap, f32::consts::FRAC_1_SQRT_2};

const LIGHT_ANGLE: Vec3A = Vec3A::new(FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0.0);
const LIGHT_MIN: f32 = 0.75;
const LIGHT_MAX: f32 = 1.00;

pub struct Draw {
    width: usize,
    height: usize,
    depth_buffer: Vec<f32>,
    pub textures: HashMap<String, Texture>,
}

impl Draw {
    pub fn new(width: usize, height: usize, textures: HashMap<String, Texture>) -> Self {
        Draw {
            width,
            height,
            depth_buffer: vec![f32::MAX; width * height],
            textures,
        }
    }

    fn pixel(&mut self, frame: &mut [u8], x: usize, y: usize, z: f32, rgba: [u8; 4]) {
        // TODO: Actual transparency handling
        if rgba[3] == 0 {
            return;
        }
        let index = x + y * self.width;
        let Some(depth) = self.depth_buffer.get_mut(index) else {
            return;
        };
        if z > *depth {
            return;
        }
        *depth = z;

        if let Some(slice) = frame.get_mut(4 * index..4 * index + 4) {
            slice.copy_from_slice(&rgba);
        }
        self.depth_buffer[index] = z;
    }

    // TODO: More efficient method than bounding box
    fn bounding_box(&self, vertices: &[Vec3A; 3]) -> [usize; 4] {
        vertices
            .iter()
            .fold(
                [(self.width - 1) as f32, 0.0, (self.height - 1) as f32, 0.0],
                |[x_min, x_max, y_min, y_max], e| {
                    [
                        x_min.min(e.x).max(0.0),
                        x_max.max(e.x).min((self.width - 1) as f32),
                        y_min.min(e.y).max(0.0),
                        y_max.max(e.y).min((self.height - 1) as f32),
                    ]
                },
            )
            .map(|n| n.round() as usize)
    }

    fn triangle_area(a: Vec2, b: Vec2, c: Vec2) -> f32 {
        // This is actually twice the triangle's area,
        // but dividing by 2 would just cancel out later anyway
        (c - a).perp_dot(b - a)
    }

    pub fn fill_triangle(&mut self, frame: &mut [u8], triangle: &Triangle) {
        let vertices = triangle.vertices.map(|v| v.position);
        let textures = triangle.vertices.map(|v| v.texture);
        let normals = triangle.vertices.map(|v| v.normal);
        let texture = self
            .textures
            .get(triangle.texture_name.as_str())
            .cloned()
            .unwrap_or_default();

        let [a, b, c] = vertices.map(Vec3A::truncate);
        let z_coordinates = Vec3A::from_array(vertices.map(|point| point.z));
        let [x_min, x_max, y_min, y_max] = self.bounding_box(&vertices);
        let area = Self::triangle_area(a, b, c);

        for y in y_min..=y_max {
            for x in x_min..=x_max {
                let point = Vec2::new(x as f32, y as f32) + 0.5;

                let bcp = Self::triangle_area(b, c, point);
                let cap = Self::triangle_area(c, a, point);
                let abp = Self::triangle_area(a, b, point);

                if (bcp < 0.0) != (cap < 0.0) || (cap < 0.0) != (abp < 0.0) {
                    continue;
                }

                let weights = Vec3A::new(bcp, cap, abp) / area;
                let z = z_coordinates.dot(weights);

                let apply_weights = |attributes: [Vec3A; 3]| {
                    attributes
                        .iter()
                        .zip(weights.to_array().iter())
                        .map(|(a, b)| a * b)
                        .sum::<Vec3A>()
                };

                let texture_coordinates = apply_weights(textures);
                let normal = apply_weights(normals);

                let scaled_texture = (texture_coordinates / texture_coordinates.z) % 1.0;
                let scaled_normal = normal / normal.z;

                let [texture_x, texture_y] = (scaled_texture
                    * Vec3A::new(texture.width as f32, texture.height as f32, 1.0))
                .truncate()
                .to_array()
                .map(|float| float as usize);

                let mut rgba = texture.get_pixel(texture_x, texture_y);

                let scaled_lighting =
                    f32::lerp(LIGHT_MIN, LIGHT_MAX, scaled_normal.dot(LIGHT_ANGLE));

                for channel in &mut rgba[0..3] {
                    *channel = (f32::from(*channel) * scaled_lighting) as u8;
                }

                self.pixel(frame, x, y, z, rgba);
            }
        }
    }

    pub fn clear_depth_buffer(&mut self) {
        self.depth_buffer.fill(0.0);
    }
}
