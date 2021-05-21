use std::io::{stdout, Write};
use std::path::Path;

use image::imageops;
use image::io::Reader as ImgReader;

use terminal_size::{terminal_size, Height, Width};

use crate::ansi::TermAction;
use crate::err::{RTError, Result};
use crate::options::DisplayUnit;
use crate::utils::wait_for_kb;

pub fn backspace(len: isize, unit: DisplayUnit) {
    if unit.is_char() {
        TermAction::MoveCursor(-len, 0)
            .then(TermAction::EraseLineFromCursor)
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
        println!();
        img.resize_exact(w, h, imageops::FilterType::CatmullRom)
            .grayscale()
            .to_bytes()
            .into_iter()
            .map(|b| ASCII_CHARS[NUM_CHARS - 1 - (NUM_CHARS * (b as usize) / 256)])
            .for_each(|c| print!("{}", c));
        let _ = stdout().flush();
        wait_for_kb();
        Ok(())
    } else {
        Err(RTError::Internal("Could not get the terminal dimensions"))
    }
}

pub fn img_to_term<P: AsRef<Path>>(path: P) -> Result<()> {
    // I thought I stopped having to look at ugly type names when I decided not to use C++
    const TERM_COLORS: [(&str, [u8; 3]); 14] = [
        ("\x1b[0;41m", [128, 0, 0]), // red
        ("\x1b[0;101m", [255, 0, 0]),
        ("\x1b[0;42m", [0, 128, 0]), // green
        ("\x1b[0;102m", [0, 255, 0]),
        ("\x1b[0;43m", [128, 128, 0]), // yellow
        ("\x1b[0;103m", [255, 255, 0]),
        ("\x1b[0;44m", [0, 0, 128]), // blue
        ("\x1b[0;104m", [0, 0, 255]),
        ("\x1b[0;45m", [128, 0, 128]), // magenta
        ("\x1b[0;105m", [255, 0, 255]),
        ("\x1b[0;46m", [0, 128, 128]), // cyan
        ("\x1b[0;106m", [0, 255, 255]),
        ("\x1b[0;47m", [128, 128, 128]), // grey
        ("\x1b[0;107m", [255, 255, 255]),
    ];
    if let Some((Width(w), Height(h))) = terminal_size() {
        let (w, h) = (w as u32, h as u32);
        let img = ImgReader::open(path)?.decode()?;

        println!();
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
            .for_each(|c| print!("{} ", c));
        let _ = stdout().flush();
        wait_for_kb();
        TermAction::ResetColor.execute();
        Ok(())
    } else {
        Err(RTError::Internal("Could not get the terminal dimensions"))
    }
}
