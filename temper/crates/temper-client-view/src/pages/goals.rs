//! The project goals' page (temper/docs/design/client/README.md, 3.5).
use crate::{
    frame::Bindings,
    look,
    words::{Times, Word, Words},
};
use skein_ui::{Builder, Role};
use temper_client_domain::Domain;

pub(crate) fn main(
    b: &mut Builder,
    _root: &Domain,
    words: &Words,
    _bindings: &mut Bindings,
    _project: u32,
    _times: &mut Times,
) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.open(Role::Heading { level: 1 });
    b.name(&words.say(Word::Goals, &[]));
    words.text(b, Word::Goals, &[]);
    b.close();
    b.close();
}
