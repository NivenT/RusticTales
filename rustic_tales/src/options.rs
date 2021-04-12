use std::str::FromStr;

use crate::err::{RTError, Result};

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
