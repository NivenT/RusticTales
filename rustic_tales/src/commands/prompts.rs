use std::io::{stdin, stdout, Read, Write};
use std::os::unix::io::AsRawFd;
use std::time::{Duration, Instant};

use crate::ansi::TermAction;
use crate::err::{RTError, Result};
use crate::utils::menu;

pub fn prompt_yesno(def: Option<String>) -> String {
    print!(" (y/n) ");
    let _ = stdout().flush();
    let mut temp = String::new();
    let _ = stdin().read_line(&mut temp);
    match temp.trim().to_lowercase().as_ref() {
        "yes" | "y" | "sure" | "yeah" | "ok" | "k" | "yup" | "yy" => "y".to_owned(),
        "no" | "n" | "nah" | "no thanks" | "nope" | "nn" => "n".to_owned(),
        _ => def.unwrap_or_else(|| "n".to_string()),
    }
}

// This function could probably use some comments/documentation
pub fn force_input(input: &str) -> Result<()> {
    use termios::*;
    const SLOW_ERASE_THRESHOLD: Duration = Duration::from_millis(1000);
    const FAST_ERASE_THRESHOLD: Duration = Duration::from_millis(600);

    let is_alpha_num = |s: &str| {
        s.chars()
            .all(|c| char::is_ascii_alphanumeric(&c) || char::is_ascii_punctuation(&c) || c == ' ')
    };

    if !is_alpha_num(input) {
        let msg = format!(
            "'force_input' can only force inputs which are alphanumeric. '{}' is not alphanumeric",
            input
        );
        return Err(RTError::InvalidInput(msg));
    }

    let stdin_fd = stdin().as_raw_fd();
    let orig_termios = Termios::from_fd(stdin_fd)?;

    let mut new_termios = orig_termios;
    new_termios.c_lflag &= !(ICANON | ECHO);
    new_termios.c_cc[VMIN] = 0;
    new_termios.c_cc[VTIME] = 0;
    tcsetattr(stdin_fd, TCSANOW, &new_termios)?;

    let mut user_str = String::new();
    let mut last_erase_time = Instant::now();
    let mut erase_threshold = SLOW_ERASE_THRESHOLD;
    while user_str != input {
        let now = Instant::now();
        if now.duration_since(last_erase_time) > erase_threshold
            && !user_str.is_empty()
            && !input.starts_with(&user_str)
        {
            last_erase_time = now;
            erase_threshold = FAST_ERASE_THRESHOLD;

            TermAction::EraseCharsOnLine(1).execute();
            user_str.pop();
        } else if input.starts_with(&user_str) {
            last_erase_time = now;
        }

        let new_stuff = String::from_utf8(stdin().bytes().filter_map(|res| res.ok()).collect())
            .map_err(|_| {
                RTError::InvalidInput(
                    "Could not understand key strokes for some reasone".to_owned(),
                )
            })?;
        if is_alpha_num(&new_stuff) {
            erase_threshold = SLOW_ERASE_THRESHOLD;

            user_str += &new_stuff;
            print!("{}", new_stuff);
            let _ = stdout().flush();
        }
    }
    std::thread::sleep(Duration::from_millis(350));
    tcsetattr(stdin_fd, TCSANOW, &orig_termios)?;

    Ok(())
}

pub fn choice_menu(choices: &[impl AsRef<str>]) -> Result<String> {
    println!();
    let mut choice = menu(choices, None, false);
    while choice.is_err() {
        TermAction::EraseLines(choices.len() + 2).execute();
        choice = menu(choices, None, false);
    }
    TermAction::EraseLines(choices.len() + 2).execute();
    Ok(choices[choice.unwrap()].as_ref().to_owned())
}
