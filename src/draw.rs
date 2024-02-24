use glam::{IVec3, ivec3};
use crate::line::plot_line;

pub struct Draw {
    to_draw: Vec<(IVec3, [u8; 4])>,
    width: usize,
    cleared_frame: Vec<u8>,
}

impl Draw {
    pub fn new(width: usize, height: usize) -> Self {
        Draw {
            to_draw: Vec::new(),
            width,
            cleared_frame: vec![0; width * height * 4],
        }
    }

    pub fn copy_to_frame(&mut self, frame: &mut [u8]) {
        frame.copy_from_slice(self.cleared_frame.as_slice());
        while let Some((point, rgba)) = self.to_draw.pop() {
            let index = 4 * (point.x as usize + point.y as usize * self.width);
            // SAFETY: Caller must ensure that index is within bounds.
            unsafe { frame.get_unchecked_mut(index..index + 4) }.copy_from_slice(rgba.as_slice());
        }
    }

    pub fn pixel(&mut self, point: IVec3, rgba: [u8; 4]) {
        self.to_draw.push((point, rgba));
    }

    pub fn line(&mut self, a: IVec3, b: IVec3, rgba: [u8; 4]) {
        plot_line(a.x, a.y, b.x, b.y, |x, y| self.to_draw.push((ivec3(x, y, 0), rgba)));
    }

    pub fn triangle(&mut self, points: [IVec3; 3], rgba: [u8; 4]) {
        self.line(points[0], points[1], rgba);
        self.line(points[1], points[2], rgba);
        self.line(points[2], points[0], rgba);
    }

    fn draw_filled_flat_triangle(&mut self, apex: IVec3, flat_left_x: i32, flat_right_x: i32, flat_y: i32, rgba: [u8; 4]) {
        let find_edge_points = |flat_x: i32| {
            let mut edge_points = vec![];
            let mut last_point = apex;
            plot_line(apex.x, apex.y, flat_x, flat_y, |x, y| {
                if y != last_point.y {
                    edge_points.push(last_point);
                }
                last_point = ivec3(x, y, 0);
            });
            if !edge_points.contains(&last_point) {
                edge_points.push(last_point);
            }
            edge_points
        };
        let left_points = find_edge_points(flat_left_x);
        let right_points = find_edge_points(flat_right_x);

        left_points.into_iter().zip(right_points).for_each(|(start, end)| {
            self.line(start, end, rgba);
        });
    }

    pub fn fill_triangle(&mut self, mut points: [IVec3; 3], rgba: [u8; 4]) {
        points.sort_unstable_by_key(|t| t.y);
        let [a, b, c] = points;
        match [a.y == b.y, b.y == c.y] {
            [true, true] => self.line(a, c, rgba),
            [true, false] => self.draw_filled_flat_triangle(c, a.x, b.x, a.y, rgba),
            [false, true] => self.draw_filled_flat_triangle(a, b.x, c.x, b.y, rgba),
            [false, false] => {
                let ac_x = (b.y - a.y) * (a.x - c.x) / (a.y - c.y) + a.x;
                self.draw_filled_flat_triangle(a, b.x, ac_x, b.y, rgba);
                self.draw_filled_flat_triangle(c, b.x, ac_x, b.y, rgba);
            }
        }
    }

}
