use std::path::Path;

use image::imageops;
use image::io::Reader as ImgReader;

use terminal_size::{terminal_size, Height, Width};

use crate::buffer::{BaseColor, Color, TermBuffer, TextEffect};
use crate::err::{RTError, Result};
use crate::options::DisplayUnit;

pub mod prompts;

// TODO: Implement the rest of this
pub fn backspace(count: usize, unit: DisplayUnit, buf: &mut TermBuffer) {
    if unit.is_char() {
        buf.erase_chars(count);
    } else {
        unimplemented!()
    }
}

pub fn img_to_ascii(path: impl AsRef<Path>, buf: &mut TermBuffer) -> Result<()> {
    // Adapted from the short sequence here: http://paulbourke.net/dataformats/asciiart/
    const ASCII_CHARS: &[u8] = " .,:;-=+*#&%@$".as_bytes();
    const NUM_CHARS: usize = ASCII_CHARS.len();

    if let Some((Width(w), Height(h))) = terminal_size() {
        let (w, h) = (w as u32, h as u32);
        let img = ImgReader::open(path)?.decode()?;
        buf.turn_page();
        img.resize_exact(w, h, imageops::FilterType::Lanczos3)
            .grayscale()
            .to_bytes()
            .into_iter()
            .map(|b| ASCII_CHARS[NUM_CHARS * (b as usize) / 256])
            .for_each(|c| buf.write_char(c as char));
        Ok(())
    } else {
        Err(RTError::Internal("Could not get the terminal dimensions"))
    }
}

pub fn img_to_term(path: impl AsRef<Path>, buf: &mut TermBuffer) -> Result<()> {
    use BaseColor::*;
    // I thought I stopped having to look at ugly type names when I decided not to use C++
    const TERM_COLORS: [(Color, [u8; 3]); 15] = [
        (Color::dark(Black), [0, 0, 0]), // black
        (Color::dark(Red), [128, 0, 0]), // red
        (Color::light(Red), [255, 0, 0]),
        (Color::dark(Green), [0, 128, 0]), // green
        (Color::light(Green), [0, 255, 0]),
        (Color::dark(Yellow), [128, 128, 0]), // yellow
        (Color::light(Yellow), [255, 255, 0]),
        (Color::dark(Blue), [0, 0, 128]), // blue
        (Color::light(Blue), [0, 0, 255]),
        (Color::dark(Magenta), [128, 0, 128]), // magenta
        (Color::light(Magenta), [255, 0, 255]),
        (Color::dark(Cyan), [0, 128, 128]), // cyan
        (Color::light(Cyan), [0, 255, 255]),
        (Color::dark(Grey), [128, 128, 128]), // grey
        (Color::light(Grey), [255, 255, 255]),
    ];
    if let Some((Width(w), Height(h))) = terminal_size() {
        let (w, h) = (w as u32, h as u32);
        let img = ImgReader::open(path)?.decode()?;

        buf.turn_page();
        // I can't tell if this is trashy or idiomatic rust
        img.resize_exact(w, h, imageops::FilterType::CatmullRom)
            .into_rgb8()
            .pixels()
            .map(|p| {
                TERM_COLORS
                    .iter()
                    .min_by_key(|&c| {
                        c.1.iter()
                            .zip(p.0.iter())
                            .map(|(&l, &r)| l as isize - r as isize)
                            .map(|diff| diff * diff)
                            .sum::<isize>()
                    })
                    .expect("Iterator is not empty")
                    .0
            })
            .for_each(|c| {
                buf.undo_modifiers();
                buf.add_bg_color(c); // Add color modifier
                buf.write_char(' ');
            });
        buf.add_text_effect(TextEffect::None);
        Ok(())
    } else {
        Err(RTError::Internal("Could not get the terminal dimensions"))
    }
}
