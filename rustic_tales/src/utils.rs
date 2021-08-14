use std::io::{stdin, Read, Write};
use std::os::unix::io::AsRawFd;
use std::process::Command;
use std::{env, fs};

use terminal_size::{terminal_size, Height, Width};

use globset::{Glob, GlobSetBuilder};

use crate::ansi::TermAction;
use crate::err::{RTError, Result};

pub fn wait_for_enter(prompt: &str) {
    print!("{}", prompt);
    let _ = std::io::stdout().flush();
    let mut temp = String::new();
    let _ = std::io::stdin().read_line(&mut temp);
}

pub fn terminal_dims() -> (u16, u16) {
    if let Some((Width(w), Height(h))) = terminal_size() {
        (w, h)
    } else {
        (80, 25)
    }
}

pub fn no_term_echo() -> Option<termios::Termios> {
    use termios::*;
    let stdin_fd = stdin().as_raw_fd();
    let orig_termios = Termios::from_fd(stdin_fd).ok()?;

    let mut new_termios = orig_termios;
    new_termios.c_lflag &= !(ICANON | ECHO);
    tcsetattr(stdin_fd, TCSANOW, &new_termios).ok()?;
    Some(orig_termios)
}

pub fn restore_term(settings: Option<termios::Termios>) {
    use termios::*;
    if let Some(termios) = settings {
        let stdin_fd = stdin().as_raw_fd();
        let _ = tcsetattr(stdin_fd, TCSANOW, &termios);
    }
}

// This works on unix-like systems only
pub fn get_kb() -> Option<u8> {
    use termios::*;
    let stdin_fd = stdin().as_raw_fd();
    let orig_termios = Termios::from_fd(stdin_fd).ok()?;

    let mut new_termios = orig_termios;
    new_termios.c_lflag &= !(ICANON | ECHO);
    new_termios.c_cc[VMIN] = 0;
    new_termios.c_cc[VTIME] = 0;
    tcsetattr(stdin_fd, TCSANOW, &new_termios).ok()?;
    let res = stdin().bytes().next().and_then(|res| res.ok());
    tcsetattr(stdin_fd, TCSANOW, &orig_termios).ok()?;
    res
}

// Don't tell anyone I wrote a spinlock, ok?
pub fn wait_for_kb() {
    while get_kb() == None {}
}

pub fn wait_for_kb_with_prompt(prompt: &str) {
    print!("{}", prompt);
    let _ = std::io::stdout().flush();
    wait_for_kb();
}

pub fn exhaust_kb() {
    while get_kb() != None {}
}

pub fn get_user() -> Option<String> {
    Command::new("sh")
        .arg("-c")
        .arg("id -un")
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|name| name.trim().to_owned())
}

pub fn clear_screen() {
    TermAction::ClearScreen
        .then(TermAction::SetCursor(0, 0))
        .then(TermAction::ResetColor)
        .execute_raw();
}

pub fn menu(
    items: &[impl AsRef<str>],
    ignore_patterns: Option<&[String]>,
    clear: bool,
) -> Result<usize> {
    if clear {
        clear_screen();
    }
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
        .filter(|(_, item)| {
            let s = item.as_ref();
            globs.as_ref().map_or(true, |gs| !gs.is_match(s))
        })
        .enumerate()
    {
        println!("{}. {}", num + 1, item.as_ref());
        true_indices.push(idx);
    }
    println!();

    let mut choice = String::new();
    let _ = std::io::stdin().read_line(&mut choice)?;
    let choice: usize = choice.trim().parse()?;
    let max_choice = true_indices.len();

    if choice == 0 || choice > max_choice {
        let err_msg = format!("Need to make a choice in range 1 -- {}", max_choice);
        Err(RTError::InvalidInput(err_msg))
    } else {
        Ok(true_indices[choice - 1])
    }
}

pub fn choose_story(ignore_patterns: &[String], folder: &str) -> Result<String> {
    let mut dir = env::current_dir()?;
    dir.push(folder);

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
    let file_name = &stories[menu(&stories, Some(ignore_patterns), true)?];
    Ok(format!("{}/{}", folder, file_name))
}
