use std::collections::HashMap;
use std::fs;
use std::io::{stdout, Write};
use std::path::Path;
use std::str::FromStr;
use std::{thread::sleep, time::Duration};

use regex::Regex;

use terminal_size::{terminal_size, Height, Width};

use script::token::{tokenize, Token};

use super::ansi::TermAction;
use super::commands::*;
use super::err::{RTError, Result};
use super::wait_for_enter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayUnit {
    Char,
    Word,
}

impl FromStr for DisplayUnit {
    type Err = RTError;

    fn from_str(s: &str) -> Result<Self> {
        if s.eq_ignore_ascii_case("chars") {
            Ok(DisplayUnit::Char)
        } else if s.eq_ignore_ascii_case("words") {
            Ok(DisplayUnit::Word)
        } else {
            Err(RTError::InvalidInput(
                "DisplayUnit can only be constructed from 'chars' or 'words'".to_string(),
            ))
        }
    }
}

impl DisplayUnit {
    pub fn is_char(&self) -> bool {
        matches!(self, DisplayUnit::Char)
    }
}

// TODO: Add JSON config file (files to ignore, auto vs. manual scroll, etc.)
#[derive(Debug, Clone)]
struct STOptions {
    ms_per_symbol: usize,
    disp_by: DisplayUnit,
}

impl Default for STOptions {
    fn default() -> Self {
        STOptions {
            ms_per_symbol: 458,
            disp_by: DisplayUnit::Word,
        }
    }
}

#[derive(Debug, Clone)]
enum Unit {
    Char(char),
    Word(String),
    Special(Token), // Not Token::Text or Token::Char
}

impl Unit {
    fn from_token(tkn: &Token) -> Vec<Unit> {
        use Unit::*;
        match tkn {
            Token::Text(s) => {
                let re = Regex::new("[[:space:]]+").expect("Typo if this does not work");
                re.split(s).map(|w| Word(w.to_string())).collect()
            }
            Token::Char(c) => vec![Unit::Char(*c)],
            t => vec![Special(t.clone())],
        }
    }
    fn len(&self) -> usize {
        match self {
            Unit::Char(_) => 1,
            Unit::Word(w) => w.chars().count() + 1, // +1 cause of the space after the word
            Unit::Special(t) => match t {
                Token::Command(_, _) => 0,
                Token::Variable(_) => 7, // can't know variable length a priori so just guess
                Token::Symbol(s) => s.len() + 2,
                _ => unreachable!(),
            },
        }
    }
    fn is_page_end(&self) -> bool {
        matches!(self, Unit::Special(Token::PageEnd))
    }
    fn is_word(&self) -> bool {
        matches!(self, Unit::Word(_))
    }
}

#[derive(Debug, Clone, Default)]
struct Page {
    // index into the 'contents' of the containing story
    start_idx: usize,
    len: usize,
}

impl Page {
    fn max_page_len() -> usize {
        if let Some((Width(w), Height(h))) = terminal_size() {
            (w as usize) * (h as usize)
        } else {
            80 * 25
        }
    }
}

// Should this be copy?
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Bookmark {
    page: usize,
    word: usize,
    letter: usize,
}

#[derive(Debug, Clone)]
struct Story {
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
    fn is_over(&self) -> bool {
        self.place.page >= self.pages.len()
            || self.pages[self.place.page].start_idx + self.place.word >= self.contents.len()
    }
    fn get<'a>(&'a self, place: Bookmark) -> &'a Unit {
        &self.contents[self.pages[place.page].start_idx + place.word]
    }
    // Returns true if entered a new page
    fn advance(&mut self, disp_by: DisplayUnit) -> bool {
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
}

// TODO: Make state machine (e.g. so can backspace over time)
#[derive(Debug, Clone)]
pub struct StoryTeller {
    story: Story,
    options: STOptions,
    env: HashMap<String, String>,
}

impl StoryTeller {
    fn prepare_builtins() -> HashMap<String, String> {
        let mut env = HashMap::new();

        // color support ("\033[%dm")
        //   FG val + 10 ==    BG val
        // dark val + 60 == light val
        const COLORS: [&str; 7] = ["RED", "GREEN", "YELLOW", "BLUE", "MAGENTA", "CYAN", "GREY"];
        for (val, name) in COLORS.iter().enumerate() {
            env.insert(format!("{}_DFG", name), format!("\x1b[{}m", val + 31));
            env.insert(format!("{}_DBG", name), format!("\x1b[{}m", val + 41));
            env.insert(format!("{}_LFG", name), format!("\x1b[{}m", val + 91));
            env.insert(format!("{}_LBG", name), format!("\x1b[{}m", val + 101));
        }
        // This placement is awkward, but can't put it before calls to
        // "env.insert" since this closure mutably borrows env
        let mut add = |k: &str, v: &str| {
            env.insert(k.to_string(), v.to_string());
        };

        add("DEFCOL_FG", "\x1b[39m");
        add("DEFCOL_BG", "\x1b[49m");
        add("RED_LFG", "\x1b[91m");

        add("BOLD", "\x1b[1m");
        add("DIM", "\x1b[2m");
        add("UNDERLINE", "\x1b[4m");
        add("BLINK", "\x1b[5m");
        add("NORMAL", "\x1b[0m");

        env
    }

    pub fn new<P: AsRef<Path>>(story: P) -> Result<Self> {
        let story: Story = fs::read_to_string(story)?.parse()?;

        println!("{:?}", story);
        println!(
            "There were {} pages. The max page length is {}.",
            story.pages.len(),
            Page::max_page_len()
        );
        wait_for_enter("...");

        Ok(StoryTeller {
            story: story,
            options: STOptions::default(),
            env: StoryTeller::prepare_builtins(),
        })
    }
    fn write(&mut self, place: Bookmark) {
        // self.eval_command mutably borrows self, so need to clone or something
        let unit = self.story.get(place).clone();
        match unit {
            Unit::Char(c) => print!("{}", c),
            Unit::Word(w) => {
                if self.options.disp_by == DisplayUnit::Char {
                    print!(
                        "{}",
                        w.chars()
                            .skip(self.story.place.letter)
                            .next()
                            .expect("story.place should be valid index")
                    );
                    if self.story.place.letter + 1 == w.chars().count() {
                        print!(" ");
                    }
                } else {
                    print!("{} ", w);
                }
            }
            Unit::Special(t) => {
                assert!(!t.is_text() && !t.is_page_end());
                match t {
                    Token::Variable(s) => print!("{}", self.get_val(&s)),
                    Token::Command(func, args) => {
                        if let Err(e) = self.eval_command(&func, &args) {
                            eprintln!("\nError: {}", e)
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
    pub fn tell(&mut self) {
        self.setup();
        while !self.story.is_over() {
            self.write(self.story.place);
            let _ = stdout().flush();

            sleep(Duration::from_millis(self.options.ms_per_symbol as u64));
            if self.story.advance(self.options.disp_by) {
                self.turn_page();
            }
        }
        self.cleanup();
    }

    fn setup(&self) {
        TermAction::ClearScreen
            .then(TermAction::SetCursor(0, 0))
            .then(TermAction::ResetColor)
            .execute();
    }
    fn cleanup(&self) {
        wait_for_enter(&format!("{}The end...", self.get_val("NORMAL")));
        TermAction::ClearScreen
            .then(TermAction::SetCursor(0, 0))
            .execute();
    }
    fn turn_page(&self) {
        wait_for_enter("Next page...");
        TermAction::ClearScreen
            .then(TermAction::SetCursor(0, 0))
            .then(TermAction::ResetColor)
            .execute();
    }

    fn get_val(&self, var: &str) -> String {
        self.env.get(var).unwrap_or(&String::new()).clone()
    }
    fn eval_command(&mut self, func: &str, args: &Vec<String>) -> Result<()> {
        match func {
            "backspace" => {
                if args.len() < 2 {
                    Err(RTError::InvalidInput(
                        "'backspace' requires two arguments".to_string(),
                    ))
                } else {
                    Ok(backspace(args[0].parse()?, args[1].parse()?))
                }
            }
            "display_img" => {
                if !matches!(args.len(), 1 | 2) {
                    Err(RTError::InvalidInput(
                        "'display_img' takes 1 or 2 args".to_string(),
                    ))
                } else if args.len() == 2 && args[1].eq_ignore_ascii_case("term") {
                    img_to_term(&args[0])
                } else {
                    img_to_ascii(&args[0])
                }
            }
            _ => Err(RTError::UnrecognizedCommand(func.to_string())),
        }
    }
}
