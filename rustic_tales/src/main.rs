extern crate script;

mod err;

use std::io::stdin;
use std::{env, fs};

use err::{RTError, Result};

fn menu(items: Vec<&str>) -> Result<usize> {
    for (idx, item) in items.iter().enumerate() {
        println!("{}. {}", idx + 1, item);
    }

    let mut choice = String::new();
    let _ = stdin().read_line(&mut choice)?;
    let choice: usize = choice
        .trim()
        .parse()
        .map_err(|e| RTError::InvalidInput(format!("Could not parse choice because: {}", e)))?;

    if choice == 0 || choice > items.len() {
        Err(RTError::InvalidInput(format!(
            "Need to make a choice in range 1 -- {}",
            items.len()
        )))
    } else {
        Ok(choice - 1)
    }
}

fn choose_story() -> Result<String> {
    let mut dir = env::current_dir()?;
    dir.push("stories");

    let stories: Vec<String> = fs::read_dir(dir)?
        .filter(|e| e.is_ok())
        .map(|e| e.unwrap().file_name().into_string())
        .filter(|s| s.is_ok())
        .map(|s| s.unwrap())
        .collect();
    // I should just make menu take Vec<String>, but meh
    let refs = stories.iter().map(|s| &s[..]).collect::<Vec<_>>();
    Ok(stories[menu(refs)?].to_string())
}

fn main() {
    loop {
        match menu(vec!["Tell me a story", "Goodbye"]) {
            Err(e) => println!(
                "I did not understand your choice.\n{}\nPlease try again.",
                e
            ),
            Ok(0) => match choose_story() {
                Ok(story) => {}
                Err(e) => println!("I could not understand your choic\n{}", e),
            },
            Ok(_) => break,
        }
    }
}
