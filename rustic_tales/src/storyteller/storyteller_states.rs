use humantime::parse_duration;

use std::fs;
use std::io::{stdout, Write};
use std::{thread::sleep, time::Duration};

use script::token::{tokenize, Token};

use crate::ansi::TermAction;
use crate::commands::prompts::*;
use crate::commands::*;
use crate::err::{RTError, Result};
use crate::options::{DisplayUnit, STOptions, ScrollRate};
use crate::utils::*;

use super::story::{Span, Story};
use super::storyteller_base::*;
use super::unit::Unit;

#[derive(Default)]
pub struct Debug;
#[derive(Default)]
pub struct Telling;
pub struct Paused;

impl<'a> StoryTeller<'a, Telling> {
    fn write(&mut self) {
        // self.eval_command mutably borrows self, so need to clone or something
        let unit = self.story.get_curr().clone();
        match unit {
            Unit::Char(c) => print!("{}", c),
            Unit::Word(w) => {
                if self.opts().disp_by == DisplayUnit::Char {
                    print!(
                        "{}",
                        w.chars()
                            .nth(self.story.get_place().letter)
                            .expect("story.place should be a valid index")
                    );
                } else {
                    print!("{}", w);
                }
            }
            Unit::WhiteSpace(w) => print!("{}", w),
            Unit::Special(t) => {
                debug_assert!(!t.is_text() && !t.is_page_end() && !t.is_sect_start());
                match t {
                    Token::Variable(s) => print!("{}", self.get_val(&s)),
                    Token::Symbol(s) => print!("${}$", s),
                    Token::Command(func, args, _) => {
                        // Do I want this?
                        let _ = std::io::stdout().flush();
                        if let Err(e) = self.eval_command(&func, &args) {
                            eprintln!("\nError: {}", e)
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
    fn write_and_advance(&mut self, disp_by: DisplayUnit) -> Option<Span> {
        self.write();
        let the_story_goes_on = !self.story.is_over();
        let ret = self.story.advance(disp_by);
        if ret == Span::Page && !self.story.is_over() {
            self.turn_page();
        }
        the_story_goes_on.then(|| ret)
    }
    fn tell_millis(&mut self, num: usize, ms: u64) -> SnippetInfo {
        let mut info = SnippetInfo::Nothing;
        for _ in 0..num {
            let span = self.write_and_advance(self.opts().disp_by);
            if span == None {
                info = SnippetInfo::StoryOver;
                break;
            } else if self.story.get_curr().is_blocking_command() {
                info = SnippetInfo::EndedWith(Span::BlockingCommand);
                break;
            }
        }
        let _ = stdout().flush();
        sleep(Duration::from_millis(ms));
        info
    }
    fn tell_words(&mut self, num: usize) -> SnippetInfo {
        let mut last_span = Span::Word;
        'outer: for _ in 0..num {
            loop {
                // There's gotta be a better way to write this
                let span = match self.write_and_advance(DisplayUnit::Word) {
                    Some(span) => span,
                    None => return SnippetInfo::StoryOver,
                };
                if self.story.get_curr().is_blocking_command() {
                    last_span = Span::BlockingCommand;
                    break 'outer;
                } else if matches!(span, Span::Page | Span::Line) {
                    last_span = span;
                    break 'outer;
                } else if !matches!(span, Span::WhiteSpace) {
                    break;
                }
            }
        }
        let _ = stdout().flush();
        self.wait_kb();
        SnippetInfo::EndedWith(last_span)
    }
    fn tell_lines(&mut self, num: usize) -> SnippetInfo {
        let mut last_span = Span::Line;
        'outer: for _ in 0..num {
            // There's gotta be a better way to write this
            let mut span = match self.write_and_advance(DisplayUnit::Word) {
                Some(span) => span,
                None => return SnippetInfo::StoryOver,
            };
            while span != Span::Line {
                if self.story.get_curr().is_blocking_command() {
                    last_span = Span::BlockingCommand;
                    break 'outer;
                } else if span == Span::Page {
                    last_span = span;
                    break 'outer;
                }
                span = match self.write_and_advance(DisplayUnit::Word) {
                    Some(span) => span,
                    None => return SnippetInfo::StoryOver,
                };
            }
        }
        let _ = stdout().flush();
        SnippetInfo::EndedWith(last_span)
    }
    fn tell_onepage(&mut self) -> SnippetInfo {
        let mut last_span = Span::Page;
        let mut span = match self.write_and_advance(DisplayUnit::Word) {
            Some(span) => span,
            None => return SnippetInfo::StoryOver,
        };
        while span != Span::Page {
            if self.story.get_curr().is_blocking_command() {
                last_span = Span::BlockingCommand;
                break;
            }
            span = match self.write_and_advance(DisplayUnit::Word) {
                Some(span) => span,
                None => return SnippetInfo::StoryOver,
            };
        }
        let _ = stdout().flush();
        SnippetInfo::EndedWith(last_span)
    }

    fn turn_page(&self) {
        wait_for_enter("\nNext page...");
        TermAction::ClearScreen
            .then(TermAction::SetCursor(0, 0))
            .then(TermAction::ResetColor)
            .execute();
    }
    fn parse_arg(&self, arg: &str) -> Result<String> {
        use Token::*;
        let mut tkns = tokenize(&arg);
        if tkns.len() != 1 {
            let msg = format!("Could not parse arg '{}'", arg);
            let e = RTError::InvalidInput(msg);
            return Err(e);
        }
        let tkn = tkns.pop().unwrap();
        match tkn {
            Text(s) => Ok(s),
            Symbol(s) => Ok(s),
            Variable(v) => Ok(self.get_val(&v)),
            _ => {
                let msg = format!("commands can only take text, symbols, and variables. {} is none of they above.", arg);
                let e = RTError::InvalidInput(msg);
                Err(e)
            }
        }
    }
    fn eval_command(&mut self, func: &str, args: &[String]) -> Result<()> {
        match func {
            "backspace" => {
                if args.len() < 2 {
                    Err(RTError::InvalidInput(
                        "'backspace' requires two arguments".to_string(),
                    ))
                } else {
                    backspace(args[0].parse()?, args[1].parse()?);
                    Ok(())
                }
            }
            "display_img" => {
                if !matches!(args.len(), 1 | 2) {
                    Err(RTError::InvalidInput(
                        "'display_img' takes 1 or 2 args".to_string(),
                    ))
                } else if args.len() == 2 && args[1].eq_ignore_ascii_case("term") {
                    img_to_term(self.get_full_path(&args[0]))
                } else {
                    img_to_ascii(self.get_full_path(&args[0]))
                }
            }
            "prompt_yesno" => {
                if args.is_empty() {
                    let e = RTError::InvalidInput("'prompt_yesno' requires an argument".to_owned());
                    Err(e)
                } else {
                    // This should probably check that args[0] is a Token::Symbol, but what kinda
                    // person has the patience to write correct code?
                    self.set_val(
                        self.parse_arg(&args[0])?,
                        prompt_yesno(args.get(1).cloned()),
                    );
                    Ok(())
                }
            }
            "jump_if_eq" => {
                if args.len() < 3 {
                    let msg = "'jump_if_eq' requires at least 3 arguments".to_owned();
                    let e = RTError::InvalidInput(msg);
                    Err(e)
                } else {
                    let lhs = self.parse_arg(&args[0])?;
                    let rhs = self.parse_arg(&args[1])?;
                    let jump_happened = if lhs == rhs {
                        self.story.jump_to_section(args.get(2))
                    } else {
                        self.story.jump_to_section(args.get(3))
                    };
                    if jump_happened { /* TODO */ }
                    Ok(())
                }
            }
            "pause" => {
                if args.len() != 1 {
                    let msg = "'pause' takes exactly 1 argument".to_owned();
                    let e = RTError::InvalidInput(msg);
                    Err(e)
                } else {
                    let dur = parse_duration(&args[0])?;
                    let _ = std::io::stdout().flush();
                    let orig = no_term_echo();
                    sleep(dur);
                    restore_term(orig);
                    // Ignore all keys user pressed while paused
                    exhaust_kb();
                    Ok(())
                }
            }
            "force_input" => {
                if args.len() != 1 {
                    let msg = "'force_input' takes exactly 1 argument".to_owned();
                    let e = RTError::InvalidInput(msg);
                    Err(e)
                } else {
                    force_input(&self.parse_arg(&args[0])?)
                }
            }
            "choice_menu" => {
                if args.len() < 2 {
                    let msg = "'choice_menu' requires at least 1 choice".to_owned();
                    let e = RTError::InvalidInput(msg);
                    Err(e)
                } else {
                    // This should probably check that args[0] is a Token::Symbol, but what kinda
                    // person has the patience to write correct code?
                    self.set_val(self.parse_arg(&args[0])?, choice_menu(&args[1..])?);
                    Ok(())
                }
            }
            "wait_kb" => Ok(self.wait_kb()),
            "move_cursor_back" => {
                if args.len() != 1 {
                    Err(RTError::WrongNumArguments(
                        "move_cursor_back",
                        "1",
                        args.len(),
                    ))
                } else {
                    TermAction::MoveCursor(-args[0].parse()?, 0).execute();
                    Ok(())
                }
            }
            _ => Err(RTError::UnrecognizedCommand(func.to_string())),
        }
    }

    // low-key I should just mem::transmute and hope Rust has enough guarantees that things just work
    fn pause(self) -> StoryTeller<'a, Paused> {
        StoryTeller {
            story: self.story,
            options: self.options,
            env: self.env,
            state: Paused,
        }
    }
}

impl<'a> StoryTeller<'a, Paused> {
    fn resume(self) -> StoryTeller<'a, Telling> {
        StoryTeller {
            story: self.story,
            options: self.options,
            env: self.env,
            state: Telling,
        }
    }
}

// Felt like separating out debug stuff
impl<'a> StoryTeller<'a, Debug> {
    pub fn get_tokens(story: &str) -> Result<Vec<Token>> {
        Ok(tokenize(&fs::read_to_string(story)?))
    }
    pub fn get_story(&self) -> &Story {
        &self.story
    }
}

pub enum StatefulStoryTeller<'a> {
    Telling(StoryTeller<'a, Telling>),
    Paused(StoryTeller<'a, Paused>),
}

impl<'a> StatefulStoryTeller<'a> {
    pub fn from_telling(st: StoryTeller<'a, Telling>) -> Self {
        StatefulStoryTeller::Telling(st)
    }
    pub fn setup(&mut self, opts: &'a STOptions) {
        use StatefulStoryTeller::*;
        match self {
            Telling(st) => st.setup(opts),
            Paused(st) => st.setup(opts),
        }
    }
    pub fn step(&mut self) -> SnippetInfo {
        use StatefulStoryTeller::*;
        match self {
            Telling(st) => {
                let snippet_info = match st.opts().scroll_rate {
                    ScrollRate::Millis { num, ms } => st.tell_millis(num, ms),
                    ScrollRate::Words(num) => st.tell_words(num),
                    ScrollRate::Lines(num) => st.tell_lines(num),
                    ScrollRate::OnePage => st.tell_onepage(),
                };
                //println!("snippet info: {:?}", snippet_info);
                if snippet_info.should_wait_for_kb() {
                    st.wait_kb();
                }
                snippet_info
            }
            _ => SnippetInfo::Nothing,
        }
    }
    pub fn transition(self) -> Self {
        use StatefulStoryTeller::*;
        match self {
            Telling(st) => {
                if let Some(b'p') = get_kb() {
                    Paused(st.pause())
                } else {
                    Telling(st)
                }
            }
            Paused(st) => {
                if let Some(b'p') = get_kb() {
                    Telling(st.resume())
                } else {
                    Paused(st)
                }
            }
        }
    }
    pub fn cleanup(&self) {
        use StatefulStoryTeller::*;
        match self {
            Telling(st) => st.cleanup(),
            Paused(st) => st.cleanup(),
        }
    }
}
