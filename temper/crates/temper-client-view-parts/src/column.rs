//! The outside pages' column (temper/docs/design/client/view-crates.md, 3.2).
use crate::{
    look,
    words::{Word, Words},
};
use skein_ui::{Builder, Role, Run, Style, phrases::Hole};
use temper_client_domain::Domain;

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

/// The Welcome step's paragraph under the cards: what temper box is for,
/// and where running temper on a server is told (client/pages/welcome.md, 1).
pub(crate) fn box_line(builder: &mut Builder, words: &Words) {
    let name = words.say(Word::WelcomeBoxName, &[]);
    let line = words.say(Word::WelcomeBoxLine, &[]);
    let server = words.say(Word::WelcomeBoxServer, &[]);
    builder.open(Role::Paragraph);
    builder.style(look::WELCOME_AFTER);
    builder.style(look::WELCOME_BOX_NOTE);
    builder.open(Role::Text);
    builder.run(Run::Strong, &name);
    builder.run(Run::Plain, &line);
    builder.close();
    crate::flow::control(
        builder,
        crate::flow::Control::External { name: &server, address: crate::forge::connect::DEPLOYMENT },
    );
    builder.close();
}

pub(crate) fn unavailable(root: &Domain) -> Option<Word> {
    match root.client().link().state {
        jig_client::Link::SignedOut => Some(Word::SignedOut),
        jig_client::Link::OutOfDate => Some(Word::OutOfDate),
        jig_client::Link::Starting | jig_client::Link::Live | jig_client::Link::Offline => None,
    }
}
