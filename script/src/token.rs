use regex::Regex;
//use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Token {
    Text(String),
    Command(String, Vec<String>),
    Variable(String),
    Symbol(String),
}

fn parse_symbol(stream: &str) -> Option<(Token, usize)> {
    let re =
        Regex::new(r"^\$([^[[:space:]]]+)\$").expect("If this regex is invalid, that is a bug");
    if let Some(cap) = re.captures(stream) {
        Some((Token::Symbol(cap[1].to_string()), cap[0].len()))
    } else {
        None
    }
}

// A command should take up an entire line
fn parse_command(stream: &str) -> Option<(Token, usize)> {
    // regex are completely incomprehensible (it doesn't help that I suck at writing them)
    let re = Regex::new(r"^\{\{[[:space:]]*(\b\w+\b)[[:space:]]*:(.*)\}\}[[:space:]]*(\n|$)")
        .expect("If this is invalid, there is a bug");
    if let Some(cap) = re.captures(stream) {
        let name = cap[1].to_string();
        let arg_list = &cap[2];
        let arg_list = if arg_list.len() == 0 {
            vec![]
        } else {
            arg_list
                .split("|,|")
                .map(|s| s.trim().to_string())
                .collect()
        };

        Some((Token::Command(name, arg_list), cap[0].len()))
    } else {
        None
    }
}

fn parse_variable(stream: &str) -> Option<(Token, usize)> {
    let re = Regex::new(r"^\$\{\{([^[[:space:]]]+)\}\}").expect("If bad, then bug");
    if let Some(cap) = re.captures(stream) {
        Some((Token::Variable(cap[1].to_string()), cap[0].len()))
    } else {
        None
    }
}

pub(crate) fn tokenize(stream: &str) -> Vec<Token> {
    let mut ret = vec![];

    let mut beg = 0;
    let mut search_pos = beg;
    while beg < stream.len() {
        let special_chars: &[char] = &['{', '$'];
        if let Some(end) = stream[search_pos..].find(special_chars) {
            search_pos += end;
            // at most one of these can be Some
            let sym = parse_symbol(&stream[search_pos..]);
            let cmd = parse_command(&stream[search_pos..]);
            let var = parse_variable(&stream[search_pos..]);
            if let Some((tkn, len)) = sym.or(cmd).or(var) {
                ret.push(Token::Text(stream[beg..search_pos].to_string()));
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

    ret
}

/*
// Maybe move this somewheer else (and change Vec<Token>) later on
impl FromStr for Vec<Token> {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(tokenize(s))
    }
}
*/

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
}
