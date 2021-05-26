use std::str::FromStr;

use terminal_size::{terminal_size, Height, Width};

use script::token::{tokenize, Token};

use crate::err::{RTError, Result};
use crate::options::DisplayUnit;

use super::unit::Unit;

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
                    if next.is_page_end() || next.is_sect_start() {
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
                || units[idx].is_sect_start()
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

#[derive(Debug, Clone)]
pub struct Section {
    pages: Vec<Page>,
    name: String,
}

impl Section {
    // Returns number of units in this page
    fn extract_section(name: &str, units: &[Unit], offset: usize) -> (Section, usize) {
        let mut pages = Vec::new();
        let mut idx = 0;
        while idx < units.len() && !units[idx].is_sect_start() {
            let (page, offset) = Page::extract_page(&units[idx..], offset + idx);
            if offset > 0 {
                if !page.lines.is_empty() {
                    pages.push(page);
                }
                idx += offset;
            } else {
                break;
            }
        }
        let sect = Section {
            pages,
            name: name.to_owned(),
        };
        (sect, idx)
    }
    /*
        pub fn num_pages(&self) -> usize {
            self.pages.len()
        }
        pub fn start_idx(&self) -> Option<usize> {
            self.pages
                .first()
                .and_then(|page| page.lines.first())
                .map(|line| line.start_idx)
        }
    */
}

// Ord defaults to lexicographic order based on top-down declaration of members
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bookmark {
    pub page: usize,
    pub line: usize,
    pub word: usize,
    pub letter: usize,
}

impl Bookmark {
    fn reset(&mut self) {
        *self = Bookmark::default();
    }
}

// This is an awful name, but naming things is hard...
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Span {
    PAGE,
    LINE,
    WORD,
    CHAR,
    COMMAND,
    SECTION,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StoryFlags {
    just_changed_section: bool,
}

// Instead of directly printing everything, should there be a buffer keeping better track of words and whatnot?
#[derive(Debug, Clone)]
pub struct Story {
    sections: Vec<Section>,
    contents: Vec<Unit>,
    place: Bookmark,
    curr_sect_idx: usize,
    flags: StoryFlags,
}

impl FromStr for Story {
    type Err = RTError;

    fn from_str(s: &str) -> Result<Self> {
        let tkns = tokenize(s);
        let contents: Vec<_> = tkns
            .into_iter()
            // Oh, so I had heard of flat map? (see 4f03ae49273ff751a6f4603bd3194d16d65448ec)
            .flat_map(|t| Unit::from_token(&t))
            .collect();

        let mut sects = Vec::new();
        let mut idx = 0;
        let mut name = "Main Section"; // The (default) name of the 0th section
        while idx < contents.len() {
            if let Unit::Special(Token::SectionStart(sname)) = &contents[idx] {
                name = sname;
                idx += 1;
            } else {
                let (sect, offset) = Section::extract_section(name, &contents[idx..], idx);
                if offset > 0 {
                    if !sect.pages.is_empty() {
                        sects.push(sect);
                    }
                    idx += offset;
                }
            }
        }

        Ok(Story {
            sections: sects,
            contents,
            place: Bookmark::default(),
            curr_sect_idx: 0,
            flags: StoryFlags::default(),
        })
    }
}

impl Story {
    fn end(&self) -> Bookmark {
        let sect = self.curr_sect();
        let last_page = sect.pages.len().saturating_sub(1);
        let last_line = sect.pages[last_page].lines.len().saturating_sub(1);
        let last_word = sect.pages[last_page].lines[last_line].len.saturating_sub(1);

        let last_word_idx = sect.pages[last_page].lines[last_line].start_idx + last_word;
        let last_letter = if let Unit::Word(w) = &self.contents[last_word_idx] {
            w.chars().count().saturating_sub(1)
        } else {
            0
        };
        Bookmark {
            page: last_page,
            line: last_line,
            word: last_word,
            letter: last_letter,
        }
    }
    // This *might* no longer be what I want now that I've added sections
    // This checks to see if the current section is over
    // I *don't* want sections to automatically roll over to the next one,
    // so I *think* this *is* what I want.
    pub fn is_over(&self) -> bool {
        self.place >= self.end()
    }
    pub fn curr_sect(&self) -> &Section {
        &self.sections[self.curr_sect_idx]
    }
    pub fn get(&self, place: Bookmark) -> &Unit {
        let sect = self.curr_sect();
        &self.contents[sect.pages[place.page].lines[place.line].start_idx + place.word]
    }
    /*
        pub fn get_by_absolute_idx(&self, idx: usize) -> &Unit {
            &self.contents[idx]
        }
    */
    pub fn get_curr(&self) -> &Unit {
        self.get(self.place)
    }
    pub fn advance(&mut self, disp_by: DisplayUnit) -> Span {
        if self.flags.just_changed_section {
            self.flags.just_changed_section = false;
            return Span::SECTION;
        }
        // Would rather call self.curr_sect(), but then the complier seems to think
        // I'm borrowing all of self and not just one field, since things are happening
        // across function boundaries (at least, I think this is the issue)
        let sect = &self.sections[self.curr_sect_idx];
        let unit = self.get(self.place).clone(); // I really hate these clone's
        if disp_by == DisplayUnit::Word || !unit.is_word() {
            // There's probably a more concise way to write this, but this works
            self.place.letter = 0;
            self.place.word += 1;
            if self.place.word == sect.pages[self.place.page].lines[self.place.line].len {
                self.place.word = 0;
                self.place.line += 1;
                if self.place.line == sect.pages[self.place.page].lines.len() {
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
    // Returns true if a jump occured
    pub fn jump_to_section(&mut self, sect_identifier: Option<&String>) -> bool {
        self.flags.just_changed_section = false;
        if let Some(ident) = sect_identifier {
            let idx = ident.parse::<usize>().ok().or_else(|| {
                self.sections
                    .iter()
                    .enumerate()
                    .find(|(_, sect)| &sect.name == ident)
                    .map(|(i, _)| i)
            });
            if let Some(idx) = idx {
                let old_idx = self.curr_sect_idx;
                self.curr_sect_idx = idx;
                self.place.reset();
                self.flags.just_changed_section = old_idx != idx;
            }
        }
        self.flags.just_changed_section
    }
    pub fn get_place(&self) -> Bookmark {
        self.place
    }
}

impl Story {
    pub fn get_contents(&self) -> &Vec<Unit> {
        &self.contents
    }
}
