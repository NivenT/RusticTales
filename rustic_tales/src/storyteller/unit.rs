use regex::Regex;

use script::token::Token;

#[derive(Debug, Clone)]
pub enum Unit {
    Char(char),
    Word(String),
    Special(Token), // Not Token::Text or Token::Char
}

impl Unit {
    pub fn from_token(tkn: &Token) -> Vec<Unit> {
        use Unit::*;
        match tkn {
            Token::Text(s) => {
                let re = Regex::new("[[:space:]]+").expect("Typo if this does not work");
                re.split(s).map(|w| Word(w.to_string())).collect()
            }
            Token::Char(c) => vec![Unit::Char(*c)],
            t => vec![Special(t.clone())],
        }
    }
    /*
        pub fn len(&self) -> usize {
            match self {
                Unit::Char(_) => 1,
                Unit::Word(w) => w.chars().count() + 1, // +1 cause of the space after the word
                Unit::Special(t) => match t {
                    Token::Command(_, _) => 0,
                    Token::Variable(_) => 7, // can't know variable length a priori so just guess
                    Token::Symbol(s) => s.len() + 2,
                    _ => unreachable!(),
                },
            }
        }
    */
    // basically len but keeps track of vertical spacing as well
    pub fn area(&self) -> (usize, usize) {
        match self {
            Unit::Char('\n') => (0, 1),
            Unit::Char(_) => (1, 0),
            // words aren't allowed to have newline/space type characters in them
            Unit::Word(w) => (w.chars().count() + 1, 0),
            Unit::Special(t) => match t {
                // might need to depend on the command in the future
                Token::Command(_, _) => (0, 0),
                // can't know variable length a priori so just guess
                // ^^^^^^^ This is dumb. I should make pagination more dynamic at some point
                Token::Variable(_) => (7, 0),
                Token::Symbol(s) => (s.len() + 2, 0),
                _ => unreachable!(),
            },
        }
    }
    pub fn is_page_end(&self) -> bool {
        matches!(self, Unit::Special(Token::PageEnd))
    }
    pub fn is_sect_start(&self) -> bool {
        matches!(self, Unit::Special(Token::SectionStart(_)))
    }
    pub fn is_word(&self) -> bool {
        matches!(self, Unit::Word(_))
    }
    pub fn is_command(&self) -> bool {
        matches!(self, Unit::Special(Token::Command(_, _)))
    }
}
