//! UI rendering of the domain's adoption phrases.
use crate::words::Words;
use alloc::boxed::Box;
use skein_ui::{Builder, Run, phrases::Hole};
use temper_client_domain::forge::words::{self, Word};
#[must_use]
pub fn say(words: &Words, word: Word, holes: &[Hole]) -> Box<[u8]> {
    words.book.say(words.language, words::id(words.connector, word), holes).expect("checked adoption phrase").text
}
pub fn text(words: &Words, builder: &mut Builder, word: Word, holes: &[Hole]) {
    builder.text(Run::Plain, &say(words, word, holes));
}
#[must_use]
pub fn capacity(words: &Words) -> Option<u32> {
    let mut most = 0;
    for (word, _, _) in words::ROWS {
        most = most.max(words.book.capacity(words::id(words.connector, word))?);
    }
    for word in words::RESOURCES {
        most = most.max(words.book.capacity(word.id(words.connector))?);
    }
    for (word, _, _) in words::CHANGES {
        most = most.max(words.book.capacity(word.id(words.connector))?);
    }
    // The pull prefix and repository share a text node; likewise all three bounds.
    let id = words::ChangeWord::id;
    let pull = words
        .book
        .capacity(id(words::ChangeWord::Pull, words.connector))?
        .max(words.book.capacity(id(words::ChangeWord::NoPull, words.connector))?)
        .checked_add(words.book.capacity(id(words::ChangeWord::Repository, words.connector))?)?;
    let bounds = words
        .book
        .capacity(id(words::ChangeWord::Repairs, words.connector))?
        .checked_add(words.book.capacity(id(words::ChangeWord::Resolutions, words.connector))?)?
        .checked_add(words.book.capacity(id(words::ChangeWord::Updates, words.connector))?)?
        .checked_add(words.book.capacity(id(words::ChangeWord::Separator, words.connector))?.checked_mul(2)?)?;
    // A full supplied external address and its label must fit every view preset.
    let external = temper_client_domain::forge::change::ADDRESS_BYTES
        .checked_add(words.book.capacity(id(words::ChangeWord::View, words.connector))?)?;
    Some(most.max(pull).max(bounds).max(external))
}
