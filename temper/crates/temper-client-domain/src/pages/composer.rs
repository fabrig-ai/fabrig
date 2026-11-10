//! What a person types into a composer, held as typed (client/view.md, 4).
use alloc::boxed::Box;

/// An edit's first `hold` bytes, cut back to a character, and the characters past them.
#[must_use]
pub fn held(words: Box<[u8]>, hold: u32) -> (Box<[u8]>, u64) {
    let end = fitting(&words, hold);
    if end == words.len() {
        return (words, 0);
    }
    let beyond = over(&words, 0, hold);
    (Box::from(words.get(..end).expect("fitting prefix")), beyond)
}

/// The characters of what was typed past `bound` bytes: none within it.
#[must_use]
pub fn over(words: &[u8], beyond: u64, bound: u32) -> u64 {
    let end = fitting(words, bound);
    let mut characters = beyond;
    for byte in words.get(end..).expect("fitting suffix") {
        if *byte & 0xc0 != 0x80 {
            characters = characters.saturating_add(1);
        }
    }
    characters
}

fn fitting(words: &[u8], bound: u32) -> usize {
    let bound = usize::try_from(bound).expect("text bound fits");
    if words.len() <= bound {
        return words.len();
    }
    let mut end = 0;
    for (index, byte) in words.iter().enumerate().take(bound.saturating_add(1)) {
        if *byte & 0xc0 != 0x80 {
            end = index;
        }
    }
    end
}
