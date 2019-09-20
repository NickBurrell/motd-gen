extern crate image;
extern crate num_traits;

pub const ASCII_CHARS: [char; 11] = ['@', '#', 'S', '%', '?', '*', '+', ';', ':', ',', '.'];

use image::imageops::resize;
use image::{FilterType, GenericImageView, ImageBuffer, Pixel};
use num_traits::NumCast;

struct RgbColor(u32, u32, u32);
const ANSI_RESET: &str = "\x1b[0m";
impl RgbColor {
    pub fn to_ansi_escape(self) -> String {
        if self.0 == self.1 && self.1 == self.2 {
            if self.0 < 8 {
                return format!("\x1b[38;5;16m");
            }
            if self.0 > 248 {
                return format!("\x1b[38;5;231m");
            }
            return format!("\x1b[38;5;{}m", (((self.0 - 8) / 247) * 24) + 232);
        }
        println!(
            "{}, {}, {}\n{}",
            self.0,
            self.1,
            self.2,
            16. + (36. * (self.0 as f32 / 255. * 5.))
                + (6. * (self.1 as f32 / 255. * 5.))
                + (self.2 as f32 / 255. * 5.)
        );
        format!(
            "\x1b[38;5;{}m",
            (16. + (36. * (self.0 as f32 / 255. * 5.))
                + (6. * (self.1 as f32 / 255. * 5.))
                + (self.2 as f32 / 255. * 5.))
                .floor() as u32
        )
    }
    pub fn to_ansi_escape_true_color(self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.0, self.1, self.2)
    }
}

fn fix_aspect_ratio<'a, I: GenericImageView>(
    image: &'a I,
    new_width: u32,
    correct_font: bool,
) -> ImageBuffer<I::Pixel, Vec<<I::Pixel as Pixel>::Subpixel>>
where
    I::Pixel: 'static,
    <I::Pixel as Pixel>::Subpixel: 'static,
{
    let (current_width, current_height) = image.dimensions();
    let aspect_ratio = current_height as f32 / current_width as f32;
    let height_scalar = if correct_font { 3. / 5. } else { 1. };
    resize(
        image,
        new_width,
        (aspect_ratio * new_width as f32 * height_scalar) as u32,
        FilterType::Gaussian,
    )
}
fn to_ascii<'a, P: Pixel>(image: &'a ImageBuffer<P, Vec<P::Subpixel>>, buckets: u32) -> Vec<char>
where
    P: 'static,
{
    image
        .enumerate_pixels()
        .map(|(_, _, pixel)| {
            let index: u32 = NumCast::from(pixel.to_luma().channels()[0]).unwrap();
            ASCII_CHARS[(index / buckets) as usize]
        })
        .collect()
}

fn to_ascii_preserve_color<'a, P: Pixel>(
    image: &'a ImageBuffer<P, Vec<P::Subpixel>>,
    buckets: u32,
) -> Vec<String>
where
    P: 'static,
{
    image
        .enumerate_pixels()
        .map(|(_, _, pixel)| {
            let index: u32 = NumCast::from(pixel.to_luma().channels()[0]).unwrap();
            let (r_, g_, b_, _) = pixel.channels4(); // TODO: Fix all usage of channels4 prepending deprecation
            let r: u32 = NumCast::from(r_).unwrap();
            let g: u32 = NumCast::from(g_).unwrap();
            let b: u32 = NumCast::from(b_).unwrap();
            color_char(
                ASCII_CHARS[(index / buckets) as usize],
                RgbColor(r, g, b),
                true,
            )
        })
        .collect()
}

fn color_char(c: char, color: RgbColor, true_color: bool) -> String {
    format!(
        "{}{}{}",
        if true_color {
            color.to_ansi_escape_true_color()
        } else {
            color.to_ansi_escape()
        },
        c,
        ANSI_RESET
    )
}

pub fn asciify_image(filepath: &str, image_width: u32, correct_for_font: bool) -> Vec<String> {
    let img = image::open(filepath).unwrap();
    let scaled_image = fix_aspect_ratio(&img, image_width, correct_for_font);
    let raw_ascii = to_ascii(&scaled_image, 25);
    let mut line_idx = 0;
    let mut char_idx = 0;
    let mut current_string = String::new();
    let mut output: Vec<String> = Vec::new();
    for pixel_idx in 0..raw_ascii.len() {
        if pixel_idx as u32 % image_width == 0 {
            char_idx = 0;
            line_idx = line_idx + 1;
            output.push(current_string.clone());
            current_string.clear();
        }
        current_string.push(raw_ascii[pixel_idx]);
        char_idx = char_idx + 1;
    }
    output
}

pub fn asciify_image_color(
    filepath: &str,
    image_width: u32,
    correct_for_font: bool,
    truecolor: bool,
) -> Vec<String> {
    let img = image::open(filepath).unwrap();
    let scaled_image = fix_aspect_ratio(&img, image_width, correct_for_font);
    let raw_ascii = to_ascii_preserve_color(&scaled_image, 25);
    let mut line_idx = 0;
    let mut char_idx = 0;
    let mut current_string = String::new();
    let mut output: Vec<String> = Vec::new();
    for pixel_idx in 0..raw_ascii.len() {
        if pixel_idx as u32 % image_width == 0 {
            char_idx = 0;
            line_idx = line_idx + 1;
            output.push(current_string.clone());
            current_string.clear();
        }
        current_string.push_str(&raw_ascii[pixel_idx as usize]);
        char_idx = char_idx + 1;
    }
    output
}

#[cfg(test)]
mod tests {
    #[test]
    fn asciify_test() {
        let ascii = super::asciify_image_color("/home/nick/Downloads/test.png", 250, true, false);
        for line in ascii {
            println!("{}", line);
        }
    }
}
