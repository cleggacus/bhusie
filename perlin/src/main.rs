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


fn generate(width: u32, height: u32, density: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let mut buffer = ImageBuffer::new(width, height);

    let density = density as f32 / width as f32;

    for x in 0..width {
        for y in 0..height {
            let rx = x as f32 * density ;
            let ry = y as f32 * density;
            let val = (perlin(rx, ry) * 256.0) as u8;

            buffer.put_pixel(x, y, Rgba([val, val, val, val]));
        }
    }

    buffer
}

fn spiral(buffer: ImageBuffer<Rgba<u8>, Vec<u8>>, amount: f32, power: f32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let image_width = buffer.width();
    let image_height = buffer.height();

    let mut new_buffer = ImageBuffer::new(image_width, image_height);

    for x in 0..image_width {
        for y in 0..image_height {
            let rx = (x as f32 / image_width as f32) * 2.0 - 1.0;
            let ry = (y as f32 / image_height as f32) * 2.0 - 1.0;

            let r = (rx.powi(2) + ry.powi(2)).sqrt();
            let theta = ry.atan2(rx);

            let theta = ((theta + PI + r.powf(power) * PI * amount) % (2.0 * PI)) - PI;

            let rx = r * theta.cos();
            let ry = r * theta.sin();

            let nx = ((rx * 0.5 + 0.5) * image_width as f32) as u32 % image_width;
            let ny = ((ry * 0.5 + 0.5) * image_height as f32) as u32 % image_height;

            let pixel = buffer.get_pixel(nx, ny);

            new_buffer.put_pixel(x, y, *pixel);
        }
    }

    new_buffer
}


fn merge(buffer1: ImageBuffer<Rgba<u8>, Vec<u8>>, buffer2: ImageBuffer<Rgba<u8>, Vec<u8>>, amount: f32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    let image_width = buffer1.width();
    let image_height = buffer1.height();

    let mut new_buffer = ImageBuffer::new(image_width, image_height);

    for x in 0..image_width {
        for y in 0..image_height {
            let p1 = buffer1.get_pixel(x, y);
            let p2 = buffer2.get_pixel(x, y);

            new_buffer.put_pixel(x, y, Rgba([
                (p1[0] as f32 * amount + p2[0] as f32 * (1.0 - amount)) as u8,
                (p1[1] as f32 * amount + p2[1] as f32 * (1.0 - amount)) as u8,
                (p1[2] as f32 * amount + p2[2] as f32 * (1.0 - amount)) as u8,
                (p1[3] as f32 * amount + p2[3] as f32 * (1.0 - amount)) as u8,
            ]));
        }
    }

    new_buffer
}

fn main() {
    let buffer_0 = generate(1000, 1000, 4);
    let spiral_0 = spiral(buffer_0, 2.0, 0.5);
    let buffer_1 = generate(1000, 1000, 20);
    let spiral_1 = spiral(buffer_1, 2.0, 0.5);
    let buffer_2 = generate(1000, 1000, 50);
    let spiral_2 = spiral(buffer_2, 2.0, 0.5);
    let buffer_3 = generate(1000, 1000, 100);
    let spiral_3 = spiral(buffer_3, 2.0, 0.5);

    let m1 = merge(spiral_3, spiral_2, 0.5);
    let m2 = merge(m1, spiral_1, 0.5);
    let m3 = merge(m2, spiral_0, 0.5);

    m3.save("./src/renderer/textures/disk.png").unwrap();
}
