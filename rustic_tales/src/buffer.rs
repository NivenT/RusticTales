use std::collections::LinkedList;
use std::fmt;

use regex::Regex;

use crate::utils::*;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
#[allow(dead_code)]
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
    pub const fn light(col: BaseColor) -> Color {
        Color(col, true)
    }
    pub const fn dark(col: BaseColor) -> Color {
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
#[allow(dead_code)]
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

impl CellModifier {
    pub fn from_val(val: u8) -> Option<CellModifier> {
        use std::mem::transmute;
        match val {
            // SAFETY: #[repr(u8)] + bounds
            n @ 0..=9 => Some(CellModifier::Effect(unsafe { transmute(n) })),
            n @ 30..=37 => Some(CellModifier::FGColor(Color(
                unsafe { transmute(n - 30) },
                false,
            ))),
            n @ 40..=47 => Some(CellModifier::BGColor(Color(
                unsafe { transmute(n - 40) },
                false,
            ))),
            n @ 90..=97 => Some(CellModifier::FGColor(Color(
                unsafe { transmute(n - 90) },
                true,
            ))),
            n @ 100..=107 => Some(CellModifier::BGColor(Color(
                unsafe { transmute(n - 100) },
                true,
            ))),
            _ => None,
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
    pub fn area(&self) -> (usize, usize) {
        // tabs don't exist
        match self.c {
            '\0' => (0, 0),
            '\n' => (1, 0),
            _ => (0, 1),
        }
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

#[derive(Debug, Clone, Copy, Default)]
struct DirtyFlags {
    page_turned: bool,
    modified: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TermBuffer {
    cells: Vec<Cell>,
    rows: usize,
    cols: usize,
    curr_idx: usize,
    dirty: DirtyFlags,
}

impl TermBuffer {
    pub fn new() -> TermBuffer {
        let mut buf = TermBuffer::default();
        buf.resize();
        buf.cells.resize_with(buf.page_size(), Default::default);
        buf
    }
    pub fn resize(&mut self) {
        let (rows, cols) = terminal_dims();
        self.rows = rows as usize;
        self.cols = cols as usize;
    }
    pub fn just_turned_page(&mut self) -> bool {
        let ret = self.dirty.page_turned;
        self.dirty.page_turned = false;
        ret
    }
    pub fn just_modified(&mut self) -> bool {
        let ret = self.dirty.modified;
        self.dirty.modified = false;
        ret
    }
    pub fn page_size(&self) -> usize {
        self.rows * self.cols
    }
    pub fn move_cursor(&mut self, num_cells: isize) {
        use std::cmp::{max, min};
        let naive_new = self.curr_idx as isize + num_cells;
        self.curr_idx = min(max(naive_new, 0) as usize, self.cells.len() - 1);
    }
    // returns (row, column)
    pub fn get_cursor(&self) -> (usize, usize) {
        let mut ret = (0, 0);
        for cell in self.curr_content() {
            let (dr, dc) = cell.area();
            ret.0 += dr;
            ret.1 += dc;
        }
        ret
    }
    pub fn turn_page(&mut self) {
        self.curr_idx = self.curr_page_end_idx();
        self.advance_idx();
    }
    pub fn write_char(&mut self, c: char) {
        self.dirty.modified = true;
        self.get_curr_mut().c = c;
        self.advance_idx();
    }
    // TODO: handle t which includes both modifiers and text?
    pub fn write_text(&mut self, t: &str) {
        let idx = self.try_parse_modifier(t);
        t[idx..].chars().for_each(|c| self.write_char(c))
    }
    pub fn add_modifier(&mut self, m: CellModifier) {
        self.get_curr_mut().modifiers.push_back(m);
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
        self.dirty.modified = true;
        self.cells.clear();
        self.resize();
        self.cells.resize_with(self.page_size(), Default::default);
        self.curr_idx = 0;
    }
    pub fn erase_chars(&mut self, count: usize) {
        self.dirty.modified = true;
        let new_idx = self.curr_idx.saturating_sub(count);
        for i in new_idx..=self.curr_idx {
            self.cells[i].clear();
        }
        self.curr_idx = new_idx;
    }
    // TODO: Make this not trash
    pub fn erase_lines(&mut self, count: usize) {
        let (orig_last_line, _) = self.get_cursor();
        let mut last_line = orig_last_line;
        while last_line > 0 && (orig_last_line - last_line) < count {
            self.erase_chars(1);
            last_line = self.get_cursor().0;
        }
    }

    pub fn clear_and_dump(&self) {
        use std::io::Write;
        clear_screen();
        print!("{}", self);
        let _ = std::io::stdout().flush();
    }
    pub fn clear_and_dump_prev_page(&mut self) {
        debug_assert!(self.curr_page() > 0);
        self.curr_idx -= self.page_size();
        self.clear_and_dump();
        self.curr_idx += self.page_size();
    }

    fn try_parse_modifier(&mut self, m: &str) -> usize {
        let re = Regex::new(r"^\u{1b}\[((\d+;?)+)m").expect("Typo if this does not work");
        re.captures(m)
            .map(|cap| {
                cap[1]
                    .to_string()
                    .split(';')
                    .filter_map(|d| d.parse::<u8>().ok())
                    .filter_map(CellModifier::from_val)
                    .for_each(|m| self.add_modifier(m));
                cap[0].len()
            })
            .unwrap_or(0)
    }
    fn advance_idx(&mut self) {
        self.curr_idx += 1;
        while self.curr_idx >= self.cells.len() {
            self.dirty.page_turned = true;
            self.cells
                .resize_with(self.cells.len() + self.page_size(), Default::default);
        }
    }
    fn curr_page(&self) -> usize {
        self.curr_idx / self.page_size()
    }
    fn curr_content(&self) -> &[Cell] {
        &self.cells[self.curr_page_start_idx()..=self.curr_page_end_idx()]
    }
    fn curr_page_start_idx(&self) -> usize {
        self.page_size() * self.curr_page()
    }
    fn curr_page_end_idx(&self) -> usize {
        self.page_size() * (self.curr_page() + 1) - 1
    }
    fn get_curr_mut(&mut self) -> &mut Cell {
        &mut self.cells[self.curr_idx]
    }
}

impl fmt::Display for TermBuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}Page {}\n{}",
            CellModifier::FGColor(Color::light(BaseColor::Red)),
            self.curr_page(),
            TextEffect::None
        )?;
        for cell in self.curr_content() {
            write!(f, "{}", cell)?
        }
        // return things to normal
        write!(f, "\x1b[0m")
    }
}
