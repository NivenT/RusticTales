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

    #[test]
    fn tokenization_with_cash_sign() {
        let test = "\
          Your total comes out to ${{BLUE_DBG}} $5.98. Is that ok?\n\
          {{ user_input : $response$}}\n\
          You answered '${{response}}'.\
        ";
        assert_eq!(
            tokenize(&test),
            vec![
                Token::Text("Your total comes out to ".to_string()),
                Token::Variable("BLUE_DBG".to_string()),
                Token::Text(" $5.98. Is that ok?\n".to_string()),
                Token::Command("user_input".to_string(), vec!["$response$".to_string()]),
                Token::Text("You answered '".to_string()),
                Token::Variable("response".to_string()),
                Token::Text("'.".to_string())
            ]
        );
    }
}
