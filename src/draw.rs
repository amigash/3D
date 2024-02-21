use clipline::clipline;
use glam::{IVec2};

pub struct Draw {
    to_draw: Vec<(IVec2, [u8; 4])>,
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

    pub fn pixel(&mut self, point: IVec2, rgba: [u8; 4]) {
        self.to_draw.push((point, rgba));
    }

    pub fn line(&mut self, a: IVec2, b: IVec2, rgba: [u8; 4]) {
        clipline(
            (a.into(), b.into()),
            ((0, 0), (self.width as i32 - 1, self.height as i32 - 1)),
            |x, y| {
                self.to_draw.push((IVec2::from((x, y)), rgba));
            },
        );
    }

    pub fn triangle(&mut self, a: IVec2, b: IVec2, c: IVec2, rgba: [u8; 4]) {
        self.line(a, b, rgba);
        self.line(b, c, rgba);
        self.line(c, a, rgba);
    }
}

