//! UTF-8 projections fitted to forge byte bounds (domain/forge.md, 8.1 and 12).
use alloc::boxed::Box;
use skein_lib::Writer;

/// Use the first line's first sentence, dropping its final full stop.
/// An overlong sentence ends at a word with an ellipsis, or on UTF-8 when no word fits.
#[must_use]
pub fn title(words: &[u8], bound: usize) -> Box<[u8]> {
    let mut end = words.len();
    for (index, byte) in words.iter().enumerate() {
        if *byte == b'\n' {
            end = index;
            break;
        }
    }
    let line = words.get(..end).expect("first line");
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    let mut end = line.len();
    for (index, pair) in line.windows(2).enumerate() {
        if pair == b". " || pair == b"? " || pair == b"! " {
            end = index.checked_add(1).expect("sentence end within line");
            break;
        }
    }
    let sentence = line.get(..end).expect("first sentence");
    let sentence = sentence.strip_suffix(b".").unwrap_or(sentence);
    if sentence.len() <= bound {
        return sentence.into();
    }
    let marker = "…".as_bytes();
    let mut word_end = None;
    for (index, byte) in sentence.iter().enumerate() {
        if index > bound.saturating_sub(marker.len()) {
            break;
        }
        if *byte == b' ' && index > 0 {
            word_end = Some(index);
        }
    }
    let room = match word_end {
        Some(end) => end.checked_add(marker.len()).expect("word and marker within bound"),
        None => bound,
    };
    fit(sentence, room, marker)
}

/// Keep UTF-8 words unchanged when they fit, otherwise append the supplied cut marker.
/// A bound smaller than the marker carries an ellipsis when it can fit one.
#[must_use]
pub fn fit(words: &[u8], bound: usize, marker: &[u8]) -> Box<[u8]> {
    if words.len() <= bound {
        return words.into();
    }
    let marker = if marker.len() <= bound {
        marker
    } else if bound >= 3 {
        "…".as_bytes()
    } else {
        return Box::new([]);
    };
    let mut end = bound.saturating_sub(marker.len()).min(words.len());
    while end > 0 {
        match words.get(end) {
            Some(byte) if byte & 0xc0 == 0x80 => end = end.checked_sub(1).expect("positive prefix length"),
            Some(_) | None => break,
        }
    }
    let mut out = Writer::new(end.checked_add(marker.len()).expect("bounded projection length"));
    out.put(words.get(..end).expect("fitted prefix length")).expect("fitted prefix");
    out.put(marker).expect("fitted marker");
    out.finish()
}
