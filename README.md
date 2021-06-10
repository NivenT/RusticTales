# RusticTales
Interactive stories in your terminal.

More details on what this is coming soon, whenever I feel like typing them up.

# Script

The stories are written in a custom language, called 'script' (without the apostrophes). The philosphy of this language is maybe something like "simplicity over expressiveness, and also over aesthetics". It does not look particularly clean, and is somewhat constrained in what you can do with it, but it's not that complicated, so easy to get something working with it. Here are the things I try to keep in mind when deciding on what features to add...
* If you were to [download a random project gutenberg book and just give it to this program, I want to whole thing to parse as just normal text without any unintentional special effects](https://github.com/NivenT/RusticTales/blob/master/script/src/lib.rs#L75). As a consequence, the syntax of the language has to strange enough to not happen to appear in an ordinary book.
* Similarly, if you [look at one of these stories](https://github.com/NivenT/RusticTales/tree/master/rustic_tales/stories), it should be easy to tell where something out of the ordinary is happening. I want the effects to really stick out in the source.
* To try and keep complexity creep at bay, I'm trying to prefer specific capabilities over general ones. For instance, a story may want to do some form of branching (e.g. if it's a choose your own adventure or if it's ending depends on the time of day or whatever). To keep things simple, you can't jump to any point you want to, or based on any condition you want. There are a certain amount of built-in jump commands without only allow conditions of certain forms (e.g. `x = y`) and only let you "jump" to the start of (an expliclty marked) section or to another file.
  * secretly this isn't implimented yet (or maybe it is? See the TODO)

## Syntax

Maybe I'll add something later... For now, just look at the tests in the [script folder](https://github.com/NivenT/RusticTales/tree/master/script), and maybe at the definition of the [Token enum](https://github.com/NivenT/RusticTales/blob/master/script/src/token.rs). There's not much there.

## Commands

Again, I'll type up something more helpful when I feel like it. For now, see the [commands folder](https://github.com/NivenT/RusticTales/tree/master/rustic_tales/src/commands), and maybe also the relevant function in [storyteller_states.rs](https://github.com/NivenT/RusticTales/tree/master/rustic_tales/src/storyteller/storyteller_states.rs).

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
