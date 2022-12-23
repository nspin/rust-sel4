use crate::{sel4_cfg_usize, sys};

pub type Word = sys::seL4_Word;

pub const WORD_SIZE: usize = sel4_cfg_usize!(WORD_SIZE);

pub type Badge = Word;

pub fn r#yield() {
    sys::seL4_Yield()
}
