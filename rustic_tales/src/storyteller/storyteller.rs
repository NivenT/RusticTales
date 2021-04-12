use std::collections::HashMap;
use std::fs;
use std::io::{stdout, Write};
use std::path::Path;
use std::{thread::sleep, time::Duration};

use script::token::Token;

use crate::ansi::TermAction;
use crate::commands::*;
use crate::err::{RTError, Result};
use crate::options::{DisplayUnit, STOptions};
use crate::utils::wait_for_enter;

use super::story::{Bookmark, Page, Story};
use super::unit::Unit;

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

        // color support (e.g. "\033[%dm")
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
            story.num_pages(),
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
                            .skip(self.story.get_place().letter)
                            .next()
                            .expect("story.place should be valid index")
                    );
                    if self.story.get_place().letter + 1 == w.chars().count() {
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
            self.write(self.story.get_place());
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
        wait_for_enter(&format!("{}\nThe end...", self.get_val("NORMAL")));
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