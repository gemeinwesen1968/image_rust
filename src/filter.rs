use image::{imageops, DynamicImage, Pixel, GenericImageView, GrayImage, ImageBuffer, Luma, Rgb, RgbImage };
use std::f32;
use crate::palette::*;


#[derive(Debug, Clone, Copy)]
pub enum FilterOperation {
    Palette,
    Pixelate(u32),
    FloydSteinberg,
    Reverse,
}

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn from_rgb_components(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }
    
    pub fn from_rgb(rgb: &Rgb<u8>) -> Self {
        Self::from_rgb_components(rgb[0], rgb[1], rgb[2])
    }

}


// fn color_distance(c1: Color, c2: Color) -> f32 {
//     let r: f32 = (c1.r as f32 - c2.r as f32).powi(2);
//     let g: f32 = (c1.g as f32 - c2.g as f32).powi(2);
//     let b: f32 = (c1.b as f32 - c2.b as f32).powi(2);
//     (r + g + b).sqrt()
// }

pub fn save<P, Container>(output_path: &str, img: ImageBuffer<P, Container>) -> () 
where 
    P: Pixel<Subpixel = u8> + 'static + image::PixelWithColorType,
    Container: std::ops::Deref<Target = [u8]>,
{
    img.save(output_path).expect("Failed to save image!");
    println!("The image is saved: {}", output_path);
}


pub fn apply_palette(input_image: &DynamicImage, palette_path: &str) -> RgbImage {
    let (width, height) = input_image.dimensions();

    let palette = match Palette::from_file(palette_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error loading palette from {}: {}", palette_path, e);
            return fallback_palette(input_image);
        }
    };

    println!("Palette: {}\n{}\n{:?}", palette.name, palette.description, palette.colors);
    
    let palette_colors: Vec<Rgb<u8>> = palette.get_colors();

    if palette_colors.is_empty() {
        eprintln!("Warning: Palette has no colors, using fallback");
        return fallback_palette(input_image);
    }

    let colors: Vec<Color> = palette_colors.iter()
        .map(|rgb| Color::from_rgb(rgb))
        .collect();

    set_active_palette(&colors);

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

pub fn grayscale(image: &RgbImage) -> GrayImage {
    let (width, height) = image.dimensions();
    let mut gray_image: ImageBuffer<Luma<u8>, Vec<u8>> = GrayImage::new(width, height);

    for (x, y, pixel) in image.enumerate_pixels() {
        let Rgb([r, g, b]) = *pixel;
        let gray_value: u8 = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) as u8;
        gray_image.put_pixel(x, y, Luma([gray_value]));
    }
    gray_image
}


pub fn reverse(image: &DynamicImage) -> RgbImage {
    let (width, height) = image.dimensions();

    ImageBuffer::from_fn(width, height, |x, y| {
        let pixel: image::Rgba<u8> = image.get_pixel(x, y);
        let new_color: Color = Color { r: 255 - pixel[0], g: 255 - pixel[1], b: 255 - pixel[2]};
        Rgb([new_color.r, new_color.g, new_color.b])
    })
}

pub fn floyd_steinberg_dithering(image: &GrayImage) -> GrayImage {
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

pub fn apply_floyd_steinberg_dithering(image: &DynamicImage) -> GrayImage {
    let rgb_img: ImageBuffer<Rgb<u8>, Vec<u8>> = image.clone().into_rgb8();
    let grayscaled_img: ImageBuffer<Luma<u8>, Vec<u8>> = grayscale(&rgb_img);
    floyd_steinberg_dithering(&grayscaled_img)
}

pub fn pixelate(image: &DynamicImage, pixel_size: u32) -> RgbImage {
    let rgb_img: ImageBuffer<Rgb<u8>, Vec<u8>> = image.clone().into_rgb8();
    let (width, height) = rgb_img.dimensions();

    let small_width: u32 = width / pixel_size;
    let small_height: u32 = height / pixel_size;
    let small_img: ImageBuffer<Rgb<u8>, Vec<u8>> = imageops::resize(&rgb_img, small_width, small_height, imageops::FilterType::Nearest);
    imageops::resize(&small_img, width, height, imageops::FilterType::Nearest)
}