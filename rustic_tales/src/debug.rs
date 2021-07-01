use std::io::Write;

use crate::buffer::*;
use crate::err::Result;
use crate::options::Options;
use crate::storyteller::story::{Line, Page};
use crate::storyteller::{Debug, StoryTeller};
use crate::utils::*;

pub fn debug_menu(opts: &Options) -> Result<bool> {
    let mut should_wait = true;
    let debug_fns = [tokenize_story, parse_story, get_pagination_info];
    match menu(
        &[
            "Tokenize Story",
            "Separate Story into Units",
            "Pagination Info for Story",
            "Print Some Constants",
            "Test the buffer stuff",
        ],
        None,
        true,
    ) {
        Err(e) => println!("Something went wrong: '{}'", e),
        Ok(n) if (0..debug_fns.len()).contains(&n) => {
            match choose_story(opts.get_ignored(), opts.get_story_folder()) {
                Ok(story) => match StoryTeller::new(&story) {
                    Ok(st) => {
                        should_wait = false;
                        debug_fns[n](story, st)
                    }

                    Err(e) => println!("Could not parse story because '{}'", e),
                },
                Err(e) => println!("Something went wrong: '{}'", e),
            }
        }
        Ok(3) => print_some_constants(opts),
        Ok(4) => run_buffer_tests(),
        Ok(_) => unreachable!("Menu only returns valid choices"),
    }
    Ok(!should_wait)
}

fn tokenize_story(story: String, _teller: StoryTeller<Debug>) {
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

fn parse_story(_story: String, teller: StoryTeller<Debug>) {
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

fn get_pagination_info(story: String, teller: StoryTeller<Debug>) {
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

fn print_some_constants(opts: &Options) {
    println!("max_line_length: {}", Line::max_line_len());
    println!("max_page_height: {}", Page::max_page_height());
    println!("Options: {:?}", opts);
}

fn run_buffer_tests() {
    fn print_and_wait(buf: &mut TermBuffer) {
        clear_screen();
        print!("{}", buf);
        wait_for_kb_with_prompt(">");
    }

    let mut buf = TermBuffer::default();
    buf.resize();

    buf.write_text("Test text. Just making sure the basics work...\n");
    buf.add_fg_color(Color::light(BaseColor::Blue));
    buf.write_text("Blue text. Fancy, huh?\n");
    buf.add_text_effect(TextEffect::Inverse);
    buf.write_text("More text (but now inverted)\n");
    buf.undo_modifiers();
    buf.write_text("Normal text and then ");
    buf.add_bg_color(Color::dark(BaseColor::Red));
    buf.add_text_effect(TextEffect::Blink);
    buf.add_text_effect(TextEffect::Bold);
    buf.write_text("blinking bold text on a red background.\n\n");
    buf.undo_modifiers();

    buf.write_text("Let's now test some other stuff\n");
    buf.write_text("For example, we can delete a ton of text");
    print_and_wait(&mut buf);
    buf.erase_chars(50);
    print_and_wait(&mut buf);

    println!();
}
