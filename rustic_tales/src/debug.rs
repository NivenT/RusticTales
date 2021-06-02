use crate::err::Result;
use crate::options::Options;
use crate::storyteller::StoryTeller;
use crate::utils::*;

pub fn debug_menu(opts: &Options) -> Result<bool> {
    let mut all_according_to_plan = false;
    let debug_fns = [tokenize_story, parse_story, get_pagination_info];
    match menu(
        &[
            "Tokenize Story",
            "Separate Story into Units",
            "Pagination Info for Story",
        ],
        None,
        true,
    ) {
        Err(e) => println!("Something went wrong: '{}'", e),
        Ok(n) if (0..=2).contains(&n) => {
            match choose_story(opts.get_ignored(), opts.get_story_folder()) {
                Ok(story) => match StoryTeller::new(&story) {
                    Ok(st) => {
                        all_according_to_plan = true;
                        debug_fns[n](story, st)
                    }

                    Err(e) => println!("Could not parse story because '{}'", e),
                },
                Err(e) => println!("Something went wrong: '{}'", e),
            }
        }
        Ok(_) => unreachable!("Menu only returns valid choices"),
    }
    Ok(all_according_to_plan)
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

fn get_pagination_info(story: String, teller: StoryTeller) {
    let s = teller.get_story();
    let contents = s.get_contents();
    let sections = s.get_sections();

    println!("'{}' has {} section(s)", story, sections.len());
    println!("It has a total of {} unit(s)", contents.len());
    for (i, sect) in sections.iter().enumerate() {
        let pages = sect.get_pages();
        println!("SECTION {} ({})", i, sect.get_name());
        println!("* Starts at index {}", sect.start_idx().unwrap());
        println!("* There are {} page(s)", pages.len());
        for (j, page) in pages.iter().enumerate() {
            let lines = page.get_lines();
            println!("* PAGE {}", j);
            println!("* * Starts at index {}", page.start_idx().unwrap());
            println!("* * There {} line(s)", lines.len());
            for (k, line) in lines.iter().enumerate() {
                println!("* * Line {}", k);
                println!(
                    "* * * Starts at index {} (with {:?})",
                    line.get_start(),
                    contents[line.get_start()]
                );
                println!(
                    "* * * Ends with index {} (with {:?})",
                    line.get_end(),
                    contents[line.get_end()]
                );
            }
        }
    }
    wait_for_enter("Press enter to continue...");
}
