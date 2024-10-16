use crate::mesh::Texture;
use crate::triangle::Triangle;
use glam::{Vec2, Vec3A};
use std::collections::HashMap;

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
            depth_buffer: vec![0.0; width * height],
            textures,
        }
    }

    fn pixel(&mut self, frame: &mut [u8], x: usize, y: usize, z: f32, rgba: [u8; 4]) {
        let index = x + y * self.width;
        if let Some(slice) = frame.get_mut(4 * index..4 * index + 4) {
            slice.copy_from_slice(&rgba);
        }
    }

    fn bounding_box(&self, vertices: &[Vec3A; 3]) -> [usize; 4] {
        vertices
            .iter()
            .fold(
                [
                    (self.width as f32) - 1.0,
                    0.0,
                    (self.height - 1) as f32,
                    0.0,
                ],
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

    fn triangle_area(a: Vec2, b: Vec2, c: Vec2) -> f32 {
        // This is actually twice the triangle's area,
        // but dividing by 2 would just cancel out later anyway
        (c - a).perp_dot(b - a)
    }

    pub fn fill_triangle(&mut self, frame: &mut [u8], triangle: &Triangle) {
        let vertices = triangle.vertices.map(|v| v.position);
        let textures = triangle.vertices.map(|v| v.texture);
        let texture_name = triangle.texture_name.as_str();
        let [a, b, c] = vertices.map(Vec3A::truncate);
        let z_coordinates = Vec3A::from_array(vertices.map(|point| point.z));
        let [x_min, x_max, y_min, y_max] = self.bounding_box(&vertices);
        let area = Self::triangle_area(a, b, c);

        let texture = self.textures[texture_name].clone();

        for y in y_min..=y_max {
            for x in x_min..=x_max {
                let point = Vec2::new(x as f32, y as f32) + 0.5;
                let w_a = Self::triangle_area(b, c, point);
                let w_b = Self::triangle_area(c, a, point);
                let w_c = Self::triangle_area(a, b, point);

                if w_a < 0.0 || w_b < 0.0 || w_c < 0.0 {
                    continue;
                }

                let weights = Vec3A::new(w_a, w_b, w_c) / area;
                let z = z_coordinates.dot(weights);

                let texture_coordinates = textures
                    .iter()
                    .zip(weights.to_array().iter())
                    .map(|(a, b)| a * b)
                    .sum::<Vec3A>();

                let scaled_texture = texture_coordinates / texture_coordinates.z;

                let texture_x = (scaled_texture.x * texture.width as f32) as usize;
                let texture_y = (scaled_texture.y * texture.height as f32) as usize;

                let rgba = texture.get_pixel(texture_x, texture_y);

                self.pixel(frame, x, y, z, rgba);
            }
        }
    }

    pub fn clear_depth_buffer(&mut self) {
        self.depth_buffer.fill(0.0);
    }
}
