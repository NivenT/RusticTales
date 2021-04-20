use std::io::stdin;
use std::{env, fs};

use globset::{Glob, GlobSetBuilder};

use crate::ansi::TermAction;
use crate::err::{RTError, Result};

pub fn wait_for_enter(prompt: &str) {
    println!("{}", prompt);
    let mut temp = String::new();
    let _ = std::io::stdin().read_line(&mut temp);
}

pub fn clear_screen() {
    TermAction::ClearScreen
        .then(TermAction::SetCursor(0, 0))
        .execute();
}

pub fn menu(items: Vec<&str>, ignore_patterns: Option<&Vec<String>>) -> Result<usize> {
    clear_screen();
    let globs = ignore_patterns.and_then(|patts| {
        patts
            .iter()
            .fold(GlobSetBuilder::new(), |mut builder, pat| {
                if let Ok(glob) = Glob::new(pat) {
                    builder.add(glob);
                }
                builder
            })
            .build()
            .ok()
    });

    // This wastes some space, but I expect items.len() < 20 in practice, so who cares?
    let mut true_indices = Vec::with_capacity(items.len());
    for (num, (idx, item)) in items
        .iter()
        .enumerate()
        .filter(|(_, item)| globs.as_ref().map_or(true, |gs| !gs.is_match(item)))
        .enumerate()
    {
        println!("{}. {}", num + 1, item);
        true_indices.push(idx);
    }
    println!();

    let mut choice = String::new();
    let _ = stdin().read_line(&mut choice)?;
    let choice: usize = choice.trim().parse()?;
    let max_choice = true_indices.len();

    if choice == 0 || choice > max_choice {
        Err(RTError::InvalidInput(format!(
            "Need to make a choice in range 1 -- {}",
            max_choice
        )))
    } else {
        Ok(true_indices[choice - 1])
    }
}

pub fn choose_story(ignore_patterns: &Vec<String>) -> Result<String> {
    let mut dir = env::current_dir()?;
    dir.push("stories");

    let stories: Vec<String> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type()
                .ok()
                .filter(|file_type| file_type.is_file())
                .is_some()
        })
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    // I should just make menu take Vec<String>, but meh
    let refs = stories.iter().map(|s| &s[..]).collect::<Vec<_>>();
    let file_name = &stories[menu(refs, Some(ignore_patterns))?];

    Ok(format!("stories/{}", file_name))
}
