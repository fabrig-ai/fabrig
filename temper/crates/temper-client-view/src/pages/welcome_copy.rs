//! The welcome's command block and its Copy (temper/docs/design/client/README.md, 3.5).
use crate::{
    accounts::{Bindings, Meaning},
    look,
    words::{Shell, Word, Words},
};
use skein_ui::{Builder, Role, Run};
use temper_client_domain::{Domain, copy::Label};

pub(super) fn block(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, words: &[u8]) {
    let browser = w.shell == Shell::Browser;
    if browser {
        b.open(Role::Slot);
        b.style(look::WELCOME_CMD_BOX);
    }
    b.open(Role::Code);
    b.style(look::WELCOME_CMD);
    b.text(Run::Plain, words);
    b.close();
    if browser {
        let word = match root.copy_label() {
            Label::Copy => Word::WelcomeCopy,
            Label::Copied => Word::WelcomeCopied,
            Label::NotCopied => Word::WelcomeNotCopied,
        };
        if let Some(to) = bindings.insert(Meaning::Copy { words: skein_lib::bytes::copy_of(words) }) {
            b.open(Role::Button);
            b.style(look::FLOW_QUIET);
            b.style(look::WELCOME_COPY);
            b.name(&w.say(word, &[]));
            w.text(b, word, &[]);
            b.bind(to);
            b.close();
        }
        b.close();
    }
}
