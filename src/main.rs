use filter::filter::*;
use image::{ DynamicImage, GrayImage, ImageBuffer, Luma, Rgb };

fn apply() {
    let args: Vec<String> = std::env::args().collect();
     
    if args.len() < 3 {
        println!("Usage: cargo r [filter operations] input_path output_path");
        println!("Filter operations:");
        println!("  -pal: Apply palette");
        println!("  -pixpal: Apply pixelation and palette");
        println!("  -pix=N: Apply pixelation with size N (default 8)");
        println!("  -floyd: Apply Floyd-Steinberg dithering");
        println!("  -rev: Reverse colors");
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
         } else if arg == "-rev" {
            operations.push(FilterOperation::Reverse);
         }else {
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
            },
            FilterOperation::Reverse => {
               if gray_image_option.is_some() {
                   let gray: ImageBuffer<Luma<u8>, Vec<u8>> = gray_image_option.take().unwrap();
                   image = DynamicImage::ImageLuma8(gray).into();
                  }
               let rgb_image: ImageBuffer<Rgb<u8>, Vec<u8>> = reverse(&image);
               image = DynamicImage::ImageRgb8(rgb_image);
               gray_image_option = None;
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

fn main() {
     apply();       
}
