use std::fs::File;
use std::path::Path;
use std::str::FromStr;

use ron::de::from_reader;
use ron::ser::{to_writer_pretty, PrettyConfig};
use serde::{Deserialize, Serialize};

use crate::err::{RTError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScrollRate {
    Millis { num: usize, ms: u64 }, // num symbols every ms milliseconds
    Lines(usize),                   // display ??? lines at a time
    OnePage,                        // display 1 page at a time
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct STOptions {
    pub scroll_rate: ScrollRate,
    pub disp_by: DisplayUnit,
    pub stories_directory: String,
}

impl Default for STOptions {
    fn default() -> Self {
        use ScrollRate::*;
        STOptions {
            scroll_rate: Millis { num: 5, ms: 743 },
            disp_by: DisplayUnit::Word,
            stories_directory: "stories".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Options {
    st_opts: STOptions,
    file_ignore_patterns: Vec<String>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            st_opts: STOptions::default(),
            file_ignore_patterns: vec!["*~".to_owned()],
        }
    }
}

impl Options {
    pub fn to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = File::create(path)?;
        to_writer_pretty(file, self, PrettyConfig::default())?;
        Ok(())
    }
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let opts = from_reader(File::open(path)?)?;
        Ok(opts)
    }
    pub fn get_ignored(&self) -> &Vec<String> {
        &self.file_ignore_patterns
    }
    pub fn get_story_opts(&self) -> &STOptions {
        &self.st_opts
    }
    pub fn get_story_folder(&self) -> &String {
        &self.st_opts.stories_directory
    }
}
