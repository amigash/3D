#![deny(clippy::all)]
#![forbid(unsafe_code)]

use clipline::clipline;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::EventLoop,
    keyboard::KeyCode,
    window::WindowBuilder,
    event::Event
};
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 80;
const HEIGHT: u32 = 60;
const SCREEN: ((i16, i16), (i16, i16)) = ((0, 0), (WIDTH as i16, HEIGHT as i16));


/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
}

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    event_loop.run(move |event, elwt| {
        // Draw the current frame
        if let Event::WindowEvent {event: WindowEvent::RedrawRequested, ..} = event {
            let frame = pixels.frame_mut();
            world.clear(frame);
            let tri = ((10, 10), (25, 20), (30, 10));
            world.draw_triangle(frame, tri.0, tri.1, tri.2, [255, 255, 0, 255]);
            world.draw_filled_flat_triangle(frame, tri.1, tri.0.0, tri.2.0, tri.0.1, [255, 0, 0, 255]);
            if pixels.render().is_err() {
                elwt.exit();
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() || input.destroyed() {
                elwt.exit();
            }


            // Resize the window
            if let Some(size) = input.window_resized() {
                if pixels.resize_surface(size.width, size.height).is_err() {
                    elwt.exit();
                }
            }

            // Update internal state and request a redraw
            world.update();
            window.request_redraw();
        }
    }).unwrap();
    Ok(())
}

impl World {
    fn new() -> Self {
        Self {
        }
    }

    fn update(&mut self) {
    }

    fn draw_pixel(&self, frame: &mut [u8], x: i16, y: i16, rgba: [u8; 4]) {
        let index = 4 * (x + y * WIDTH as i16) as usize;
        frame[index..index + 4].copy_from_slice(&rgba);
    }

    fn clear(&self, frame: &mut [u8]) {
        for pixel in frame.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0, 0, 0, 0]);
        }
    }


    fn draw_line(&self, frame: &mut [u8], start: (i16, i16), end: (i16, i16), rgba: [u8; 4]) {
        clipline((start, end), SCREEN, |x, y| {
            self.draw_pixel(frame, x, y, rgba);
        });
    }

    fn draw_triangle(&self, frame: &mut [u8], a: (i16, i16), b: (i16, i16), c: (i16, i16), rgba: [u8; 4]) {
        self.draw_line(frame, a, b, rgba);
        self.draw_line(frame, b, c, rgba);
        self.draw_line(frame, c, a, rgba);
    }

    fn draw_filled_flat_triangle(&self, frame: &mut [u8], a: (i16, i16), b_x: i16, c_x: i16, bc_y: i16, rgba: [u8; 4]) {
        let edge_points = |p_x: i16| {
            let mut side = vec![];
            let mut prev = a;

            clipline((a, (p_x, bc_y)), SCREEN, |x, y| {
                if y != prev.1 {
                    side.push(prev);
                }
                prev = (x, y);
            });
            side.push(prev);
            side
        };
        let side_a = edge_points(b_x);
        let side_b = edge_points(c_x);

        for (point_a, point_b) in side_a.into_iter().zip(side_b) {
            self.draw_line(frame, point_a,  point_b, rgba);
        }
    }

    // fn draw_filled_triangle(&self, frame: &mut [u8], a: (i16, i16), b: (i16, i16), c: (i16, i16), rgba: [u8; 4]) {
    //
    //
    //
    // }
}