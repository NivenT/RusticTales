extern crate regex;
pub mod token;

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

    #[test]
    fn tokenization_consecutive_variables() {
        let test = "${{VAR1}}${{VAR2}}Text${{VAR3}}Text${{VAR4}}${{VAR5}}${{VAR6}}";
        assert_eq!(
            tokenize(&test),
            vec![
                Token::Variable("VAR1".to_string()),
                Token::Variable("VAR2".to_string()),
                Token::Text("Text".to_string()),
                Token::Variable("VAR3".to_string()),
                Token::Text("Text".to_string()),
                Token::Variable("VAR4".to_string()),
                Token::Variable("VAR5".to_string()),
                Token::Variable("VAR6".to_string()),
            ]
        );
    }

    #[test]
    fn tokenize_gutenberg() -> reqwest::Result<()> {
        extern crate reqwest;

        const BOOKS: [&'static str; 5] = [
            // Soup and Soup Making
            "https://www.gutenberg.org/files/64140/64140-0.txt",
            // The Price of Things
            "https://dev.gutenberg.org/cache/epub/9809/pg9809.txt",
            // Kid Wolf of Texas
            "https://www.gutenberg.org/cache/epub/22057/pg22057.txt",
            // Odd
            "https://www.gutenberg.org/cache/epub/22291/pg22291.txt",
            // Death, the Knight, and the Lady
            "https://www.gutenberg.org/cache/epub/55708/pg55708.txt",
        ];
        for &book in &BOOKS {
            let test = reqwest::blocking::get(book)?.text()?;
            let tkns = tokenize(&test);
            assert!(tkns.len() == 1);
            assert!(tkns[0].is_text());
            if let Token::Text(s) = &tkns[0] {
                assert!(s.len() > 1000); // just to make sure we downloaded something
            }
        }
        Ok(())
    }

    #[test]
    fn tokenize_unicode() {
        let test = "\
          Il était une fois que j'ai oublié mon {.}{.}{.} portable.\n\
          Alor, j'ai dû emprunter un portable à mon ami. Malheureusement, il était anglais.\n\
          Donc, je n'ai pas su comment trouver le clé '${{BLUE_LFG}}{é}'. C'était triste.\
        ";
        assert_eq!(
            tokenize(&test),
            vec![
                Token::Text("Il était une fois que j'ai oublié mon ".to_string()),
                Token::Char('.'),
                Token::Char('.'),
                Token::Char('.'),
                Token::Text(" portable.\nAlor, j'ai dû emprunter un portable à mon ami. Malheureusement, il était anglais.\nDonc, je n'ai pas su comment trouver le clé '".to_string()),
                Token::Variable("BLUE_LFG".to_string()),
                Token::Char('é'),
                Token::Text("'. C'était triste.".to_string()),
            ]
        );
    }

    #[test]
    fn tokenize_pages() {
        let test = "Page 1/PAGE/ Page 2/PAGE/ 3 and then empty/PAGE//PAGE/Fin.";
        assert_eq!(
            tokenize(&test),
            vec![
                Token::Text("Page 1".to_string()),
                Token::PageEnd,
                Token::Text(" Page 2".to_string()),
                Token::PageEnd,
                Token::Text(" 3 and then empty".to_string()),
                Token::PageEnd,
                Token::PageEnd,
                Token::Text("Fin.".to_string()),
            ]
        );
    }
}
