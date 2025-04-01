use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb, RgbImage};
use crate::filter::*;
use std::sync::RwLock;
use once_cell::sync::Lazy;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Palette {
    pub name: String,
    pub description: String,
    pub colors: Vec<[u8; 3]>,
}

impl Palette {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let palette: Palette = serde_json::from_reader(reader)?;
        Ok(palette)
    }

    pub fn get_colors(&self) -> Vec<Rgb<u8>> {
        self.colors.iter()
            .map(|&[r, g, b]| Rgb([r, g, b]))
            .collect()
    }
}

static ACTIVE_PALETTE: Lazy<RwLock<Vec<Color>>> = Lazy::new(|| {
    RwLock::new(vec![
        Color { r: 0, g: 0, b: 0 },       // Black
        Color { r: 255, g: 255, b: 255 }, // White
        Color { r: 255, g: 0, b: 0 },     // Red
        Color { r: 0, g: 255, b: 0 },     // Green
        Color { r: 0, g: 0, b: 255 },     // Blue
        Color { r: 255, g: 255, b: 0 },   // Yellow
        Color { r: 255, g: 0, b: 255 },   // Magenta
        Color { r: 0, g: 255, b: 255 },   // Cyan
    ])
});

pub fn set_active_palette(colors: &[Color]) {
    if let Ok(mut palette) = ACTIVE_PALETTE.write() {
        palette.clear();
        palette.extend_from_slice(colors);
    } else {
        eprintln!("Warning: Failed to acquire write lock for palette.");
    }
}

pub fn get_nearest_color(color: Color) -> Color {
    if let Ok(palette) = ACTIVE_PALETTE.read() {
        if palette.is_empty() {
            return color;
        }

        palette.iter()
            .min_by_key(|&&palette_color| {
                let dr = palette_color.r as i32 - color.r as i32;
                let dg = palette_color.g as i32 - color.g as i32;
                let db = palette_color.b as i32 - color.b as i32;
                dr * dr + dg * dg + db * db
            })
            .copied()
            .unwrap_or(color)
    } else {
        eprintln!("Warning: Failed to acquire read lock for palette.");
        color
    }
}

pub fn fallback_palette(input_image: &DynamicImage) -> RgbImage {
    if let Ok(palette) = ACTIVE_PALETTE.read() {
        if palette.len() > 1 {
        } else {
            drop(palette);
                let default_colors = vec![
                Color { r: 0, g: 0, b: 0 },       // Black
                Color { r: 255, g: 255, b: 255 }, // White
                Color { r: 255, g: 0, b: 0 },     // Red
                Color { r: 0, g: 255, b: 0 },     // Green
                Color { r: 0, g: 0, b: 255 },     // Blue
                Color { r: 255, g: 255, b: 0 },   // Yellow
                Color { r: 255, g: 0, b: 255 },   // Magenta
                Color { r: 0, g: 255, b: 255 },   // Cyan
            ];
            set_active_palette(&default_colors);
        }
    }
    
    let (width, height) = input_image.dimensions();
    
    ImageBuffer::from_fn(width, height, |x, y| {
        let pixel = input_image.get_pixel(x, y);
        let input_color = Color { r: pixel[0], g: pixel[1], b: pixel[2] };
        let new_color = get_nearest_color(input_color);
        Rgb([new_color.r, new_color.g, new_color.b])
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::{create_dir_all, remove_file}, io::Write};
    #[test]
    fn write_palette_and_read() {
        let test_dir: &str = "./test_files";
        create_dir_all(test_dir).expect("Failed to create test directory");
        let test_file_path: String = format!("{}/palette.json", test_dir);

        let mock_json: &str = r#"
        {
            "name": "Warm Colors",
            "description": "A palette of warm colors",
            "colors": [
                [255, 0, 0],
                [255, 165, 0],
                [255, 255, 0]
            ]
        }
        "#;

        let mut file: File = File::create(&test_file_path).expect("Failed to create test file");
        file.write_all(mock_json.as_bytes()).expect("Failed to write to test file");

        let result = Palette::from_file(&test_file_path);

        assert!(result.is_ok());
        let palette = result.unwrap();
        assert_eq!(
            palette,
            Palette {
                name: "Warm Colors".to_string(),
                description: "A palette of warm colors".to_string(),
                colors: vec![
                    [255, 0, 0],
                    [255, 165, 0],
                    [255, 255, 0]
                ]
            }
        );

        remove_file(&test_file_path).expect("Failed to delete test file");
    }
}
