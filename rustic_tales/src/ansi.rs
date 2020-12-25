use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TermAction {
    EraseLine,
    EraseLineFromCursor,
    EraseLineToCursor,
    MoveCursor(isize, isize),
    SetCursor(usize, usize),
    ClearScreen,
}

impl fmt::Display for TermAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TermAction::*;
        match self {
            EraseLine => write!(f, "\x1b[2K"),
            EraseLineFromCursor => write!(f, "\x1b[0K"),
            EraseLineToCursor => write!(f, "\x1b[1K"),
            MoveCursor(x, y) => {
                let fb = if *x > 0 { 'C' } else { 'D' };
                let ud = if *y > 0 { 'A' } else { 'B' };
                write!(f, "\x1b[{}{}\x1b[{}{}", x.abs(), fb, y.abs(), ud)
            }
            SetCursor(x, y) => write!(f, "\x1b[H\x1b[{}C\x1b[{}B", x, y),
            ClearScreen => write!(f, "\x1b[2J"),
        }
    }
}

impl TermAction {
    pub fn execute(&self) {
        print!("{}", self)
    }
    pub fn and_then(&self, then: TermAction) -> TermActions {
        TermActions::Nil.and_then(*self).and_then(then)
    }
}

#[derive(Clone, Debug)]
pub enum TermActions {
    Nil,
    Cons(Box<TermActions>, TermAction),
}

impl TermActions {
    pub fn execute(&self) {
        if let TermActions::Cons(pre, last) = self {
            pre.execute();
            last.execute();
        }
    }
    pub fn and_then(self, then: TermAction) -> Self {
        TermActions::Cons(Box::new(self), then)
    }
}
