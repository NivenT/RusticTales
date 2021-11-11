use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::err::Result;
use crate::options::{STOptions, ScrollRate};
use crate::utils::*;

use super::story::{Span, Story};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnippetInfo {
    Nothing,
    EndedWith(Span),
    Transitioning,
    StoryOver,
}

impl SnippetInfo {
    pub fn should_wait_for_kb(&self, scroll_rate: &ScrollRate) -> bool {
        use SnippetInfo::*;
        if matches!(self, EndedWith(Span::BlockingCommand)) {
            true
        } else if scroll_rate.is_lines() {
            matches!(self, EndedWith(Span::Line))
        } else if scroll_rate.is_words() {
            matches!(self, EndedWith(Span::Word))
        } else {
            false
        }
    }
    pub fn story_ended(&self) -> bool {
        use SnippetInfo::*;
        matches!(self, StoryOver)
    }
    pub fn page_over(&self) -> bool {
        use SnippetInfo::*;
        matches!(self, EndedWith(Span::Page))
    }
}

#[derive(Debug, Clone)]
pub struct StoryTeller<'a, S> {
    pub(super) story: Story,
    pub(super) options: Option<&'a STOptions>,
    pub(super) env: HashMap<String, String>,
    pub(super) state: S,
}

// Shared functionality
impl<'a, S> StoryTeller<'a, S> {
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

        add("BOLD", "\x1b[1m");
        add("DIM", "\x1b[2m");
        add("ITALIC", "\x1b[3m");
        add("UNDERLINE", "\x1b[4m");
        add("BLINK", "\x1b[5m");
        add("SIX_AEC", "\x1b[6m"); // AEC = ANSI Escape Code
        add("INVERSE", "\x1b[7m");
        add("EIGHT_AEC", "\x1b[8m");
        add("STRIKETHROUGH", "\x1b[9m");
        add("NORMAL", "\x1b[0m");

        if let Some(user) = get_user() {
            env.insert("USER_NAME".to_owned(), user);
        }

        env
    }
    pub fn setup(&mut self, opts: &'a STOptions) {
        self.options = Some(opts);
    }

    pub(super) fn opts(&self) -> &STOptions {
        self.options
            .expect("opts should only be called after setup")
    }
    pub(super) fn get_full_path(&self, p: &str) -> String {
        format!("{}/{}", self.opts().stories_directory, p)
    }
    pub(super) fn get_val(&self, var: &str) -> String {
        self.env.get(var).unwrap_or(&String::new()).clone()
    }
    pub(super) fn set_val(&mut self, var: String, val: String) {
        self.env.insert(var, val);
    }
}

impl<'a, S: Default> StoryTeller<'a, S> {
    pub fn new<P: AsRef<Path>>(story: P) -> Result<Self> {
        let story: Story = fs::read_to_string(story)?.parse()?;

        Ok(StoryTeller {
            story,
            options: None,
            env: StoryTeller::<S>::prepare_builtins(),
            state: Default::default(),
        })
    }
}
