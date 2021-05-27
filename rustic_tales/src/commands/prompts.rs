use std::io::{stdin, stdout, Write};

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
