extern crate globset;
extern crate image;
extern crate regex;
extern crate ron;
extern crate script;
extern crate serde;
extern crate terminal_size;

mod ansi;
mod commands;
mod err;
mod options;
mod storyteller;
mod utils;

use err::Result;
use options::Options;
use storyteller::StoryTeller;
use utils::{choose_story, menu};

fn main() -> Result<()> {
    let options = match Options::from_file("options.ron") {
        Ok(opts) => opts,
        Err(_) => {
            let temp = Options::default();
            temp.to_file("options.ron")?;
            temp
        }
    };
    loop {
        match menu(vec!["Tell me a story", "Goodbye"], None) {
            Err(e) => println!(
                "I did not understand your choice.\n{}\nPlease try again.\n",
                e
            ),
            Ok(0) => match choose_story(options.get_ignored()) {
                Ok(story) => {
                    let mut st = StoryTeller::new(&story)
                        .expect("choose_story should only return existing files");
                    st.tell();
                }
                Err(e) => println!("I could not understand your choice\n{}", e),
            },
            Ok(n) => {
                println!("{}", n);
                break;
            }
        }
    }
    println!("G'day");
    Ok(())
}
