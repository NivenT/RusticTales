use std::str::FromStr;

use terminal_size::{terminal_size, Height, Width};

use script::token::tokenize;

use crate::err::{RTError, Result};
use crate::options::DisplayUnit;

use super::unit::Unit;

#[derive(Debug, Clone, Default)]
pub struct Page {
    // index into the 'contents' of the containing story
    start_idx: usize,
    len: usize,
}

impl Page {
    pub fn max_page_len() -> usize {
        if let Some((Width(w), Height(h))) = terminal_size() {
            (w as usize) * (h as usize)
        } else {
            80 * 25
        }
    }
}

// Should this be copy?
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Bookmark {
    pub page: usize,
    pub word: usize,
    pub letter: usize,
}

// Instead of directly printing everything, should there be a buffer keeping better track of words and whatnot?
#[derive(Debug, Clone)]
pub struct Story {
    // Should I impl Iterator for Story?
    pages: Vec<Page>,
    contents: Vec<Unit>,
    place: Bookmark,
}

impl FromStr for Story {
    type Err = RTError;

    fn from_str(s: &str) -> Result<Self> {
        let tkns = tokenize(s);
        let contents: Vec<_> = tkns
            .into_iter()
            .flat_map(|t| Unit::from_token(&t))
            .collect();

        let mut pages = Vec::new();
        let mut idx = 0;
        loop {
            let mut curr_page = Page {
                start_idx: idx,
                ..Page::default()
            };
            curr_page.len = contents[idx..]
                .iter()
                .scan(0, |len, next| {
                    if next.is_page_end() {
                        None
                    } else if *len + next.len() > Page::max_page_len() {
                        None
                    } else {
                        *len += next.len();
                        Some(next)
                    }
                })
                .count();
            idx += curr_page.len;
            pages.push(curr_page);
            if idx < contents.len() {
                idx += contents[idx].is_page_end() as usize;
            } else {
                break;
            }
        }

        Ok(Story {
            pages: pages,
            contents: contents,
            place: Bookmark::default(),
        })
    }
}

impl Story {
    pub fn is_over(&self) -> bool {
        self.place.page >= self.pages.len()
            || self.pages[self.place.page].start_idx + self.place.word >= self.contents.len()
    }
    pub fn get<'a>(&'a self, place: Bookmark) -> &'a Unit {
        &self.contents[self.pages[place.page].start_idx + place.word]
    }
    // Returns true if entered a new page
    pub fn advance(&mut self, disp_by: DisplayUnit) -> bool {
        let unit = self.get(self.place).clone(); // I really hate these clone's
        if disp_by == DisplayUnit::Word || !unit.is_word() {
            self.place.letter = 0;
            self.place.word += 1;
            if self.place.word == self.pages[self.place.page].len {
                self.place.word = 0;
                self.place.page += 1;
                true
            } else {
                false
            }
        } else if let Unit::Word(w) = unit {
            self.place.letter += 1;
            if self.place.letter == w.chars().count() {
                self.advance(DisplayUnit::Word)
            } else {
                false
            }
        } else {
            unreachable!("unit is a word or is not a word")
        }
    }
    pub fn num_pages(&self) -> usize {
        self.pages.len()
    }
    pub fn get_place(&self) -> Bookmark {
        self.place
    }
}
