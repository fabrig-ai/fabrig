//! Retained protocol-rendered transcript text (domain/engine/executors.md, 2, and domain/engine/briefs.md, 1).
pub use crate::below::runs::transcript_text::*;

use crate::Transcript;
use alloc::boxed::Box;
use skein_lib::length::len_u64;

/// The text a cut turn keeps in a fresh brief's tail (domain/engine/executors.md, 2).
pub const CUT_TURN_TEXT: &[u8] = b"[a turn too large to load]\n";

/// Rendered bytes and the original text bytes they still show (excluding a cut marker).
#[derive(Hash, Debug)]
pub struct TranscriptText {
    pub bytes: Box<[u8]>,
    pub shown: u64,
}

/// Keeps a text's end without decoding it; the protocol supplied UTF-8.
pub(crate) fn cut_transcript_text(text: &[u8], bound: usize) -> TranscriptText {
    const MARKER: &[u8] = b"[earlier text cut]\n";
    if text.len() <= bound {
        return TranscriptText { bytes: Box::from(text), shown: len_u64(text.len()) };
    }
    if bound < MARKER.len() {
        return TranscriptText { bytes: Box::new([]), shown: 0 };
    }
    let room = bound.checked_sub(MARKER.len()).expect("marker fits");
    let mut start = text.len().checked_sub(room).expect("oversized text");
    for _ in 0_u8..3 {
        if skein_lib::bytes::utf8_prefix(text, start).len() == start {
            break;
        }
        start = start.checked_add(1).expect("inside text");
    }
    let suffix = text.get(start..).expect("UTF-8 suffix boundary within text");
    let mut writer = skein_lib::Writer::new(MARKER.len().checked_add(suffix.len()).expect("bounded text"));
    writer.put(MARKER).expect("measured marker");
    writer.put(suffix).expect("measured suffix");
    TranscriptText { bytes: writer.finish(), shown: len_u64(suffix.len()) }
}

impl Transcript {
    pub(crate) fn append_text(&mut self, text: Option<Box<[u8]>>, bound: u32) {
        let text = match text {
            Some(text) => text,
            None => Box::from(&b"[a turn not readable as text]\n"[..]),
        };
        self.text_bytes = self.text_bytes.checked_add(len_u64(text.len())).expect("loaded text count");
        let kept = if text.len() <= skein_lib::length::usize_of(bound) {
            TranscriptText { shown: len_u64(text.len()), bytes: text }
        } else {
            cut_transcript_text(&text, skein_lib::length::usize_of(bound))
        };
        if kept.bytes.is_empty() {
            return;
        }
        let length = len_u64(kept.bytes.len());
        for _ in 0..self.texts.len() {
            if self.text_kept.checked_add(length).expect("two bounded texts") <= u64::from(bound) {
                break;
            }
            let old = self.texts.pop().expect("overfull text queue has an oldest text");
            self.text_kept = self.text_kept.checked_sub(len_u64(old.bytes.len())).expect("counted text");
        }
        self.texts.push(kept);
        self.text_kept = self.text_kept.checked_add(length).expect("bounded text retention");
    }
}
