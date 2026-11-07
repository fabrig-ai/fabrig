//! A character's cell width (skein/docs/design/ui-screen.md, 5.1).

mod table;

struct Range {
    first: u32,
    last: u32,
    width: u8,
}

/// Zero for combining marks and the contract's zero-width characters, two
/// for East Asian Wide and Fullwidth characters, one for every other.
#[must_use]
pub fn width(glyph: char) -> u8 {
    let value = u32::from(glyph);
    let mut ranges = table::RANGES;
    // Each step halves the remaining slice; usize::BITS bounds the search.
    for _step in 0..usize::BITS {
        let middle = ranges.len().checked_div(2).expect("two is nonzero");
        let (left, right) = ranges.split_at_checked(middle).expect("the middle is in the slice");
        let Some((range, rest)) = right.split_first() else { return 1 };
        if value < range.first {
            ranges = left;
        } else if value > range.last {
            ranges = rest;
        } else {
            return range.width;
        }
    }
    1
}
