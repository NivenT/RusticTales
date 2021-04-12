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

use options::Options;
use storyteller::StoryTeller;
use utils::{choose_story, menu};

fn main() {
    let test = Options::from_file("test.ron");
    println!("{:?}", test);
    return;

    loop {
        match menu(vec!["Tell me a story", "Goodbye"]) {
            Err(e) => println!(
                "I did not understand your choice.\n{}\nPlease try again.\n",
                e
            ),
            Ok(0) => match choose_story() {
                Ok(story) => {
                    let mut st = StoryTeller::new(&story)
                        .expect("choose_story should only return existing files");
                    st.tell();
                }
                Err(e) => println!("I could not understand your choice\n{}", e),
            },
            Ok(_) => break,
        }
    }
}
