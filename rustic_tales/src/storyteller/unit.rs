use regex::Regex;

//use either::Either;

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
    pub fn from_token<'a>(tkn: Token) -> Vec<Unit> {
        use Unit::*;
        match tkn {
            Token::Text(s) => {
                let re = Regex::new("[[:space:]]+").expect("Typo if this does not work");
                re.find_iter(&s)
                    .scan(0, |word_start, mat| {
                        let unit1 = Word(s[*word_start..mat.start()].to_owned());
                        let unit2 = WhiteSpace(mat.as_str().to_owned());
                        *word_start = mat.end();
                        // there's a better way to do this. I just don't know it...
                        let iter = std::iter::once(unit1).chain(std::iter::once(unit2));
                        Some(iter)
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
