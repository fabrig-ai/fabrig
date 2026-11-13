//! Shared change items (temper/docs/design/client/view-crates.md, 3.3).
use crate::words::Words;
use alloc::boxed::Box;
use skein_ui::phrases::Hole;
use temper_client_domain::forge::words::ChangeWord as W;

pub(crate) fn say(words: &Words, word: W, holes: &[Hole]) -> Box<[u8]> {
    words.book.say(words.language, word.id(words.connector), holes).expect("checked change phrase").text
}
