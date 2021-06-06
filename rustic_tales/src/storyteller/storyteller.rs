use std::collections::HashMap;
use std::fs;
use std::io::{stdout, Write};
use std::path::Path;
use std::{thread::sleep, time::Duration};

use humantime::parse_duration;

use script::token::{tokenize, Token};

use crate::ansi::TermAction;
use crate::commands::prompts::*;
use crate::commands::*;
use crate::err::{RTError, Result};
use crate::options::{DisplayUnit, STOptions, ScrollRate};
use crate::utils::*;

use super::story::{Span, Story};
use super::unit::Unit;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SnippetInfo {
    Nothing,
    EndedWith(Span),
    StoryOver,
}

impl SnippetInfo {
    fn should_wait_for_kb(&self) -> bool {
        use SnippetInfo::*;
        matches!(
            self,
            EndedWith(Span::Line) | EndedWith(Span::BlockingCommand)
        )
    }
    fn story_ended(&self) -> bool {
        use SnippetInfo::*;
        matches!(self, StoryOver)
    }
}

// TODO: Make state machine (e.g. so can backspace over time)
#[derive(Debug, Clone)]
pub struct StoryTeller<'a> {
    story: Story,
    options: Option<&'a STOptions>,
    env: HashMap<String, String>,
}

impl<'a> StoryTeller<'a> {
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
            env.insert(k.to_owned(), v.to_owned());
        };

        add("DEFCOL_FG", "\x1b[39m");
        add("DEFCOL_BG", "\x1b[49m");
        add("RED_LFG", "\x1b[91m");

        add("BOLD", "\x1b[1m");
        add("DIM", "\x1b[2m");
        add("UNDERLINE", "\x1b[4m");
        add("BLINK", "\x1b[5m");
        add("NORMAL", "\x1b[0m");

        if let Some(user) = get_user() {
            env.insert("USER_NAME".to_owned(), user);
        }

        env
    }
    fn opts(&self) -> &STOptions {
        self.options
            .expect("opts should only be called after setup")
    }

    pub fn new<P: AsRef<Path>>(story: P) -> Result<Self> {
        let story: Story = fs::read_to_string(story)?.parse()?;

        Ok(StoryTeller {
            story,
            options: None,
            env: StoryTeller::prepare_builtins(),
        })
    }
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

    pub fn tell(&mut self, opts: &'a STOptions) {
        self.setup(opts);
        loop {
            let snippet_info = match self.opts().scroll_rate {
                ScrollRate::Millis { num, ms } => self.tell_millis(num, ms),
                ScrollRate::Words(num) => self.tell_words(num),
                ScrollRate::Lines(num) => self.tell_lines(num),
                ScrollRate::OnePage => self.tell_onepage(),
            };
            //println!("snippet info: {:?}", snippet_info);
            if snippet_info.should_wait_for_kb() {
                self.wait_kb();
            } else if snippet_info.story_ended() {
                break;
            }
        }
        self.cleanup();
    }

    fn setup(&mut self, opts: &'a STOptions) {
        self.options = Some(opts);
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
        wait_for_enter("\nNext page...");
        TermAction::ClearScreen
            .then(TermAction::SetCursor(0, 0))
            .then(TermAction::ResetColor)
            .execute();
    }
    fn wait_kb(&self) {
        if let Some(c) = self.opts().prompt_when_wait {
            wait_for_kb_with_prompt(c);
        } else {
            wait_for_kb()
        }
        // only waits for one byte, but some keys (e.g. arrow keys) generate multiple bytes
        // we want to exhaust all of those so the next call actually waits for a new
        // keypress. This is not the nicest way to do it, but meh
        exhaust_kb();
    }

    fn get_full_path(&self, p: &str) -> String {
        format!("{}/{}", self.opts().stories_directory, p)
    }
    fn get_val(&self, var: &str) -> String {
        self.env.get(var).unwrap_or(&String::new()).clone()
    }
    fn set_val(&mut self, var: String, val: String) {
        self.env.insert(var, val);
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
}

// Felt like separating out debug stuff
impl<'a> StoryTeller<'a> {
    pub fn get_tokens(story: &str) -> Result<Vec<Token>> {
        Ok(tokenize(&fs::read_to_string(story)?))
    }
    pub fn get_story(&self) -> &Story {
        &self.story
    }
}
