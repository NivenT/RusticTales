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

Later, when I feel like it, I'll add instructions for changing the options. For now, just know that you can do this by editing the [options.ron](https://github.com/NivenT/RusticTales/blob/master/options.ron) file in the folder from which you `cargo run`. The [options.rs](https://github.com/NivenT/RusticTales/blob/master/rustic_tales/src/options.rs) file determines what values the various options can take. Of note, you can change `scroll_rate` to have the program scroll automatically (using e.g. `Millis(num: 5, ms: 700)` to display 5 units (words or characters as determined by `disp_by`) every 700 milliseconds) or to have it scroll manually (i.e. display so many words or lines or a single page every time you press a button).

# Using this on Windows

This code won't run natively on Windows, but luckily it doesn't need to because Windows users can use the [Windows Subsystem for Linux](https://docs.microsoft.com/en-us/windows/wsl/install-win10#manual-installation-steps). If you want to run this on Windows, follow the instructions in that link (I reccommend the manual install). 

At the end, you'll need to pick a Linux distro to use. The instructions below assume you picked [Ubuntu](https://www.microsoft.com/en-us/p/ubuntu/9nblggh4msv6?activetab=pivot:overviewtab). Once you have an Ubuntu terminal up and running, you'll want to install rust before cloning this repo. To do this, run the following commands. 
```bash
sudo apt update
sudo apt install build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
If the last command above doesn't work, do whatever it says [here](https://www.rust-lang.org/learn/get-started).

Now you can `git clone` and do whatever else it says under the 'How to Run' header.

# Script

The stories are written in a custom language, called 'script' (without the apostrophes). The philosphy of this language is maybe something like "simplicity over expressiveness, and also over aesthetics". It does not look particularly clean, and is somewhat constrained in what you can do with it, but it's not that complicated, so easy to get something working with it. Here are the things I try to keep in mind when deciding on what features to add...
* If you were to [download a random project gutenberg book and just give it to this program, I want to whole thing to parse as just normal text without any unintentional special effects](https://github.com/NivenT/RusticTales/blob/master/script/src/lib.rs#L75). As a consequence, the syntax of the language has to strange enough to not happen to appear in an ordinary book.
* Similarly, if you [look at one of these stories](https://github.com/NivenT/RusticTales/tree/master/rustic_tales/stories), it should be easy to tell where something out of the ordinary is happening. I want the effects to really stick out in the source.
* To try and keep complexity creep at bay, I'm trying to prefer specific capabilities over general ones. For instance, a story may want to do some form of branching (e.g. if it's a choose your own adventure or if it's ending depends on the time of day or whatever). To keep things simple, you can't jump to any point you want to, or based on any condition you want. There are a certain amount of built-in jump commands without only allow conditions of certain forms (e.g. `x = y`) and only let you "jump" to the start of (an expliclty marked) section or to another file.
  * secretly this isn't implimented yet (or maybe it is? See the TODO)

## Syntax

Maybe I'll add something later... For now, just look at the tests in the [script folder](https://github.com/NivenT/RusticTales/tree/master/script), and maybe at the definition of the [Token enum](https://github.com/NivenT/RusticTales/blob/master/script/src/token.rs). There's not much there.

## Commands

Again, I'll type up something more helpful when I feel like it. For now, see the [commands folder](https://github.com/NivenT/RusticTales/tree/master/rustic_tales/src/commands), and maybe also the relevant function in [storyteller_states.rs](https://github.com/NivenT/RusticTales/blob/master/rustic_tales/src/storyteller/storyteller_states.rs#L251). Actually, it's probably best just to look at the stories folder and see which commands are used there.

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
  - [ ] End story when pressed `Esc`
  - [X] End story when `q` is pressed
  - [ ] Move command implementations into various states so they can interop better with the rest of the program
    - [ ] e.g. should be able to pause/quit mid-command
- [ ] Internal story buffer thingy
  - [ ] Don't just immedately print to terminal
  - [ ] Keep track of cursor position
    - [ ] (Reliably) erase characters not on the current line
  - [ ] Text wrapping (e.g. set max row length)
- [ ] Write stories
  - [ ] Add features to script?
    - [ ] story markers
  - [ ] Think of a creative use of the terminal?
  - [ ] Abandon this project before getting anything worth making public?
  - [ ] Put off doing this until the very end of time?
- [ ] Windows support
  - [ ] Wrap all terminal stuff in convient functions that work for either windows or unix
- [ ] Write a decent README
