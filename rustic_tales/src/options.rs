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

// TODO: Add JSON config file (files to ignore, auto vs. manual scroll, etc.)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct STOptions {
    pub ms_per_symbol: usize,
    pub disp_by: DisplayUnit,
}

impl Default for STOptions {
    fn default() -> Self {
        STOptions {
            ms_per_symbol: 458,
            disp_by: DisplayUnit::Word,
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
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let file = File::create(path)?;
        to_writer_pretty(file, self, PrettyConfig::default())?;
        Ok(())
    }
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let opts = from_reader(File::open(path)?)?;
        Ok(opts)
    }
}
