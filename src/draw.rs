use clipline::clipline;

fn pixel(
    frame: &mut [u8],
    width: i32,
    x: i32, y: i32,
    rgba: [u8; 4]
) {
    let index = 4 * (x + y * width) as usize;
    // SAFETY: Clipline only calls draw_pixel with valid coordinates
    unsafe {frame.get_unchecked_mut(index..index + 4)}.copy_from_slice(&rgba);
}

pub fn clear(frame: &mut [u8]) {
    for pixel in frame.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[0, 0, 0, 0]);
    }
}

fn line(
    frame: &mut [u8],
    width: i32,
    height: i32,
    x0: i32, y0: i32,
    x1: i32, y1: i32,
    rgba: [u8; 4]
) {
    clipline(
        ((x0, y0), (x1, y1)),
        ((0, 0), (width - 1, height - 1)),
        |x, y| {
            pixel(frame, width, x, y, rgba);
        },
    );
}

pub fn triangle(
    frame: &mut [u8],
    width: i32,
    height: i32,
    x0: i32, y0: i32,
    x1: i32, y1: i32,
    x2: i32, y2: i32,
    rgba: [u8; 4]
) {
    line(frame, width, height, x0, y0, x1, y1, rgba);
    line(frame, width, height, x1, y1, x2, y2, rgba);
    line(frame, width, height, x2, y2, x0, y0, rgba);
}