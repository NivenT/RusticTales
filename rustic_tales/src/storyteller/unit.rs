use regex::Regex;

use either::Either;

use script::token::Token;

#[derive(Debug, Clone)]
pub enum Unit {
    Char(char),
    Word(String),
    WhiteSpace(String),
    Special(Token), // Not Token::Text or Token::Char
}

impl Unit {
    // TODO: Figure out how to return an impl Iterator<Item=Unit>
    pub fn from_token(tkn: Token) -> Vec<Unit> {
        use Unit::*;
        match tkn {
            Token::Text(s) => {
                // This Regex explictly checks for a newline for dumb reasons
                // Basically, extrace_page assumes that each Unit fits on a single line,
                // so it really doesn't like something like WhiteSpace("\n\n") or WhiteSpace(" \n")
                let re = Regex::new("(\n|[[\t ]]+)").expect("Typo if this does not work");
                // Could I have made this any worse?
                re.find_iter(&s)
                    .map(|mat| (mat.start(), mat.end(), mat.as_str()))
                    .chain(std::iter::once((0, 0, "SENTINEL")))
                    .scan(0, |word_start, mat| {
                        if (mat.1 - mat.0) != mat.2.len() {
                            if *word_start < s.len() {
                                Some(Either::Left(std::iter::once(Word(
                                    s[*word_start..].to_owned(),
                                ))))
                            } else {
                                None
                            }
                        } else {
                            let unit1 = Word(s[*word_start..mat.0].to_owned());
                            let unit2 = WhiteSpace(mat.2.to_owned());
                            *word_start = mat.1;
                            // there's a better way to do this. I just don't know it...
                            let iter = std::iter::once(unit1).chain(std::iter::once(unit2));
                            Some(Either::Right(iter))
                        }
                    })
                    .flatten()
                    .collect()
            }
            Token::Char(c) => vec![Char(c)],
            t => vec![Special(t)],
        }
    }
    // basically len but keeps track of vertical spacing as well
    pub fn area(&self) -> (usize, usize) {
        match self {
            Unit::Char('\n') => (0, 1),
            Unit::Char(_) => (1, 0),
            // words aren't allowed to have newline/space type characters in them
            Unit::Word(w) => (w.chars().count() + 1, 0),
            Unit::WhiteSpace(w) => w.chars().fold((0, 0), |acc, c| match c {
                '\n' => (acc.0, acc.1 + 1),
                '\0' => acc,
                _ => (acc.0 + 1, acc.1),
            }),
            Unit::Special(t) => match t {
                // might need to depend on the command in the future
                Token::Command(..) => (0, 0),
                // can't know variable length a priori so just guess
                // ^^^^^^^ This is dumb. I should make pagination more dynamic at some point
                Token::Variable(_) => (7, 0),
                Token::Symbol(s) => (s.len() + 2, 0),
                _ => unreachable!(),
            },
        }
    }
    // This is kinda dumb
    pub fn is_newline(&self) -> bool {
        match self {
            Unit::Char('\n') => true,
            Unit::Word(w) | Unit::WhiteSpace(w) => w == "\n",
            _ => false,
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            Unit::Char('\0') => true,
            Unit::Word(w) | Unit::WhiteSpace(w) => w.is_empty(),
            Unit::Special(t) => t.is_empty(),
            _ => false,
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
        matches!(self, Unit::Special(Token::Command(..)))
    }
    pub fn is_blocking_command(&self) -> bool {
        matches!(self, Unit::Special(Token::Command(.., true)))
    }
    pub fn is_whitespace(&self) -> bool {
        matches!(self, Unit::WhiteSpace(..))
    }
}
