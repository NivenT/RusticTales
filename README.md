# RusticTales
Interactive stories in your terminal.

More details on what this is coming soon, whenever I feel like typing them up.

# How to Build

First clone this repo, and then build it.
```bash
git clone https://github.com/NivenT/RusticTales
cd RusticTales
cargo build
```

# How to Run

Just run it (from the root directory of this project)
```bash
cargo run
```

Later, when I feel like it, I'll add instructions for changing the options. For now, just know that you can do this by editing the [options.ron](https://github.com/NivenT/RusticTales/blob/master/options.ron) file in the folder from which you `cargo run`. The [options.rs](https://github.com/NivenT/RusticTales/blob/master/rustic_tales/src/options.rs) file determines what values the various options can take. Of note, you can change `scroll_rate` to have the program scroll automatically (using e.g. `Millis(num: 5, ms: 700)` to display 5 units (words or characters as determined by `disp_by`) every 700 milliseconds) or to have it scroll manually (i.e. display so many words or lines or a single page every time you press a button, e.g. with `Lines(4)`, `Words(10)` or `OnePage`).

# Using this on Windows

This code won't run natively on Windows, but luckily it doesn't need to because Windows users can use the [Windows Subsystem for Linux](https://docs.microsoft.com/en-us/windows/wsl/install-win10#manual-installation-steps). If you want to run this on Windows, follow the instructions in that link (I reccommend the manual install). **You need to be running WSL 2 for this program to work**. If you run into issues getting this working (or run into other issues), scroll down a little.

At the end, you'll need to pick a Linux distro to use. The instructions below assume you picked [Ubuntu](https://www.microsoft.com/en-us/p/ubuntu/9nblggh4msv6?activetab=pivot:overviewtab). Once you have an Ubuntu terminal up and running, you'll want to install rust before cloning this repo. To do this, run the following commands. 
```bash
sudo apt update
sudo apt install build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Now you can `git clone` and do whatever else it says under the 'How to Build' header.

## Possible Issues getting WSL working
* `wsl --set-default-version 2` not working? (e.g. it just display help information)  
  * The likely cause here is you need to update Windows. You can check your version by `WinKey+R` and then typing `winver`. This'll give you your version number in the form [MAJOR NUMBER].[MINOR NUMBER] in order to use WSL 2, you need MAJOR_NUMBER >= 18362 *and* MINOR_NUMBER >= 1049 (e.g. if you're on version 18363.752, then your MINOR_NUMBER is too low). [Step 2 of the manual WSL instructions](https://docs.microsoft.com/en-us/windows/wsl/install-win10#manual-installation-steps) mention ways to update your Windows version. If you ask me, the easiest thing to do would be to get the relevant update straight from the [Microsoft Update Catalog](https://www.catalog.update.microsoft.com/Search.aspx?q=KB4566116). In any case, after updating Windows, you should be able to get WSL 2 without any trouble.
* Getting a dialog box saying something like `Update only applies to machines with WSL`?
  * Restart your computer.  
* The `curl` command not working?
  * The likely cause here is that my instructions are out of date. To get Rust, do whatever it says  [here](https://www.rust-lang.org/learn/get-started). You'll still want to get separately `build-essential` though.

# Script

The stories are written in a custom language, called 'Script' (without the apostrophes). The philosphy of this language is maybe something like "simplicity over expressiveness, and also over aesthetics". It does not look particularly clean, and is somewhat constrained in what you can do with it. On the bright side, it's not that complicated, so it's easy to get something working. Here are the things I try to keep in mind when deciding on what features to add...
* If you were to [download a random project gutenberg book and just give it to this program, I want to whole thing to parse as just normal text without any unintentional special effects](https://github.com/NivenT/RusticTales/blob/master/script/src/lib.rs#L75). As a consequence, the syntax of the language has to be strange enough for no special tokens to accidentally appear in an ordinary book.
* Similarly, if you [look at one of these stories](https://github.com/NivenT/RusticTales/tree/master/rustic_tales/stories), I want any sort of special token/language feature to really pop out. It should be easy to tell what's ordinary text and what's not.
* To try and keep complexity creep at bay, I'm trying to prefer specific capabilities over general ones. For instance, a story may want to do some form of branching (e.g. if it's a choose your own adventure or if it's ending depends on the time of day or whatever). To keep things simple, you can't do arbitrary branching to any point in the story based on any conditions. There are a certain number of built-in jump commands which only allow conditions of certain forms (e.g. `x = y`) and only let you "jump" to the start of (an expliclty marked) section or to another file.
  * secretly this isn't implimented yet (or maybe it is? See the TODO)

## Syntax

Maybe I'll add something later... For now, just look at the tests in the [script/src folder](https://github.com/NivenT/RusticTales/tree/master/script/src), and maybe at the definition of the [Token enum](https://github.com/NivenT/RusticTales/blob/master/script/src/token.rs). Also, you can see examples in the [stories folder](https://github.com/NivenT/RusticTales/tree/master/rustic_tales/stories).

## Commands

Again, I'll type up something more helpful when I feel like it. For now, see the [commands folder](https://github.com/NivenT/RusticTales/tree/master/rustic_tales/src/commands), and also the relevant function in [storyteller_states.rs](https://github.com/NivenT/RusticTales/blob/master/rustic_tales/src/storyteller/storyteller_states.rs#L251). Actually, it's probably best just to look at the stories folder and see which commands are used there.

# TODO (In no particular order)

- [X] Make a TODO List
- [X] Pagination
  - [X] Hit enter to go to next page
  - [X] Pagination takes into account newlines
  - [X] Other stuff... It's been too long since I worked on this. I don't remember what I need to do
- [X] Config file
  - [X] Story directory
  - [X] Word every x seconds vs. word on enter
  - [X] More
- [X] Figure out what the '...' should be
- [ ] Figure out a way to do branching
  - [ ] Stories across multiple files?
  - [X] Label sections?
- [X] Pagination again, but for sections
- [X] Add debug features?
- [ ] State machine
  - [X] backspace one character at a time
  - [ ] Other things
  - [X] Pause story (press `p` to pause/resume)
    - [ ] Indicate when story paused
  - [X] End story when pressed `Esc`
  - [X] End story when `q` is pressed (there are some places where `Esc` ends the story but `q` does not)
  - [ ] Move command implementations into various states so they can interop better with the rest of the program
    - [ ] e.g. should be able to pause/quit mid-command
    - [X] See e.g. how the `repeat` command is implemented. It's an annoying amount of work, but doing this for every command will make for a better program.
  - [ ] Somthing something proc macro?
- [ ] Internal story buffer thingy
  - [X] Don't just immedately print to terminal
  - [ ] Keep track of cursor position
    - [X] (Reliably) erase characters not on the current line
  - [ ] Dynamic pagination?
  - [X] Text wrapping (e.g. set max row length)
  - [ ] Make sure this thing actually words as intended
- [ ] Better naviagation
  - [ ] Move back a page
  - [ ] General purpose undo?
- [ ] Write stories
  - [ ] Add features to Script?
    - [ ] story markers?
  - [ ] Think of a creative use of the terminal?
  - [ ] Abandon this project before getting anything worth making public?
  - [ ] Put off doing this until the very end of time?
- [ ] Windows support
  - [ ] Wrap all terminal stuff in convient functions that work for either windows or unix
  - [X] Quasi-Windows support via WSL
- [ ] Write a decent README
  - [ ] Make the TODO list coherent
  - [X] Reticulate splines
  - [ ] Fix all the spelling/grammar mistakes
- [ ] Clean up code
  - [ ] Somehow reduce the amount of logic duplication in this codebase
  - [ ] Remove old code that's no longer needed
    - [ ] Get rid of ansi.rs?
  - [ ] Rewrite it Rust? I feel like this is suppose to fix anything

