use super::storyteller::DisplayUnit;
use crate::ansi::TermAction;

pub fn backspace(len: isize, unit: DisplayUnit) {
    if unit.is_char() {
        /*
        for _ in 0..len {
            print!("\u{8}")
        }
         */
        TermAction::MoveCursor(-len, 0)
            .and_then(TermAction::EraseLineFromCursor)
            .execute()
    } else {
        unimplemented!()
    }
}
