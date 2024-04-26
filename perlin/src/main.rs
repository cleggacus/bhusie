use std::f32::consts::PI;

use cgmath::Vector2;
use image::{ImageBuffer, Rgba};

fn random_gradient(ix: u32, iy: u32) -> Vector2<f32> {
    let w = 8 * std::mem::size_of::<u32>() as u32;
    let s = w / 2; // rotation width
    let mut a = ix;
    let mut b = iy;
    
    a = a.wrapping_mul(3284157443);
    b ^= a.rotate_left(s);
    
    b = b.wrapping_mul(1911520717);
    a ^= b.rotate_left(s);
    
    a = a.wrapping_mul(2048419325);
    
    let random = a as f32 * (std::f32::consts::PI / !(0u32 >> 1) as f32); // in [0, 2*Pi]

    Vector2::new(random.cos(), random.sin())
}

fn dot_grid_gradient(ix: u32, iy: u32, x: f32, y: f32) -> f32 {
    let gradient = random_gradient(ix, iy);

    let dx = x - ix as f32;
    let dy = y - iy as f32;

    dx*gradient.x + dy*gradient.y
}

fn interpolate(a0: f32, a1: f32, w: f32) -> f32 {
    // (a1 - a0) * w + a0
    (a1 - a0) * ((w * (w * 6.0 - 15.0) + 10.0) * w * w * w) + a0
}

fn perlin(x: f32, y: f32) -> f32 {
    let x0 = x.floor() as u32;
    let x1 = x0 + 1;
    let y0 = y.floor() as u32;
    let y1 = y0 + 1;

    let sx = x - x0 as f32;
    let sy = y - y0 as f32;

    let n0 = dot_grid_gradient(x0, y0, x, y);
    let n1 = dot_grid_gradient(x1, y0, x, y);
    let ix0 = interpolate(n0, n1, sx);

    let n0 = dot_grid_gradient(x0, y1, x, y);
    let n1 = dot_grid_gradient(x1, y1, x, y);
    let ix1 = interpolate(n0, n1, sx);

    interpolate(ix0, ix1, sy) * 0.5 + 0.5
}


fn main() {
    const IMAGE_WIDTH: u32 = 1000;
    const IMAGE_HEIGHT: u32 = 1000;
    const DENSITY: u32 = 30;

    let mut buffer = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);

    let density = DENSITY as f32 / IMAGE_WIDTH as f32;

    for x in 0..IMAGE_WIDTH {
        for y in 0..IMAGE_HEIGHT {
            let rx = x as f32 * density ;
            let ry = y as f32 * density;
            let val = (perlin(rx, ry) * 256.0) as u8;

            buffer.put_pixel(x, y, Rgba([255, 255, 255, val]));
        }
    }

    let mut new_buffer = ImageBuffer::new(IMAGE_WIDTH, IMAGE_HEIGHT);

    for x in 0..IMAGE_WIDTH {
        for y in 0..IMAGE_HEIGHT {
            let rx = (x as f32 / IMAGE_WIDTH as f32) * 2.0 - 1.0;
            let ry = (y as f32 / IMAGE_HEIGHT as f32) * 2.0 - 1.0;

            let r = (rx.powi(2) + ry.powi(2)).sqrt();
            let theta = ry.atan2(rx);

            let theta = ((theta + PI + r.powf(0.7) * PI * 2.0) % (2.0 * PI)) - PI;

            let rx = r * theta.cos();
            let ry = r * theta.sin();

            let nx = ((rx * 0.5 + 0.5) * IMAGE_WIDTH as f32) as u32 % IMAGE_WIDTH ;
            let ny = ((ry * 0.5 + 0.5) * IMAGE_HEIGHT as f32) as u32 % IMAGE_HEIGHT;

            let pixel = buffer.get_pixel(nx, ny);

            new_buffer.put_pixel(x, y, *pixel);
        }
    }

    buffer.save("./src/renderer/textures/perlin.png").unwrap();
    new_buffer.save("./src/renderer/textures/disk.png").unwrap();
}
