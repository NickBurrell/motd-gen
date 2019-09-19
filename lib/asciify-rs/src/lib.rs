#[macro_use]
extern crate lazy_static;
extern crate colored;
extern crate image;
extern crate num_traits;

pub const ASCII_CHARS: [char; 11] = ['@', '#', 'S', '%', '?', '*', '+', ';', ':', ',', '.'];

pub(crate) mod util {

    use image::imageops::resize;
    use image::{FilterType, GenericImageView, ImageBuffer, Pixel};
    use num_traits::NumCast;
    use std::cmp::Ordering;
    use std::collections::HashMap;
    use std::num::Wrapping;
    use std::string::String;
    #[derive(Eq, PartialEq, Hash, Clone, Debug)]
    pub enum Color {
        Black,
        Red,
        Green,
        Yellow,
        Blue,
        Magenta,
        Cyan,
        DarkGray,
        LightGray,
        LightRed,
        LightGreen,
        LightYellow,
        LightBlue,
        LightMagenta,
        LightCyan,
        White,
    }

    lazy_static! {
        static ref COLOR_MAP: HashMap<Color, (u32, u32, u32)> = {
            let mut m = HashMap::new();
            use Color::*;
            m.insert(Black, (0, 0, 0));
            m.insert(Red, (205, 0, 0));
            m.insert(Green, (0, 205, 0));
            m.insert(Yellow, (205, 205, 0));
            m.insert(Blue, (0, 0, 238));
            m.insert(Magenta, (205, 0, 205));
            m.insert(Cyan, (0, 205, 205));
            m.insert(LightGray, (229, 229, 229));
            m.insert(DarkGray, (127, 127, 127));
            m.insert(LightRed, (255, 0, 0));
            m.insert(LightGreen, (0, 255, 0));
            m.insert(LightYellow, (255, 255, 0));
            m.insert(LightBlue, (92, 92, 255));
            m.insert(LightMagenta, (255, 0, 255));
            m.insert(LightCyan, (0, 255, 255));
            m.insert(White, (255, 255, 255));
            m
        };
    }

    pub fn fix_aspect_ratio<'a, I: GenericImageView>(
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
    pub fn to_ascii<'a, P: Pixel>(
        image: &'a ImageBuffer<P, Vec<P::Subpixel>>,
        buckets: u32,
    ) -> Vec<char>
    where
        P: 'static,
    {
        image
            .enumerate_pixels()
            .map(|(_, _, pixel)| {
                let index: u32 = NumCast::from(pixel.to_luma().channels()[0]).unwrap();
                super::ASCII_CHARS[(index / buckets) as usize]
            })
            .collect()
    }

    pub fn to_ascii_preserve_color<'a, P: Pixel>(
        image: &'a ImageBuffer<P, Vec<P::Subpixel>>,
        buckets: u32,
    ) -> Vec<(char, Color)>
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
                (
                    super::ASCII_CHARS[(index / buckets) as usize],
                    nearest_color((r, g, b)),
                )
            })
            .collect()
    }

    pub fn color_compare(
        source: (u32, u32, u32),
    ) -> Box<dyn FnMut(&(&Color, &(u32, u32, u32)), &(&Color, &(u32, u32, u32))) -> Ordering> {
        let (source1, source2, source3) = source;
        Box::new(move |a, b| {
            let square = |val| Wrapping(val) * Wrapping(val);
            let (a1, a2, a3) = a.1;
            let (b1, b2, b3) = b.1;
            if square(a1.overflowing_sub(source1).0)
                + square(a2.overflowing_sub(source2).0)
                + square(a3.overflowing_sub(source3).0)
                < square(b1.overflowing_sub(source1).0)
                    + square(b2.overflowing_sub(source2).0)
                    + square(b3.overflowing_sub(source3).0)
            {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        })
    }

    pub fn nearest_color(color: (u32, u32, u32)) -> Color {
        COLOR_MAP
            .clone()
            .iter()
            .min_by(color_compare(color))
            .unwrap()
            .0
            .clone()
    }

    pub fn color_char(c: char, color: Color) -> String {
        use self::Color::*;
        use colored::*;
        match color {
            Black => c.to_string().black().to_string(),
            Red => c.to_string().red().to_string(),
            Green => c.to_string().green().to_string(),
            Yellow => c.to_string().yellow().to_string(),
            Blue => c.to_string().blue().to_string(),
            Magenta => c.to_string().magenta().to_string(),
            Cyan => c.to_string().cyan().to_string(),
            LightGray => c.to_string().white().to_string(),
            DarkGray => c.to_string().bright_black().to_string(),
            LightRed => c.to_string().bright_red().to_string(),
            LightGreen => c.to_string().bright_green().to_string(),
            LightYellow => c.to_string().bright_yellow().to_string(),
            LightBlue => c.to_string().bright_blue().to_string(),
            LightMagenta => c.to_string().bright_magenta().to_string(),
            LightCyan => c.to_string().bright_cyan().to_string(),
            White => c.to_string().bright_white().to_string(),
        }
    }
}

pub fn asciify_image(filepath: &str, image_width: u32, correct_for_font: bool) -> Vec<String> {
    let img = image::open(filepath).unwrap();
    let scaled_image = util::fix_aspect_ratio(&img, image_width, correct_for_font);
    let raw_ascii = util::to_ascii(&scaled_image, 25);
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
) -> Vec<String> {
    let img = image::open(filepath).unwrap();
    let scaled_image = util::fix_aspect_ratio(&img, image_width, correct_for_font);
    let raw_ascii = util::to_ascii_preserve_color(&scaled_image, 25);
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
        current_string.push_str(&util::color_char(
            raw_ascii[pixel_idx as usize].0,
            raw_ascii[pixel_idx as usize].1.clone(),
        ));
        char_idx = char_idx + 1;
    }
    output
}

#[cfg(test)]
mod tests {
    #[test]
    fn asciify_test() {
        let ascii = super::asciify_image_color("/home/nick/Downloads/test.png", 100, true);
        for line in ascii {
            println!("{}", line);
        }
    }
}
