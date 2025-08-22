use std::{fs, path::PathBuf};

use image::{ImageBuffer, ImageReader, Rgba};

pub(crate) fn mix_pics(input: Vec<PathBuf>, bg_img: &PathBuf) -> Vec<PathBuf> {
    let mut list = vec![];

    for pic in input {
        let new_file_path = get_mixed_name(&pic);

        let background_img = image::open(bg_img).expect(&format!(
            "load background image failed: {}",
            bg_img.display()
        ));
        let foreground_img = ImageReader::open(&pic)
            .expect(&format!(
                "ImageReader open foreground image failed: {}",
                pic.display()
            ))
            .with_guessed_format()
            .expect(&format!(
                "guess foreground image format failed: {}",
                pic.display()
            ))
            .decode()
            .expect(&format!(
                "expect foreground image failed: {}",
                pic.display()
            ));

        let background_img = background_img.to_rgba8();
        let foreground_img = foreground_img.to_rgba8();

        let (bg_width, bg_height) = background_img.dimensions();
        let (fg_width, fg_height) = foreground_img.dimensions();

        let scale = bg_width as f32 / fg_width as f32;
        let new_fg_height = (fg_height as f32 * scale) as u32;

        let resized_fg = image::imageops::resize(
            &foreground_img,
            bg_width,
            new_fg_height,
            image::imageops::FilterType::Lanczos3,
        );

        let mut output_img = ImageBuffer::new(bg_width, bg_height);

        for (x, y, pixel) in background_img.enumerate_pixels() {
            output_img.put_pixel(x, y, *pixel);
        }

        let y_pos = (bg_height - new_fg_height) / 2;

        for (x, y, pixel) in resized_fg.enumerate_pixels() {
            if y + y_pos < bg_height {
                let bg_pixel = output_img.get_pixel(x, y + y_pos);
                let blended = blend_pixels(*bg_pixel, *pixel);
                output_img.put_pixel(x, y + y_pos, blended);
            }
        }

        // 10. 保存结果
        output_img
            .save(&new_file_path)
            .expect(&format!("save image failed: {}", new_file_path.display()));

        list.push(new_file_path);
        _ = fs::remove_file(pic);
    }

    list
}

fn blend_pixels(background: Rgba<u8>, foreground: Rgba<u8>) -> Rgba<u8> {
    let alpha = foreground.0[3] as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    Rgba([
        (background.0[0] as f32 * inv_alpha + foreground.0[0] as f32 * alpha) as u8,
        (background.0[1] as f32 * inv_alpha + foreground.0[1] as f32 * alpha) as u8,
        (background.0[2] as f32 * inv_alpha + foreground.0[2] as f32 * alpha) as u8,
        (background.0[3] as f32 * inv_alpha + foreground.0[3] as f32 * alpha) as u8,
    ])
}

fn get_mixed_name(pic: &PathBuf) -> PathBuf {
    let mut new_file_path = pic.clone();
    new_file_path.set_file_name(format!(
        "{}-mixed.png",
        new_file_path.file_stem().unwrap().to_str().unwrap()
    ));

    new_file_path
}
