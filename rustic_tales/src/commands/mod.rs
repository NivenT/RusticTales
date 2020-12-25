use super::storyteller::DisplayUnit;

pub fn backspace(len: usize, unit: DisplayUnit) {
    if unit.is_char() {
        for _ in 0..len {
            print!("\u{8}")
        }
    } else {
        unimplemented!()
    }
}
