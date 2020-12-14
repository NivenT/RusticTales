extern crate regex;
mod token;

#[cfg(test)]
mod tests {
    use crate::token::{tokenize, Token};

    #[test]
    fn basic_tokenization() {
        let test = "\
          Once upon a time, there was a large and terrifying ${{RED_LFG}} credit card bill!\n\
          {{ backspace : 10 |,| chars |,| one_by_one }}\n\
          OVERDUE Credit Card Bill!\
        ";
        let tkns = tokenize(&test);
        assert_eq!(
            tkns,
            vec![
                Token::Text("Once upon a time, there was a large and terrifying ".to_string()),
                Token::Variable("RED_LFG".to_string()),
                Token::Text(" credit card bill!\n".to_string()),
                Token::Command(
                    "backspace".to_string(),
                    vec![
                        "10".to_string(),
                        "chars".to_string(),
                        "one_by_one".to_string()
                    ]
                ),
                Token::Text("OVERDUE Credit Card Bill!".to_string())
            ]
        )
    }
}
