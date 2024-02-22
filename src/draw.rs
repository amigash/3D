use clipline::clipline;
use glam::IVec3;

pub struct Draw {
    to_draw: Vec<(IVec3, [u8; 4])>,
    width: usize,
    height: usize,
    cleared_frame: Vec<u8>,
}

impl Draw {
    pub fn new(width: usize, height: usize) -> Self {
        Draw {
            to_draw: Vec::new(),
            width,
            height,
            cleared_frame: vec![0; width * height * 4],
        }
    }

    pub fn copy_to_frame(&mut self, frame: &mut [u8]) {
        frame.copy_from_slice(self.cleared_frame.as_slice());
        self.to_draw.reverse();
        while let Some((point, rgba)) = self.to_draw.pop() {
            let index = 4 * (point.x as usize + point.y as usize * self.width);
            // SAFETY: clipline only calls draw_pixel with valid coordinates
            unsafe { frame.get_unchecked_mut(index..index + 4) }.copy_from_slice(rgba.as_slice());
        }
    }

    pub fn pixel(&mut self, point: IVec3, rgba: [u8; 4]) {
        self.to_draw.push((point, rgba));
    }

    pub fn line(&mut self, a: IVec3, b: IVec3, rgba: [u8; 4]) {
        clipline(
            (a.truncate().into(), b.truncate().into()),
            ((0, 0), (self.width as i32 - 1, self.height as i32 - 1)),
            |x, y| {
                self.to_draw.push((IVec3::from((x, y, 0)), rgba));
            },
        );
    }

    pub fn triangle(&mut self, points: [IVec3; 3], rgba: [u8; 4]) {
        self.line(points[0], points[1], rgba);
        self.line(points[1], points[2], rgba);
        self.line(points[2], points[0], rgba);
    }
}
