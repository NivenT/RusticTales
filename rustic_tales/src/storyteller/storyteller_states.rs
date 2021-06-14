use humantime::parse_duration;

use std::fs;
use std::io::{stdout, Write};
use std::{thread::sleep, time::Duration};

use script::token::{tokenize, Token};

use crate::ansi::TermAction;
use crate::commands::prompts::*;
use crate::commands::*;
use crate::err::{RTError, Result};
use crate::options::{DisplayUnit, ScrollRate};
use crate::utils::*;

use super::story::{Span, Story};
use super::storyteller_base::*;
use super::unit::Unit;

// All the states StoryTeller can be in
#[derive(Default, Debug)]
pub struct Debug;
#[derive(Default, Debug)]
pub struct Telling {
    to: TransitionInfo,
}
#[derive(Default, Debug)]
pub struct Paused {
    from: TransitionInfo,
}
#[derive(Debug, Default)]
pub struct Quit;
#[derive(Debug, Clone, Copy)]
pub struct Backspacing {
    unit: DisplayUnit,
    num: usize,
    pace: Duration,
}
#[derive(Debug, Clone)]
pub struct Repeating {
    text: String,
    num: usize,
    pace: Duration,
}
#[derive(Debug, Clone, Copy)]
pub struct WaitingForKB(Option<char>);

#[derive(Debug, Clone)]
enum TransitionInfo {
    Backspacing(Backspacing),
    Repeating(Repeating),
    WaitingForKB(WaitingForKB),
    Nothing,
}

impl Default for TransitionInfo {
    fn default() -> Self {
        TransitionInfo::Nothing
    }
}

impl TransitionInfo {
    fn is_nothing(&self) -> bool {
        matches!(self, TransitionInfo::Nothing)
    }
}

// Shared functionality
impl<'a, S> StoryTeller<'a, S> {
    fn into_telling(self) -> StoryTeller<'a, Telling> {
        self.into_state_def()
    }
    fn quit(self) -> StoryTeller<'a, Quit> {
        self.into_state_def()
    }
    fn into_state_def<SS: Default>(self) -> StoryTeller<'a, SS> {
        StoryTeller {
            story: self.story,
            options: self.options,
            env: self.env,
            state: SS::default(),
        }
    }
    fn into_state<SS>(self, state: SS) -> StoryTeller<'a, SS> {
        StoryTeller {
            story: self.story,
            options: self.options,
            env: self.env,
            state,
        }
    }
}

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
            } else if !self.state.to.is_nothing() {
                info = SnippetInfo::Transitioning;
                break;
            }
        }
        let _ = stdout().flush();
        sleep(Duration::from_millis(ms));
        info
    }
    fn tell_words(&mut self, num: usize) -> SnippetInfo {
        let mut info = SnippetInfo::EndedWith(Span::Word);
        'outer: for _ in 0..num {
            loop {
                // There's gotta be a better way to write this
                let span = match self.write_and_advance(DisplayUnit::Word) {
                    Some(span) => span,
                    None => return SnippetInfo::StoryOver,
                };
                if self.story.get_curr().is_blocking_command() {
                    info = SnippetInfo::EndedWith(Span::BlockingCommand);
                    break 'outer;
                } else if !self.state.to.is_nothing() {
                    info = SnippetInfo::Transitioning;
                    break 'outer;
                } else if matches!(span, Span::Page | Span::Line) {
                    info = SnippetInfo::EndedWith(span);
                    break 'outer;
                } else if !matches!(span, Span::WhiteSpace) {
                    break;
                }
            }
        }
        let _ = stdout().flush();
        self.wait_kb();
        info
    }
    fn tell_lines(&mut self, num: usize) -> SnippetInfo {
        let mut info = SnippetInfo::EndedWith(Span::Line);
        'outer: for _ in 0..num {
            // There's gotta be a better way to write this
            let mut span = match self.write_and_advance(DisplayUnit::Word) {
                Some(span) => span,
                None => return SnippetInfo::StoryOver,
            };
            while span != Span::Line {
                if self.story.get_curr().is_blocking_command() {
                    info = SnippetInfo::EndedWith(Span::BlockingCommand);
                    break 'outer;
                } else if !self.state.to.is_nothing() {
                    info = SnippetInfo::Transitioning;
                    break 'outer;
                } else if span == Span::Page {
                    info = SnippetInfo::EndedWith(span);
                    break 'outer;
                }
                span = match self.write_and_advance(DisplayUnit::Word) {
                    Some(span) => span,
                    None => return SnippetInfo::StoryOver,
                };
            }
        }
        let _ = stdout().flush();
        info
    }
    fn tell_onepage(&mut self) -> SnippetInfo {
        let mut info = SnippetInfo::EndedWith(Span::Page);
        let mut span = match self.write_and_advance(DisplayUnit::Word) {
            Some(span) => span,
            None => return SnippetInfo::StoryOver,
        };
        while span != Span::Page {
            if self.story.get_curr().is_blocking_command() {
                info = SnippetInfo::EndedWith(Span::BlockingCommand);
                break;
            } else if !self.state.to.is_nothing() {
                info = SnippetInfo::Transitioning;
                break;
            }
            span = match self.write_and_advance(DisplayUnit::Word) {
                Some(span) => span,
                None => return SnippetInfo::StoryOver,
            };
        }
        let _ = stdout().flush();
        info
    }

    fn turn_page(&self) {
        wait_for_enter("\nNext page...");
        TermAction::ClearScreen
            .then(TermAction::SetCursor(0, 0))
            //.then(TermAction::ResetColor)
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
                } else if args.len() == 2 || args[2] != "one_by_one" {
                    //                    ^^^^^^^^^^^^^^^^^^^^^^^^^^
                    // Really hope Rust does short circuiting
                    backspace(args[0].parse()?, args[1].parse()?);
                    Ok(())
                } else {
                    let pace = if args.len() >= 4 {
                        parse_duration(&args[3])?
                    } else {
                        Duration::from_millis(250)
                    };
                    self.state.to = TransitionInfo::Backspacing(Backspacing {
                        unit: args[1].parse()?,
                        num: args[0].parse()?,
                        pace,
                    });
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
            "clear_screen" => {
                clear_screen();
                Ok(())
            }
            "repeat" => {
                if args.len() != 3 {
                    Err(RTError::WrongNumArguments("repeat", "3", args.len()))
                } else {
                    self.state.to = TransitionInfo::Repeating(Repeating {
                        text: self.parse_arg(&args[0])?,
                        num: args[1].parse()?,
                        pace: parse_duration(&args[2])?,
                    });
                    Ok(())
                }
            }
            _ => Err(RTError::UnrecognizedCommand(func.to_string())),
        }
    }
    fn wait_kb(&mut self) {
        // Just in case there are some left over keys that haven't been exhausted yet. We'd hate
        // for this function to return before the user actually performs a new key press.
        exhaust_kb();
        self.state.to = TransitionInfo::WaitingForKB(WaitingForKB(self.opts().prompt_when_wait));
        /*
        if let Some(c) = self.opts().prompt_when_wait {
            wait_for_kb_with_prompt(c);
        } else {
            wait_for_kb()
        }
        */
    }

    fn pause(self) -> StoryTeller<'a, Paused> {
        StoryTeller {
            story: self.story,
            options: self.options,
            env: self.env,
            state: Paused::default(),
        }
    }
    fn transition(self) -> StatefulStoryTeller<'a> {
        match self.state.to.clone() {
            TransitionInfo::Backspacing(bs) => {
                StatefulStoryTeller::Backspacing(self.into_state(bs))
            }
            TransitionInfo::Repeating(rp) => StatefulStoryTeller::Repeating(self.into_state(rp)),
            TransitionInfo::WaitingForKB(wfkb) => {
                if let Some(ref c) = wfkb.0 {
                    print!("{}", c);
                    let _ = stdout().flush();
                }
                StatefulStoryTeller::WaitingForKB(self.into_state(wfkb))
            }
            TransitionInfo::Nothing => StatefulStoryTeller::Telling(self),
        }
    }
}

impl<'a> StoryTeller<'a, Paused> {
    fn resume(self) -> StatefulStoryTeller<'a> {
        match self.state.from.clone() {
            TransitionInfo::Backspacing(bs) => {
                StatefulStoryTeller::Backspacing(self.into_state(bs))
            }
            TransitionInfo::Repeating(rp) => StatefulStoryTeller::Repeating(self.into_state(rp)),
            TransitionInfo::WaitingForKB(wfkb) => {
                StatefulStoryTeller::WaitingForKB(self.into_state(wfkb))
            }
            TransitionInfo::Nothing => StatefulStoryTeller::Telling(self.into_telling()),
        }
    }
}

impl<'a> StoryTeller<'a, Backspacing> {
    fn pause(self) -> StoryTeller<'a, Paused> {
        StoryTeller {
            story: self.story,
            options: self.options,
            env: self.env,
            state: Paused {
                from: TransitionInfo::Backspacing(self.state),
            },
        }
    }
}

impl<'a> StoryTeller<'a, Repeating> {
    fn pause(self) -> StoryTeller<'a, Paused> {
        StoryTeller {
            story: self.story,
            options: self.options,
            env: self.env,
            state: Paused {
                from: TransitionInfo::Repeating(self.state),
            },
        }
    }
}

impl<'a> StoryTeller<'a, WaitingForKB> {
    fn key_pressed(self) -> StoryTeller<'a, Telling> {
        if self.state.0.is_some() {
            TermAction::EraseCharsOnLine(1).execute();
        }
        // get_kb only gets at most one byte, but some keys (e.g. arrow keys)
        // generate multiple bytes we want to exhaust all of those so the next call actually
        // waits for a new keypress. This is not the nicest way to do it, but meh
        exhaust_kb();
        self.into_telling()
    }
}

impl<'a> StoryTeller<'a, Debug> {
    pub fn get_tokens(story: &str) -> Result<Vec<Token>> {
        Ok(tokenize(&fs::read_to_string(story)?))
    }
    pub fn get_story(&self) -> &Story {
        &self.story
    }
}

#[derive(Debug)]
pub enum StatefulStoryTeller<'a> {
    Telling(StoryTeller<'a, Telling>),
    Paused(StoryTeller<'a, Paused>),
    Quit(StoryTeller<'a, Quit>),
    Backspacing(StoryTeller<'a, Backspacing>),
    Repeating(StoryTeller<'a, Repeating>),
    WaitingForKB(StoryTeller<'a, WaitingForKB>),
}

impl<'a> StatefulStoryTeller<'a> {
    pub fn from_telling(st: StoryTeller<'a, Telling>) -> Self {
        StatefulStoryTeller::Telling(st)
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
                if snippet_info.should_wait_for_kb() {
                    st.wait_kb();
                }
                snippet_info
            }
            Paused(..) => SnippetInfo::Nothing,
            Quit(..) => SnippetInfo::StoryOver,
            Backspacing(st) => {
                if st.state.num > 0 {
                    if st.state.unit.is_char() {
                        TermAction::EraseCharsOnLine(1).execute();
                        st.state.num -= 1;
                        let _ = stdout().flush();
                        // This number should be somehow custimizable
                        sleep(st.state.pace);
                    } else {
                        unimplemented!()
                    }
                }
                SnippetInfo::Nothing
            }
            Repeating(st) => {
                if st.state.num > 0 {
                    print!("{}", st.state.text);
                    st.state.num -= 1;
                    let _ = stdout().flush();
                    if st.state.num > 0 {
                        sleep(st.state.pace);
                    }
                }
                SnippetInfo::Nothing
            }
            WaitingForKB(..) => SnippetInfo::Nothing,
        }
    }
    pub fn transition(self) -> Self {
        use StatefulStoryTeller::*;

        const ESC_KEY: u8 = 27;
        match get_kb() {
            Some(b'p') => match self {
                Telling(st) => Paused(st.pause()),
                Paused(st) => st.resume(),
                Backspacing(st) => Paused(st.pause()),
                Repeating(st) => Paused(st.pause()),
                Quit(..) | WaitingForKB(..) => self,
            },
            Some(b'q') => match self {
                Telling(st) => Quit(st.quit()),
                Paused(st) => Quit(st.quit()),
                Backspacing(st) => Quit(st.quit()),
                Repeating(st) => Quit(st.quit()),
                WaitingForKB(st) => Telling(st.key_pressed()),
                Quit(..) => self,
            },
            Some(ESC_KEY) => match self {
                Telling(st) => Quit(st.quit()),
                Paused(st) => Quit(st.quit()),
                Backspacing(st) => Quit(st.quit()),
                Repeating(st) => Quit(st.quit()),
                WaitingForKB(st) => Quit(st.quit()),
                Quit(..) => self,
            },
            k => match self {
                Backspacing(st) if st.state.num == 0 => Telling(st.into_telling()),
                Repeating(st) if st.state.num == 0 => Telling(st.into_telling()),
                WaitingForKB(st) if k.is_some() => Telling(st.key_pressed()),
                Telling(st) => st.transition(),
                _ => self,
            },
        }
    }
    pub fn cleanup(&self) {
        use StatefulStoryTeller::*;
        match self {
            Telling(st) => st.cleanup(),
            Paused(st) => st.cleanup(),
            Backspacing(st) => st.cleanup(),
            Repeating(st) => st.cleanup(),
            WaitingForKB(st) => st.cleanup(),
            Quit(st) => st.cleanup(),
        }
    }
}
