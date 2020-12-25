use std::path::Path;

use image::imageops;
use image::io::Reader as ImgReader;

use terminal_size::{terminal_size, Height, Width};

use crate::ansi::TermAction;
use crate::err::{RTError, Result};
use crate::storyteller::DisplayUnit;

pub fn backspace(len: isize, unit: DisplayUnit) {
    if unit.is_char() {
        TermAction::MoveCursor(-len, 0)
            .and_then(TermAction::EraseLineFromCursor)
            .execute()
    } else {
        unimplemented!()
    }
}

pub fn img_to_ascii<P: AsRef<Path>>(path: P) -> Result<()> {
    const ASCII_CHARS: [char; 12] = ['@', '#', 'S', '%', '?', '*', '+', ';', ':', ',', '.', ' '];
    const NUM_CHARS: usize = ASCII_CHARS.len();

    if let Some((Width(w), Height(h))) = terminal_size() {
        let (w, h) = (w as u32, h as u32);
        let img = ImgReader::open(path)?.decode()?;
        // TODO: Get terminal dimensions
        let img = img
            .resize_exact(w, h, imageops::FilterType::CatmullRom)
            .grayscale();

        let ascii = img
            .to_bytes()
            .into_iter()
            .map(|b| ASCII_CHARS[NUM_CHARS - 1 - (NUM_CHARS * (b as usize) / 256)]);

        println!();
        for c in ascii {
            print!("{}", c);
        }
        Ok(())
    } else {
        Err(RTError::Internal("Could not get the terminal dimensions"))
    }
}
