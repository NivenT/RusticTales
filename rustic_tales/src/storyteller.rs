use std::fs;
use std::io::{stdout, Write};
use std::path::Path;
use std::str::FromStr;
use std::{thread::sleep, time::Duration};

use regex::Regex;

use script::token::{tokenize, Token};

use super::err::{RTError, Result};

#[derive(Debug, Clone, Copy)]
enum DisplayUnit {
    Char,
    Word,
}

#[derive(Debug, Clone)]
struct STOptions {
    ms_per_symbol: usize,
    disp_by: DisplayUnit,
}

impl Default for STOptions {
    fn default() -> Self {
        STOptions {
            ms_per_symbol: 700,
            disp_by: DisplayUnit::Word,
        }
    }
}

#[derive(Debug, Clone)]
enum Unit {
    Char(char), // Can probably get rid of this?
    Word(String),
    Special(Token), // Not Token::Text
}

impl Unit {
    fn from_token(tkn: &Token) -> Vec<Unit> {
        use Unit::*;
        match tkn {
            Token::Text(s) => {
                let re = Regex::new("[[:space:]]+").expect("Type if this does not work");
                re.split(s).map(|w| Word(w.to_string())).collect()
            }
            t @ _ => vec![Special(t.clone())],
        }
    }
}

#[derive(Debug, Clone)]
struct Story {
    content: Vec<Unit>,
    place: usize,
}

impl FromStr for Story {
    type Err = RTError;

    fn from_str(s: &str) -> Result<Self> {
        let tkns = tokenize(s);
        let content: Vec<_> = tkns
            .into_iter()
            .map(|t| Unit::from_token(&t).into_iter())
            .flatten()
            .collect();

        Ok(Story {
            content: content,
            place: 0,
        })
    }
}

impl Story {
    fn is_over(&self) -> bool {
        self.place >= self.content.len()
    }
}

#[derive(Debug, Clone)]
pub struct StoryTeller {
    story: Story,
    options: STOptions,
}

impl StoryTeller {
    pub fn new<P: AsRef<Path>>(story: P) -> Result<Self> {
        let story: Story = fs::read_to_string(story)?.parse()?;
        Ok(StoryTeller {
            story: story,
            options: STOptions::default(),
        })
    }
    pub fn tell(&mut self) {
        while !self.story.is_over() {
            let word = &self.story.content[self.story.place];
            match word {
                Unit::Char(c) => print!("{}", c),
                Unit::Word(w) => print!("{} ", w),
                Unit::Special(t) => {
                    assert!(!t.is_text());
                }
            }
            stdout().flush();
            sleep(Duration::from_millis(self.options.ms_per_symbol as u64));
            self.story.place += 1;
        }
    }
}
