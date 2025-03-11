use image::{imageops, DynamicImage, Pixel, GenericImageView, GrayImage, ImageBuffer, Luma, Rgb, RgbImage };
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

fn get_nearest_color(color: Color) -> Color {
    GB_PALETTE.iter()
        .copied()
        .min_by(|&a, &b| color_distance(color, a)
            .partial_cmp(&color_distance(color, b))
            .unwrap())
        .unwrap()
}

fn save<P, Container>(output_path: &str, img: ImageBuffer<P, Container>) -> () 
where 
    P: Pixel<Subpixel = u8> + 'static + image::PixelWithColorType,
    Container: std::ops::Deref<Target = [u8]>,
{
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

fn quantize(value: u8) -> u8 {
    if value < 128 { 0 } else { 255 }
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

fn grayscale(image: &RgbImage) -> GrayImage {
    let (width, height) = image.dimensions();
    let mut gray_image = GrayImage::new(width, height);

    for (x, y, pixel) in image.enumerate_pixels() {
        let Rgb([r, g, b]) = *pixel;
        let gray_value = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) as u8;
        gray_image.put_pixel(x, y, Luma([gray_value]));
    }
    gray_image
}

fn floyd_steinberg_dithering(image: &GrayImage) -> GrayImage {
    let (width, height) = image.dimensions();
    let mut img:ImageBuffer<Luma<u8>, Vec<u8>>  = image.clone();
    for y in 0..height {
        for x in 0..width {
            let old_pixel:u8  = img.get_pixel(x, y)[0];
            let new_pixel: u8 = quantize(old_pixel);
            let error: i16 = old_pixel as i16 - new_pixel as i16;

            img.put_pixel(x, y, Luma([new_pixel]));

            if x + 1 < width {
                let right_pixel: i16 = img.get_pixel(x + 1, y)[0] as i16;
                img.put_pixel(x + 1, y, Luma([(right_pixel + (error * 7 / 16) as i16).clamp(0, 255) as u8]));
            }

            if y + 1 < height {
                if x > 0 {
                    let bottom_left_pixel: i16 = img.get_pixel(x - 1, y + 1)[0] as i16;
                    img.put_pixel(x - 1, y + 1, Luma([(bottom_left_pixel + (error * 3 / 16) as i16).clamp(0, 255) as u8]));
                }

                let bottom_pixel: i16 = img.get_pixel(x, y + 1)[0] as i16;
                img.put_pixel(x, y + 1, Luma([(bottom_pixel + (error * 5 / 16) as i16).clamp(0, 255) as u8]));

                if x + 1 < width {
                    let bottom_right_pixel = img.get_pixel(x + 1, y + 1)[0] as i16;
                    img.put_pixel(x + 1, y + 1, Luma([(bottom_right_pixel + (error * 1 / 16) as i16).clamp(0, 255) as u8]));
                }
            }
        }
    }
    img
} 

fn apply_floyd_steinberg_dithering(input_path: &str, output_path: &str) {
    let img: RgbImage = image::open(input_path).expect("Failed to load image!").into_rgb8();
    let grayscaled_img: GrayImage = grayscale(&img);
    let dithered_img : GrayImage  = floyd_steinberg_dithering(&grayscaled_img);
    save(output_path, dithered_img);
}

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
        pixelate_and_apply_palette(&args[2], &args[3], 8);
    } else if args[1] == "pix" {
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = pixelate(&args[2], 8);
        save(&args[3], img);
    } else if args[1] == "floyd" {
        apply_floyd_steinberg_dithering(&args[2], &args[3]);
    } else {
        println!("Palette {} not available!", args[1]);
    }
}
