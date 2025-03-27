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

fn apply_palette(input_image: &DynamicImage) -> RgbImage {
    let (width, height) = input_image.dimensions();

    ImageBuffer::from_fn(width, height, |x, y| {
        let pixel: image::Rgba<u8> = input_image.get_pixel(x, y);
        let input_color: Color = Color { r: pixel[0], g: pixel[1], b: pixel[2] };
        let new_color: Color = get_nearest_color(input_color);
        Rgb([new_color.r, new_color.g, new_color.b])
    })
}

fn quantize(value: u8) -> u8 {
    if value < 128 { 0 } else { 255 }
}

fn grayscale(image: &RgbImage) -> GrayImage {
    let (width, height) = image.dimensions();
    let mut gray_image: ImageBuffer<Luma<u8>, Vec<u8>> = GrayImage::new(width, height);

    for (x, y, pixel) in image.enumerate_pixels() {
        let Rgb([r, g, b]) = *pixel;
        let gray_value: u8 = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) as u8;
        gray_image.put_pixel(x, y, Luma([gray_value]));
    }
    gray_image
}

fn floyd_steinberg_dithering(image: &GrayImage) -> GrayImage {
    let (width, height) = image.dimensions();
    let mut img: ImageBuffer<Luma<u8>, Vec<u8>> = image.clone();
    for y in 0..height {
        for x in 0..width {
            let old_pixel: u8 = img.get_pixel(x, y)[0];
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

fn apply_floyd_steinberg_dithering(image: &DynamicImage) -> GrayImage {
    let rgb_img: ImageBuffer<Rgb<u8>, Vec<u8>> = image.clone().into_rgb8();
    let grayscaled_img: ImageBuffer<Luma<u8>, Vec<u8>> = grayscale(&rgb_img);
    floyd_steinberg_dithering(&grayscaled_img)
}

fn pixelate(image: &DynamicImage, pixel_size: u32) -> RgbImage {
    let rgb_img: ImageBuffer<Rgb<u8>, Vec<u8>> = image.clone().into_rgb8();
    let (width, height) = rgb_img.dimensions();

    let small_width: u32 = width / pixel_size;
    let small_height: u32 = height / pixel_size;
    let small_img: ImageBuffer<Rgb<u8>, Vec<u8>> = imageops::resize(&rgb_img, small_width, small_height, imageops::FilterType::Nearest);
    imageops::resize(&small_img, width, height, imageops::FilterType::Nearest)
}

#[derive(Debug, Clone, Copy)]
enum FilterOperation {
    Palette,
    Pixelate(u32),
    FloydSteinberg,
}

pub fn apply() {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 3 {
        println!("Usage: cargo r [filter operations] input_path output_path");
        println!("Filter operations:");
        println!("  -pal: Apply Game Boy palette");
        println!("  -pixpal: Apply pixelation (with optional size, default 8) and palette");
        println!("  -pix=N: Apply pixelation with size N (default 8)");
        println!("  -floyd: Apply Floyd-Steinberg dithering");
        println!("Example: cargo r -pal -pix=4 -floyd input.png output.png");
        return;
    }
    
    let input_path: &String = &args[args.len() - 2];
    let output_path: &String = &args[args.len() - 1];
    
    let mut operations: Vec<FilterOperation> = Vec::new();
    for i in 1..(args.len() - 2) {
        let arg: &String = &args[i];
        
        if arg == "-pal" {
            operations.push(FilterOperation::Palette);
        } else if arg == "-pixpal" {
            operations.push(FilterOperation::Pixelate(8));
            operations.push(FilterOperation::Palette);
        } else if arg == "-floyd" {
            operations.push(FilterOperation::FloydSteinberg);
        } else if arg.starts_with("-pix=") {
            if let Some(size_str) = arg.strip_prefix("-pix=") {
                if let Ok(size) = size_str.parse::<u32>(){
                    if size != 0 {
                        operations.push(FilterOperation::Pixelate(size));
                    }
                } else {
                    println!("Invalid pixel size: {}", size_str);
                    return;
                }
            }
        } else if arg == "-pix" {
            operations.push(FilterOperation::Pixelate(8));
        } else {
            println!("Unknown operation: {}", arg);
            return;
        }
    }
    
    if operations.is_empty() {
        println!("No filter operations specified!");
        return;
    }
    
    let mut image: DynamicImage = match image::open(input_path) {
        Ok(img) => img,
        Err(e) => {
            println!("Failed to load image {}: {}", input_path, e);
            return;
        }
    };
    
    let mut gray_image_option: Option<GrayImage> = None;
    
    for op in operations {
        println!("Applying {:?}...", op);
        
        match op {
            FilterOperation::Palette => {
                if gray_image_option.is_some() {
                    let gray: ImageBuffer<Luma<u8>, Vec<u8>> = gray_image_option.take().unwrap();
                    image = DynamicImage::ImageLuma8(gray).into();
                }
                
                let rgb_image: ImageBuffer<Rgb<u8>, Vec<u8>> = apply_palette(&image);
                image = DynamicImage::ImageRgb8(rgb_image);
                gray_image_option = None;
            },
            FilterOperation::Pixelate(size) => {
                if gray_image_option.is_some() {
                    let gray: ImageBuffer<Luma<u8>, Vec<u8>> = gray_image_option.take().unwrap();
                    image = DynamicImage::ImageLuma8(gray).into();
                }
                
                let rgb_image: ImageBuffer<Rgb<u8>, Vec<u8>> = pixelate(&image, size);
                image = DynamicImage::ImageRgb8(rgb_image);
                gray_image_option = None;
            },
            FilterOperation::FloydSteinberg => {
                let gray_image: ImageBuffer<Luma<u8>, Vec<u8>> = apply_floyd_steinberg_dithering(&image);
                gray_image_option = Some(gray_image);
            }
        }
    }
    
    if let Some(gray_image) = gray_image_option {
        save(output_path, gray_image);
    } else {
        match image.save(output_path) {
            Ok(_) => println!("The image is saved: {}", output_path),
            Err(e) => println!("Failed to save image {}: {}", output_path, e),
        }
    }
}