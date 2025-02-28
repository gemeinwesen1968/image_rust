use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb};
use std::f32;

#[derive(Copy, Clone, Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

const GB_PALETTE: [Color; 4] = [
    Color { r: 8, g: 24, b: 32 },
    Color { r: 52, g: 104, b: 86 },
    Color { r: 136, g: 192, b: 112 },
    Color { r: 224, g: 248, b: 208 },
];

fn color_distance(c1: Color, c2: Color) -> f32 {
    let r_mean: f32 = (c1.r as f32 + c2.r as f32) / 2.0;
    let r: f32 = (c1.r as f32 - c2.r as f32).powi(2);
    let g: f32 = (c1.g as f32 - c2.g as f32).powi(2);
    let b: f32 = (c1.b as f32 - c2.b as f32).powi(2);
    ((2.0 + r_mean / 256.0) * r + 4.0 * g + (2.0 + (255.0 - r_mean) / 256.0) * b).sqrt()
}

fn get_nearest_gb_color(pixel: Color) -> Color {
    GB_PALETTE.iter()
        .copied()
        .min_by(|&a, &b| color_distance(pixel, a)
            .partial_cmp(&color_distance(pixel, b))
            .unwrap())
        .unwrap()
}

fn apply_gameboy_palette(input_path: &str, output_path: &str) {
    let img: DynamicImage = image::open(input_path).expect("Failed to load image!");
    let (width, height): (u32, u32) = img.dimensions();

    let new_img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |x, y| {
        let pixel: image::Rgba<u8> = img.get_pixel(x, y);
        let input_color: Color = Color { r: pixel[0], g: pixel[1], b: pixel[2] };
        let new_color: Color = get_nearest_gb_color(input_color);
        Rgb([new_color.r, new_color.g, new_color.b])
    });
    
    new_img.save(output_path).expect("Failed to save image!");
    println!("The gameboy palette applied! Saved to {}", output_path);
}

// fn apply_gameboy_palette(input_path: &str, output_path: &str) {
//     let img: DynamicImage = image::open(input_path).expect("Failed to load image!");
//     let (width, height): (u32, u32) = img.dimensions();
//     let mut new_img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::new(width, height);
// 
//     for (x, y, pixel) in img.pixels() {
//         let input_color = Color { r: pixel[0], g: pixel[1], b: pixel[2] };
//         let new_color = get_nearest_gb_color(input_color);
//         new_img.put_pixel(x, y, Rgb([new_color.r, new_color.g, new_color.b]));
//     }
// 
//     new_img.save(output_path).expect("Failed to save image");
//     println!("Game Boy palette applied! Saved to {}", output_path);
// }

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        println!("Usage: {} <palette> <input.png> <output.png>", args[0]);
        return;
    }

    if args[1] == "gameboy" {
        apply_gameboy_palette(&args[2], &args[3]);
    } else {
        println!("Palette {} not available!", args[1]);
    }
}
