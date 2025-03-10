use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb, imageops};
use std::f32;

#[derive(Copy, Clone, Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

const GB_PALETTE: [Color; 7] = [
    Color { r: 32, g: 32, b: 32 },
    Color { r: 128, g: 255, b: 0 },
    Color { r: 255, g: 255, b: 102 },
    Color { r: 51, g: 255, b: 255 },
    Color { r: 127, g: 0, b: 255},
    Color { r: 255, g: 51, b: 153 },
    Color { r: 255, g: 128, b: 0 },
];

fn color_distance(c1: Color, c2: Color) -> f32 {
    let r: f32 = (c1.r as f32 - c2.r as f32).powi(2);
    let g: f32 = (c1.g as f32 - c2.g as f32).powi(2);
    let b: f32 = (c1.b as f32 - c2.b as f32).powi(2);
    (r + g + b).sqrt()
}

fn get_nearest_color(pixel: Color) -> Color {
    GB_PALETTE.iter()
        .copied()
        .min_by(|&a, &b| color_distance(pixel, a)
            .partial_cmp(&color_distance(pixel, b))
            .unwrap())
        .unwrap()
}

fn save(output_path: &str, img: ImageBuffer<Rgb<u8>, Vec<u8>>) -> () {
    img.save(output_path).expect("Failed to save image!");
    println!("The image is saved: {}", output_path);
}

fn apply_palette(input_path: &str) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let img: DynamicImage = image::open(input_path).expect("Failed to load image!");
    let (width, height): (u32, u32) = img.dimensions();

    ImageBuffer::from_fn(width, height, |x, y| {
        let pixel: image::Rgba<u8> = img.get_pixel(x, y);
        let input_color: Color = Color { r: pixel[0], g: pixel[1], b: pixel[2] };
        let new_color: Color = get_nearest_color(input_color);
        Rgb([new_color.r, new_color.g, new_color.b])
    })
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


fn pixelate(input_path: &str, pixel_size: u32) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = image::open(input_path).expect("Failed to load image!").into_rgb8();
    let (width, height) = img.dimensions();

    let small_width: u32 = width / pixel_size;
    let small_height: u32 = height / pixel_size;
    let small_img: ImageBuffer<Rgb<u8>, Vec<u8>> = imageops::resize(&img, small_width, small_height, imageops::FilterType::Nearest);
    imageops::resize(&small_img, width, height, imageops::FilterType::Nearest)
}

fn apply_palette_in(img:  & mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
    let mut pixels_to: Vec<(u32, u32, Rgb<u8>)> = Vec::new();
    for (x, y, pixel) in img.enumerate_pixels() {
        let input_color: Color = Color { r: pixel[0], g: pixel[1], b: pixel[2] };
        let new_color: Color = get_nearest_color(input_color);
        pixels_to.push((x, y, Rgb([new_color.r, new_color.g, new_color.b])));
    }

    for (x, y, new_pixel) in pixels_to {
        img.put_pixel(x, y, new_pixel);
    }
} 

fn pixelate_and_apply_palette(input_path: &str, output_path: &str, pixel_size: u32) {
    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = pixelate(input_path, pixel_size);
    apply_palette_in(&mut img);
    save(output_path, img);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        println!("Usage: {} <palette> <input.png> <output.png>", args[0]);
        return;
    }

    if args[1] == "pal" {
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = apply_palette(&args[2]);
        save(&args[3], img);
    } else if args[1] == "pixpal" {
        pixelate_and_apply_palette(&args[2], &args[3], 4);
    } else {
        println!("Palette {} not available!", args[1]);
    }
}
