use std::collections::LinkedList;
use std::fmt;

use crate::utils::*;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum BaseColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Grey,
}

impl BaseColor {
    pub fn val(&self) -> u8 {
        30 + (*self as u8)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Color(BaseColor, bool); // true == light

impl Color {
    pub fn light(col: BaseColor) -> Color {
        Color(col, true)
    }
    pub fn dark(col: BaseColor) -> Color {
        Color(col, false)
    }
    pub fn val(&self) -> u8 {
        let base_val = self.0.val();
        if self.1 {
            base_val + 60
        } else {
            base_val
        }
    }
    pub fn is_light(&self) -> bool {
        self.1
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum TextEffect {
    None,
    Bold,
    Dim,
    Italic,
    Underline,
    Blink,
    AECSix, // ANSI escape code 6
    Inverse,
    AECEight,
    Strikethrough,
}

impl TextEffect {
    pub fn val(&self) -> u8 {
        *self as u8
    }
}

impl fmt::Display for TextEffect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "\x1b[{}m", self.val())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CellModifier {
    FGColor(Color),
    BGColor(Color),
    Effect(TextEffect),
}

impl fmt::Display for CellModifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CellModifier::*;
        match self {
            FGColor(col) => write!(f, "\x1b[{}m", col.val()),
            BGColor(col) => write!(f, "\x1b[{}m", col.val() + 10),
            Effect(eff) => write!(f, "{}", eff),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Cell {
    c: char, // Tried 'character' but that's just long
    modifiers: LinkedList<CellModifier>,
}

impl Cell {
    pub fn clear(&mut self) {
        self.c = '\0';
        self.modifiers.clear();
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for modifier in self.modifiers.iter() {
            write!(f, "{}", modifier)?
        }
        write!(f, "{}", self.c)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TermBuffer {
    cells: Vec<Cell>,
    rows: usize,
    cols: usize,
    curr_idx: usize,
}
// ^^^^^^^^^^^^^^^^^^^^^^^
// This should keep track of lines and stuff
// May need to replace Vec<Cell> w/ Vec<Line> and do other more complicated things

impl TermBuffer {
    pub fn resize(&mut self) {
        let (rows, cols) = terminal_dims();
        self.rows = rows as usize;
        self.cols = cols as usize;

        self.cells.resize(self.rows * self.cols, Cell::default());
    }
    pub fn set_cursor(&mut self, r: usize, c: usize) {
        self.curr_idx = c + r * self.cols;
    }
    pub fn move_cursor(&mut self, dr: isize, dc: isize) {
        let (r, c) = self.get_cursor();
        let (r, c) = (r as isize, c as isize);
        self.set_cursor((r + dr) as usize, (c + dc) as usize);
    }
    // (row, column)
    pub fn get_cursor(&self) -> (usize, usize) {
        (self.curr_idx / self.cols, self.curr_idx % self.cols)
    }
    pub fn write_char(&mut self, c: char) {
        self.get_curr_mut().c = c;
        self.advance_cursor();
    }
    pub fn advance_cursor(&mut self) {
        self.curr_idx += 1;
        /*
        if self.cursor.1 == self.cols - 1 {
            self.cursor.0 = (self.cursor.0 + 1) % self.rows;
        }
        self.cursor.1 = (self.cursor.1 + 1) % self.cols;
        */
    }
    pub fn write_text(&mut self, t: &str) {
        t.chars().for_each(|c| self.write_char(c))
    }
    pub fn add_modifier(&mut self, m: CellModifier) {
        self.get_curr_mut().modifiers.push_front(m);
    }
    pub fn add_fg_color(&mut self, c: Color) {
        self.add_modifier(CellModifier::FGColor(c));
    }
    pub fn add_bg_color(&mut self, c: Color) {
        self.add_modifier(CellModifier::BGColor(c));
    }
    pub fn add_text_effect(&mut self, e: TextEffect) {
        self.add_modifier(CellModifier::Effect(e));
    }
    pub fn undo_modifiers(&mut self) {
        self.add_modifier(CellModifier::Effect(TextEffect::None));
    }
    pub fn clear(&mut self) {
        self.cells.clear();
        self.resize();
    }
    pub fn erase_chars(&mut self, count: usize) {
        let new_idx = self.curr_idx.saturating_sub(count);
        for i in new_idx..=self.curr_idx {
            self.cells[i].clear();
        }
        self.curr_idx = new_idx;
    }

    fn get_curr_mut(&mut self) -> &mut Cell {
        &mut self.cells[self.curr_idx]
    }
}

impl fmt::Display for TermBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for cell in self.cells.iter() {
            write!(f, "{}", cell)?
        }
        // return things to normal
        write!(f, "\x1b[0m")
    }
}

#[allow(dead_code)]
pub struct Buffers {
    visible_buffer: TermBuffer,
    staging_buffer: TermBuffer, // TODO: Come up with a better name
}
