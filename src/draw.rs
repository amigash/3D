use clipline::clipline;
use glam::{IVec2, UVec2};

pub fn pixel(
    frame: &mut [u8],
    width: i32,
    point: IVec2,
    rgba: [u8; 4]
) {
    let index = 4 * (point.x + point.y * width) as usize;
    // SAFETY: Clipline only calls draw_pixel with valid coordinates
    unsafe {frame.get_unchecked_mut(index..index + 4)}.copy_from_slice(&rgba);
}

pub fn clear(frame: &mut [u8]) {
    for pixel in frame.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[0, 0, 0, 0]);
    }
}

#[allow(dead_code)]
pub fn clear_rectangle(frame: &mut [u8], width: i32, height: i32, x: i32, y: i32, w: i32, h: i32) {
    debug_assert!(x >= 0 && y >= 0 && x + w <= width && y + h <= height);
    for j in y..y + h {
        for i in x..x + w {
            pixel(frame, width, IVec2::from((i, j)), [0, 0, 0, 0]);
        }
    }

}

pub fn line(
    frame: &mut [u8],
    size: IVec2,
    a: IVec2,
    b: IVec2,
    rgba: [u8; 4]
) {
    clipline(
        (a.into(), b.into()),
        ((0, 0), (size - 1).into()),
        |x, y| {
            pixel(frame, size.x, IVec2::from((x, y)), rgba);
        },
    );
}

pub fn triangle(
    frame: &mut [u8],
    size: IVec2,
    a: IVec2,
    b: IVec2,
    c: IVec2,
    rgba: [u8; 4]
) {
    line(frame, size, a, b, rgba);
    line(frame, size, b, c, rgba);
    line(frame, size, c, a, rgba);
}

struct Draw<const N: usize, const M: usize> {
    frame: [u8; N * M],
    dirty: Option<UVec2>
}

impl<const N: usize, const M: usize> Draw<N, M> {
    fn to_frame(&self, frame: &mut [u8], width: usize, height: usize) {
        if width == N && height == M {
            frame.copy_from_slice(&self.frame);
        }
    }

    fn clear(&mut self) {
        for pixel in self.frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0, 0, 0, 0]);
        }
    }

    fn pixel(&mut self, point: IVec2, rgba: [u8; 4]) {
        let index = 4 * (point.x + point.y * N as i32) as usize;
        // SAFETY: Clipline only calls draw_pixel with valid coordinates
        unsafe {self.frame.get_unchecked_mut(index..index + 4)}.copy_from_slice(&rgba);

    }

    fn line(&mut self, a: IVec2, b: IVec2, rgba: [u8; 4]) {
        clipline(
            (a.into(), b.into()),
            ((0, 0), (N as i32 - 1, M as i32 - 1)),
            |x, y| {
                self.pixel(IVec2::from((x, y)), rgba);
            },
        );
    }

    fn triangle(&mut self, a: IVec2, b: IVec2, c: IVec2, rgba: [u8; 4]) {
        self.line(a, b, rgba);
        self.line(b, c, rgba);
        self.line(c, a, rgba);
    }

    fn clear_rectangle(&mut self, x: i32, y: i32, w: i32, h: i32) {
        debug_assert!(x >= 0 && y >= 0 && x + w <= N as i32 && y + h <= M as i32);
        for j in y..y + h {
            for i in x..x + w {
                self.pixel(IVec2::from((i, j)), [0, 0, 0, 0]);
            }
        }
    }
}
