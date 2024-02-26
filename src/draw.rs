use glam::{Vec3A, vec3a};
use crate::line::plot_line_with_depth;

pub struct Draw {
    to_draw: Vec<((usize, usize, f32), [u8; 4])>,
    width: usize,
    cleared_frame: Vec<u8>,
    depth_buffer: Vec<f32>,
    cleared_depth_buffer: Vec<f32>,
}

impl Draw {
    pub fn new(width: usize, height: usize) -> Self {
        Draw {
            to_draw: Vec::new(),
            width,
            cleared_frame: vec![0; width * height * 4],
            depth_buffer: vec![f32::MAX; width * height],
            cleared_depth_buffer: vec![f32::MAX; width * height],
        }
    }

    pub fn copy_depth_buffer_to_frame(&mut self, frame: &mut [u8]) {
        let (min_depth, max_depth) = self.depth_buffer.iter()
            .filter(|&&depth| depth < f32::MAX)
            .fold((f32::MAX, 0.0_f32), |(min, max), &depth| {
                (min.min(depth), max.max(depth))
            });

        for (i, &depth) in self.depth_buffer.iter().enumerate() {
            let normalized_depth = if depth < f32::MAX {
                255 - (((depth - min_depth) / (max_depth - min_depth)) * 255.0) as u8
            } else {
                0
            };

            let index = i * 4;
            frame[index..index + 4].copy_from_slice(&[normalized_depth, normalized_depth, normalized_depth, 255]);
        }
        self.to_draw.clear();
        self.depth_buffer.copy_from_slice(self.cleared_depth_buffer.as_slice());
    }

    pub fn copy_to_frame(&mut self, frame: &mut [u8]) {
        frame.copy_from_slice(self.cleared_frame.as_slice());
        self.to_draw.reverse();

        while let Some(((x, y, _), rgba)) = self.to_draw.pop() {
            let index = 4 * (x + y * self.width);
            // SAFETY: Caller must ensure that index is within bounds.
            unsafe { frame.get_unchecked_mut(index..index + 4) }.copy_from_slice(&rgba);
        }
        self.depth_buffer.copy_from_slice(self.cleared_depth_buffer.as_slice());
    }

    fn pixel(&mut self, x: usize, y: usize, z: f32, rgba: [u8; 4]) {
        if z < self.depth_buffer[x + y * self.width] {
            self.to_draw.push(((x, y, z), rgba));
            self.depth_buffer[x + y * self.width] = z;
        }
    }

    pub fn line(&mut self, a: Vec3A, b: Vec3A, rgba: [u8; 4]) {
        plot_line_with_depth(a.x as i32, a.y as i32, a.z, b.x as i32, b.y as i32, b.z, |x, y, z| self.pixel(x as usize, y as usize, z, rgba));
    }

    pub fn triangle(&mut self, points: [Vec3A; 3], rgba: [u8; 4]) {
        self.line(points[0], points[1], rgba);
        self.line(points[1], points[2], rgba);
        self.line(points[2], points[0], rgba);
    }

    fn draw_filled_flat_triangle(&mut self, apex: Vec3A, b: Vec3A, c: Vec3A, rgba: [u8; 4]) {
        let find_edge_points = |end: Vec3A| {
            let mut edge_points = vec![];
            let mut last_point = apex;
            plot_line_with_depth(apex.x as i32, apex.y as i32, apex.z, end.x as i32, end.y as i32, end.z, |x, y, z| {
                if y != last_point.y as i32 {
                    edge_points.push(last_point);
                }
                last_point = vec3a(x as f32, y as f32, z);
            });
            if !edge_points.contains(&last_point) {
                edge_points.push(last_point);
            }
            edge_points
        };
        let left_points = find_edge_points(b);
        let right_points = find_edge_points(c);

        for (start, end) in left_points.into_iter().zip(right_points) {
            self.line(start, end, rgba);
        };
    }

    pub fn fill_triangle(&mut self, mut points: [Vec3A; 3], rgba: [u8; 4]) {
        points.sort_unstable_by(|a, b| f32::total_cmp(&a.y, &b.y));
        let [a, b, c] = points;
        match [(a.y - b.y).abs() < f32::EPSILON, (b.y - c.y).abs() < f32::EPSILON] { // [a.y == b.y, b.y == c.y]
            [true, true] => self.line(a, c, rgba),
            [true, false] => self.draw_filled_flat_triangle(c, a, b, rgba),
            [false, true] => self.draw_filled_flat_triangle(a, b, c, rgba),
            [false, false] => {
                let divisor = (a.y - b.y) / (c.y - a.y);
                let d_x = (a.x - c.x) * divisor + a.x;
                let d_z = (a.z - c.z) * divisor + a.z;
                let d = vec3a(d_x, b.y, d_z);
                self.draw_filled_flat_triangle(a, b, d, rgba);
                self.draw_filled_flat_triangle(c, b, d, rgba);
            }
        }
    }

}
