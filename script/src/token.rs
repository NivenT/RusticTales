use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Text(String),                 // blah
    Command(String, Vec<String>), // {{ cmd : arg1 |,| arg2 |,| ... }}
    Variable(String),             // ${{var}}
    Symbol(String),               // $sym$
    PageEnd,                      // /PAGE/
    Char(char),                   // {c}
    SectionStart(String),         // #=$ SECTION_NAME $=#
}

impl Token {
    pub fn is_text(&self) -> bool {
        matches!(self, Token::Text(_) | Token::Char(_))
    }
    pub fn is_page_end(&self) -> bool {
        self == &Token::PageEnd
    }
    pub fn is_sect_start(&self) -> bool {
        matches!(self, Token::SectionStart(_))
    }
    pub fn is_empty(&self) -> bool {
        match self {
            Token::Text(s) => s.is_empty(),
            Token::Command(c, _) => c.is_empty(),
            Token::Variable(v) => v.is_empty(),
            Token::Symbol(s) => s.is_empty(),
            Token::PageEnd => false,
            Token::Char(c) => c == &'\0',
            // I think this is the right answer
            Token::SectionStart(_) => false,
        }
    }
}

fn parse_symbol(stream: &str) -> Option<(Token, usize)> {
    let re =
        Regex::new(r"^\$([^[[:space:]]]+)\$").expect("If this regex is invalid, that is a bug");
    re.captures(stream)
        .map(|cap| (Token::Symbol(cap[1].to_string()), cap[0].len()))
}

// A command should take up an entire line
fn parse_command(stream: &str) -> Option<(Token, usize)> {
    // regex are completely incomprehensible (it doesn't help that I suck at writing them)
    let re = Regex::new(r"^\{\{[[:space:]]*(\b\w+\b)[[:space:]]*:(.*)\}\}[[:space:]]*(\n|$)")
        .expect("If this is invalid, there is a bug");
    re.captures(stream).map(|cap| {
        let name = cap[1].to_string();
        let arg_list = &cap[2];
        let arg_list = if arg_list.is_empty() {
            vec![]
        } else {
            arg_list
                .split("|,|")
                .map(|s| s.trim().to_string())
                .collect()
        };
        (Token::Command(name, arg_list), cap[0].len())
    })
}

fn parse_variable(stream: &str) -> Option<(Token, usize)> {
    let re = Regex::new(r"^\$\{\{([^[[:space:]]\{\}]+)\}\}").expect("If bad, then bug");
    re.captures(stream)
        .map(|cap| (Token::Variable(cap[1].to_string()), cap[0].len()))
}

fn parse_pageend(stream: &str) -> Option<(Token, usize)> {
    if stream.starts_with("/PAGE/") {
        Some((Token::PageEnd, 6))
    } else {
        None
    }
}

fn parse_char(stream: &str) -> Option<(Token, usize)> {
    let re = Regex::new(r"^\{(.)\}").expect("reggie gud");
    re.captures(stream) // extracting chars from string is my least favorite part of this language
        .map(|cap| (Token::Char(cap[1].chars().next().unwrap()), cap[0].len()))
}

fn parse_sect_start(stream: &str) -> Option<(Token, usize)> {
    let re = Regex::new(r"^#=\$ (.*) \$=#(\n|$)").expect("open an issue");
    re.captures(stream)
        .map(|cap| (Token::SectionStart(cap[1].to_string()), cap[0].len()))
}

pub fn tokenize(stream: &str) -> Vec<Token> {
    let mut ret = vec![];

    let mut beg = 0;
    let mut search_pos = beg;
    while beg < stream.len() {
        let special_chars: &[char] = &['{', '$', '/', '#'];
        if let Some(end) = stream[search_pos..].find(special_chars) {
            search_pos += end;
            // (ideally) at most one of these will return Some
            const PARSE_FUNCS: [fn(&str) -> Option<(Token, usize)>; 6] = [
                parse_variable,
                parse_command,
                parse_symbol,
                parse_pageend,
                parse_char,
                parse_sect_start,
            ];

            let parsed = PARSE_FUNCS
                .iter()
                .fold(None, |acc, f| acc.or_else(|| f(&stream[search_pos..])));
            if let Some((tkn, len)) = parsed {
                ret.push(Token::Text(stream[beg..search_pos].to_string()));
                /*
                println!(
                    "parsed token '{:?}' at postion {} after \"{}\"",
                    tkn,
                    search_pos,
                    &stream[beg..search_pos]
                );
                */
                ret.push(tkn);
                search_pos += len;
                beg = search_pos;
            } else {
                search_pos += 1;
            }
        } else {
            ret.push(Token::Text(stream[beg..].trim_start().to_string()));
            beg = stream.len();
        }
    }

    ret.into_iter().filter(|t| !t.is_empty()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_parsing() {
        assert_eq!(parse_symbol("fail"), None);
        assert_eq!(
            parse_symbol("$success$"),
            Some((Token::Symbol("success".to_string()), 9))
        );
        assert_eq!(parse_symbol("$no luck&"), None);
        assert_eq!(parse_symbol("so $close$"), None);
        assert_eq!(
            parse_symbol("$special_-+-3chars$"),
            Some((Token::Symbol("special_-+-3chars".to_string()), 19))
        );
        assert_eq!(
            parse_symbol("$var$ and more text"),
            Some((Token::Symbol("var".to_string()), 5))
        );
    }
    #[test]
    fn test_command_parsing() {
        assert_eq!(parse_command("fail"), None);
        assert_eq!(
            parse_command("{{ one : arg }}"),
            Some((
                Token::Command("one".to_string(), vec!["arg".to_string()]),
                15
            ))
        );
        assert_eq!(
            parse_command("{{ properly : formatted |,| command }}"),
            Some((
                Token::Command(
                    "properly".to_string(),
                    vec!["formatted".to_string(), "command".to_string()]
                ),
                38
            ))
        );
        assert_eq!(
            parse_command("{{seems:legal|,|enough}}"),
            Some((
                Token::Command(
                    "seems".to_string(),
                    vec!["legal".to_string(), "enough".to_string()]
                ),
                24
            ))
        );
        assert_eq!(
            parse_command("{{trailing: comma |,| }}"),
            Some((
                Token::Command(
                    "trailing".to_string(),
                    vec!["comma".to_string(), String::new()]
                ),
                24
            ))
        );
        assert_eq!(parse_command("{{{}}{}{}}"), None);
        assert_eq!(parse_command("{print : here}"), None);
        assert_eq!(parse_command(" {{ gotta : start |,| with |,| it }}"), None);
        assert_eq!(
            parse_command("{{ must_be : whole |,| line}} fdasfkdlsafjd;slkfjdlas;"),
            None
        );
        assert_eq!(
            parse_command("{{ command : is |,| entire |,| line }}\n"),
            Some((
                Token::Command(
                    "command".to_string(),
                    vec!["is".to_string(), "entire".to_string(), "line".to_string()]
                ),
                39
            ))
        );
        assert_eq!(
            parse_command("{{ space_at_end_is_fine : ok}}"),
            Some((
                Token::Command("space_at_end_is_fine".to_string(), vec!["ok".to_string()]),
                30
            ))
        );
        assert_eq!(parse_command("{{ : }}"), None);
        assert_eq!(
            parse_command("{{ empty_arg : }}"),
            Some((
                Token::Command("empty_arg".to_string(), vec!["".to_string()]),
                17
            ))
        );
        assert_eq!(
            parse_command("{{ no_arg :}}"),
            Some((Token::Command("no_arg".to_string(), vec![]), 13))
        );
    }
    #[test]
    fn test_variable_parsing() {
        assert_eq!(parse_variable("fail"), None);
        assert_eq!(parse_variable("$almost$"), None);
        assert_eq!(
            parse_variable("${{finally}}"),
            Some((Token::Variable("finally".to_string()), 12))
        );
        assert_eq!(parse_variable("${{no spaces}}"), None);
        assert_eq!(
            parse_variable("${{_=?!2#232}} huh?"),
            Some((Token::Variable("_=?!2#232".to_string()), 14))
        );
    }
    #[test]
    fn test_pageend_parsing() {
        assert_eq!(parse_pageend("/PAGE/"), Some((Token::PageEnd, 6)));
        assert_eq!(parse_pageend("/page/"), None);
        assert_eq!(parse_pageend("fail"), None);
        assert_eq!(
            parse_pageend("/PAGE/ other stuff is allowed"),
            Some((Token::PageEnd, 6))
        );
    }
    #[test]
    fn test_char_parsing() {
        assert_eq!(parse_char("fail"), None);
        assert_eq!(parse_char("{c}"), Some((Token::Char('c'), 3)));
        assert_eq!(parse_char("{.}"), Some((Token::Char('.'), 3)));
        assert_eq!(parse_char("{too_long}"), None);
        assert_eq!(parse_char("{{}"), Some((Token::Char('{'), 3)));
        assert_eq!(parse_char("{}}"), Some((Token::Char('}'), 3)));
        assert_eq!(parse_char("{é}"), Some((Token::Char('é'), 4)));
        assert_eq!(parse_char("must be at beginning {n}"), None);
    }
    #[test]
    fn test_sectstart_parsing() {
        assert_eq!(parse_sect_start("fail"), None);
        assert_eq!(
            parse_sect_start("#=$ secret section $=#"),
            Some((Token::SectionStart("secret section".to_owned()), 22))
        );
        assert_eq!(
            parse_sect_start("#=$  no_trim   $=#"),
            Some((Token::SectionStart(" no_trim  ".to_owned()), 18))
        );
        assert_eq!(parse_sect_start("#=$Need space$=#"), None);
        assert_eq!(
            parse_sect_start("#=$ Need to be whole line $=# blah blah extra"),
            None
        );
        assert_eq!(
            parse_sect_start("#=$ special chars -?.#&?$ in middle are fine $=#"),
            Some((
                Token::SectionStart("special chars -?.#&?$ in middle are fine".to_owned()),
                48
            ))
        );
    }
}
