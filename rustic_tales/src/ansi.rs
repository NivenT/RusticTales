use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TermAction {
    EraseLine,
    EraseLineFromCursor,
    EraseLineToCursor,
    MoveCursor(isize, isize),
    SetCursor(usize, usize),
    ClearScreen,
    ResetColor,
}

impl fmt::Display for TermAction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TermAction::*;
        match self {
            EraseLine => write!(f, "\x1b[2K"),
            EraseLineFromCursor => write!(f, "\x1b[0K"),
            EraseLineToCursor => write!(f, "\x1b[1K"),
            MoveCursor(x, y) => {
                if *x != 0 {
                    let fb = if *x > 0 { 'C' } else { 'D' };
                    write!(f, "\x1b[{}{}", x.abs(), fb)?;
                }
                if *y != 0 {
                    let ud = if *y > 0 { 'A' } else { 'B' };
                    write!(f, "\x1b[{}{}", y.abs(), ud)?;
                }
                Ok(())
            }
            SetCursor(x, y) => {
                write!(f, "\x1b[H")?;
                if *x != 0 {
                    write!(f, "\x1b[{}C", x)?;
                }
                if *y != 0 {
                    write!(f, "\x1b[{}B", y)?;
                }
                Ok(())
            }
            ClearScreen => write!(f, "\x1b[2J"),
            ResetColor => write!(f, "\x1b[39m\x1b[49m"),
        }
    }
}

impl TermAction {
    pub fn execute(&self) {
        print!("{}", self)
    }
    pub fn then(&self, next: TermAction) -> TermActions {
        TermActions::Nil.then(*self).then(next)
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
    pub fn then(self, next: TermAction) -> Self {
        TermActions::Cons(Box::new(self), next)
    }
}
