use std::collections::HashMap;
use std::fs;
use std::io::{stdout, Write};
use std::path::Path;
use std::str::FromStr;
use std::{thread::sleep, time::Duration};

use regex::Regex;

use script::token::{tokenize, Token};

use super::ansi::TermAction;
use super::commands::*;
use super::err::{RTError, Result};

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
    #[allow(dead_code)]
    Char(char), // Can probably get rid of this?
    Word(String),
    Special(Token), // Not Token::Text
}

impl Unit {
    fn from_token(tkn: &Token) -> Vec<Unit> {
        use Unit::*;
        match tkn {
            Token::Text(s) => {
                let re = Regex::new("[[:space:]]+").expect("Typo if this does not work");
                re.split(s).map(|w| Word(w.to_string())).collect()
            }
            t => vec![Special(t.clone())],
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
        Ok(StoryTeller {
            story: story,
            options: STOptions::default(),
            env: StoryTeller::prepare_builtins(),
        })
    }
    pub fn tell(&mut self) {
        self.setup();
        while !self.story.is_over() {
            let word = self.story.content[self.story.place].clone();
            match word {
                Unit::Char(c) => print!("{}", c),
                Unit::Word(w) => print!("{} ", w),
                Unit::Special(t) => {
                    assert!(!t.is_text());
                    match t {
                        Token::Variable(s) => {
                            let val = self.get_val(&s);
                            print!("{}", val);
                        }
                        Token::Command(func, args) => {
                            if let Err(e) = self.eval_command(&func, &args) {
                                eprintln!("\nError: {}", e)
                            }
                        }
                        _ => {}
                    }
                }
            }
            let _ = stdout().flush();
            sleep(Duration::from_millis(self.options.ms_per_symbol as u64));
            self.story.place += 1;
        }
        self.cleanup();
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
    fn setup(&self) {
        TermAction::ClearScreen
            .and_then(TermAction::SetCursor(0, 0))
            .and_then(TermAction::ResetColor)
            .execute();
    }
    fn cleanup(&self) {
        println!(
            "{}{}The end...",
            self.get_val("DEFCOL_BG"),
            self.get_val("DEFCOL_FG")
        );

        let mut temp = String::new();
        let _ = std::io::stdin().read_line(&mut temp);

        TermAction::ClearScreen
            .and_then(TermAction::SetCursor(0, 0))
            .execute();
    }
}
