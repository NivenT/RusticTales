use std::str::FromStr;

use terminal_size::{terminal_size, Height, Width};

use script::token::tokenize;

use crate::err::{RTError, Result};
use crate::options::DisplayUnit;

use super::unit::Unit;

/*
#[derive(Debug, Clone, Copy, Default)]
pub struct Slice {
    // index into the 'contents' of the containing story
    start_idx: usize,
    len: usize,
}
*/

#[derive(Debug, Clone, Copy, Default)]
pub struct Line {
    // index into the 'contents' of the containing story
    start_idx: usize,
    len: usize,
}

impl Line {
    pub fn new(idx: usize) -> Line {
        Line {
            start_idx: idx,
            len: 0,
        }
    }
    pub fn max_line_len() -> usize {
        if let Some((Width(w), _)) = terminal_size() {
            w as usize
        } else {
            80
        }
    }
}

#[derive(Debug, Clone)]
pub struct Page {
    lines: Vec<Line>,
}

impl Page {
    pub fn new() -> Page {
        Page { lines: Vec::new() }
    }
    pub fn max_page_len() -> usize {
        Page::max_page_height() * Line::max_line_len()
    }
    pub fn max_page_height() -> usize {
        // Leave a couple lines open at the end to say 'Next page...'
        if let Some((_, Height(h))) = terminal_size() {
            h.checked_sub(2).unwrap_or(h) as usize
        } else {
            23
        }
    }
    pub fn len(&self) -> usize {
        self.lines.iter().map(|line| line.len).sum()
    }

    fn area_to_len((w, h): (usize, usize)) -> usize {
        w + h * Line::max_line_len()
    }
    // Returns number of units in this page
    fn extract_page(units: &[Unit], offset: usize) -> (Page, usize) {
        let mut page = Page::new();

        let mut idx = 0;
        loop {
            let mut curr_line = Line::new(idx + offset);
            curr_line.len = units[idx..]
                .iter()
                .scan(0, |len, next| {
                    if next.is_page_end() {
                        None
                    } else {
                        let unit_size = Page::area_to_len(next.area());
                        if *len + unit_size > Line::max_line_len() {
                            None
                        } else {
                            *len += unit_size;
                            Some(next)
                        }
                    }
                })
                .count();
            if curr_line.len != 0 {
                idx += curr_line.len;
                page.lines.push(curr_line);
            }
            if idx >= units.len()
                || page.lines.len() >= Page::max_page_height()
                || idx >= Page::max_page_len()
            {
                break;
            } else if units[idx].is_page_end() {
                idx += 1;
                break;
            }
        }
        (page, idx)
    }
}

// Ord defaults to lexicographic order based on top-down declaration of members
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bookmark {
    pub page: usize,
    pub line: usize,
    pub word: usize,
    pub letter: usize,
}

// This is an awful name, but naming things is hard...
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Span {
    PAGE,
    LINE,
    WORD,
    CHAR,
}

// Instead of directly printing everything, should there be a buffer keeping better track of words and whatnot?
#[derive(Debug, Clone)]
pub struct Story {
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
            // Oh, so I had heard of flat map?
            .flat_map(|t| Unit::from_token(&t))
            .collect();

        let mut pages = Vec::new();
        let mut idx = 0;
        loop {
            let (page, offset) = Page::extract_page(&contents[idx..], idx);
            if offset > 0 {
                if !page.lines.is_empty() {
                    println!(
                        "Adding page with {} lines and total length {}",
                        page.lines.len(),
                        page.len()
                    );
                    pages.push(page);
                }
                idx += offset;
            } else {
                break;
            }
        }

        Ok(Story {
            pages,
            contents,
            place: Bookmark::default(),
        })
    }
}

impl Story {
    fn end(&self) -> Bookmark {
        let last_page = self.pages.len().saturating_sub(1);
        let last_line = self.pages[last_page].lines.len().saturating_sub(1);
        let last_word = self.pages[last_page].lines[last_line].len.saturating_sub(1);
        // sometimes, I don't understand rustfmt's choices
        let last_letter = self.contents
            [self.pages[last_page].lines[last_line].start_idx + last_word]
            .len()
            .saturating_sub(1);
        Bookmark {
            page: last_page,
            line: last_line,
            word: last_word,
            letter: last_letter,
        }
    }

    pub fn is_over(&self) -> bool {
        /*
        self.place.page >= self.pages.len()
            || self.pages[self.place.page].start_idx + self.place.word >= self.contents.len()
         */
        self.place >= self.end()
    }
    pub fn get(&self, place: Bookmark) -> &Unit {
        &self.contents[self.pages[place.page].lines[place.line].start_idx + place.word]
    }
    pub fn advance(&mut self, disp_by: DisplayUnit) -> Span {
        let unit = self.get(self.place).clone(); // I really hate these clone's
        if disp_by == DisplayUnit::Word || !unit.is_word() {
            // There's probably a more concise way to write this, but this works
            self.place.letter = 0;
            self.place.word += 1;
            if self.place.word == self.pages[self.place.page].lines[self.place.line].len {
                self.place.word = 0;
                self.place.line += 1;
                if self.place.line == self.pages[self.place.page].lines.len() {
                    self.place.line = 0;
                    self.place.page += 1;
                    Span::PAGE
                } else {
                    Span::LINE
                }
            } else {
                Span::WORD
            }
        } else if let Unit::Word(w) = unit {
            self.place.letter += 1;
            if self.place.letter == w.chars().count() {
                self.advance(DisplayUnit::Word)
            } else {
                Span::CHAR
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
