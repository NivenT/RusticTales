use crate::err::Result;
use crate::options::Options;
use crate::storyteller::StoryTeller;
use crate::utils::*;

pub fn debug_menu(opts: &Options) -> Result<()> {
    let debug_fns = [tokenize_story, parse_story];
    match menu(&["Tokenize Story", "Parse Story"], None) {
        Err(e) => println!("Something went wrong: '{}'", e),
        Ok(n) if (0..=1).contains(&n) => {
            match choose_story(opts.get_ignored(), opts.get_story_folder()) {
                Ok(story) => match StoryTeller::new(&story) {
                    Ok(st) => debug_fns[n](story, st),
                    Err(e) => println!("Could not parse story because '{}'", e),
                },
                Err(e) => println!("Something went wrong: '{}'", e),
            }
        }
        Ok(_) => unreachable!("Menu only returns valid choices"),
    }
    Ok(())
}

fn tokenize_story(story: String, _teller: StoryTeller) {
    let tkns = StoryTeller::get_tokens(&story).expect(
        "It's already been tokenized once. If this would fail, it would have failed earlier",
    );
    let (_, h) = terminal_dims();
    let chunk_size = (h - 2) as usize;
    println!("Each token will be given it's own line. There will be {} tokens displayed at a time. When you are ready, press any key to see the next batch of tokens. At the end, press enter when the list has been exhausted.", chunk_size);
    for chunk in tkns.chunks(chunk_size) {
        wait_for_kb();
        clear_screen();
        for tkn in chunk {
            println!("{:?}", tkn);
        }
    }
    wait_for_enter("That's all... (hit enter)");
}

fn parse_story(_story: String, teller: StoryTeller) {
    let chunk_size = terminal_dims().1 as usize - 2;
    let story = teller.get_story();
    let contents = story.get_contents();

    println!("The parse story will be displayed as a series of 'Units'. Each one will get it's own line, with {} Units diaplayed at a time. When you are read, press any key to see the next batch of Units. At the end, press enter when the list has been exhausted.", chunk_size);

    for chunk in contents.chunks(chunk_size) {
        wait_for_kb();
        clear_screen();
        for unit in chunk {
            println!("{:?}", unit);
        }
    }
    wait_for_enter("That's all... (hit enter)");
}
