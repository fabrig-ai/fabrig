//! The frame and the column outside a project
//! (temper/docs/design/client/README.md, 5.1; temper/docs/design/client/browser.md, 2.5).
use crate::{
    look,
    words::{Word, Words},
};
use skein_ui::{Builder, Role, Run, Style, phrases::Hole};

/// Opens the column's main, `look::OUTSIDE`, then `extra` styles in order, and writes its
/// level-1 heading, named and worded `heading`; the caller draws the rest and closes the main.
pub(crate) fn column(b: &mut Builder, heading: &[u8], extra: &[Style]) {
    b.open(Role::Main);
    b.style(look::OUTSIDE);
    for style in extra {
        b.style(*style);
    }
    b.open(Role::Heading { level: 1 });
    b.name(heading);
    b.text(Run::Plain, heading);
    b.close();
}

/// One paragraph of the column's words.
pub(crate) fn words(b: &mut Builder, words: &Words, word: Word, holes: &[Hole]) {
    b.open(Role::Paragraph);
    b.name(&words.say(word, holes));
    words.text(b, word, holes);
    b.close();
}
