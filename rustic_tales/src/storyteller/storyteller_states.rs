use std::fs;

use script::token::{tokenize, Token};

use crate::err::Result;

use super::story::Story;
use super::storyteller_base::*;

#[derive(Default)]
pub struct Debug;
#[derive(Default)]
pub struct Telling;

// Felt like separating out debug stuff
impl<'a> StoryTeller<'a, Debug> {
    pub fn get_tokens(story: &str) -> Result<Vec<Token>> {
        Ok(tokenize(&fs::read_to_string(story)?))
    }
    pub fn get_story(&self) -> &Story {
        &self.story
    }
}
