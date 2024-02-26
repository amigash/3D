pub fn plot_line_with_depth(
    x0: i32, y0: i32, z0: f32,
    x1: i32, y1: i32, z1: f32,
    mut draw: impl FnMut(i32, i32, f32)) {

    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;

    let steps = std::cmp::max(dx, dy.abs());
    let dz = (z1 - z0) / steps as f32;

    let mut x = x0;
    let mut y = y0;
    let mut z = z0;
    loop {
        draw(x, y, z);
        z += dz;

        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * error;
        if e2 >= dy {
            if x == x1 {
                break;
            }
            error += dy;
            x += sx;
        }
        if e2 <= dx {
            if y == y1 {
                break;
            }
            error += dx;
            y += sy;
        }
    }
}
