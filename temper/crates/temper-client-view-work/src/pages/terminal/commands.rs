//! Commands and their lists under the box (temper/docs/design/client/pages/README.md).
use super::{Act, Bindings, Builder, Domain, Role, Run, Shortcut, State, Words, look};
pub(crate) use crate::common::terminal::commands::confirms;
use jig_client::contract as c;
use temper_client_domain::pages::terminal::{self, Command, Listing, Note};

fn words(root: &Domain) -> &[u8] {
    if let Some(chat) = root.pages().chat() {
        &chat.words
    } else if let Some(chats) = root.pages().chats() {
        &chats.words
    } else {
        b""
    }
}
pub(crate) fn typing(root: &Domain) -> bool {
    prefix(words(root))
}
#[expect(clippy::disallowed_methods, reason = "Unicode whitespace is read from the composer’s validated UTF-8")]
fn prefix(words: &[u8]) -> bool {
    if !words.starts_with(b"/") {
        return false;
    }
    let Ok(text) = core::str::from_utf8(words) else { return false };
    for character in text.chars() {
        if character.is_whitespace() {
            return false;
        }
    }
    true
}
fn menu(b: &mut Builder, name: &[u8], head: Option<&[u8]>, wide: bool) {
    b.open(Role::Menu);
    b.name(name);
    b.state(State::Options);
    if wide {
        b.style(look::TERMINAL_CHOICES);
    }
    if let Some(head) = head {
        b.open(Role::Text);
        b.style(look::TERMINAL_HEAD);
        b.name(head);
        b.text(Run::Plain, head);
        b.close();
    }
}
fn option(b: &mut Builder, bindings: &mut Bindings, name: &[u8], act: Act) {
    b.open(Role::MenuItem);
    b.name(name);
    b.bind(bindings.insert(act));
    b.close();
}
pub(super) fn under(
    b: &mut Builder,
    root: &Domain,
    task: Option<&c::Task>,
    w: &Words,
    bindings: &mut Bindings,
) -> bool {
    let listing = root.pages().listing();
    if confirms(root) {
        let Some(confirmation) = crate::shared::confirmation(root) else { return false };
        menu(b, confirmation.head, Some(confirmation.head), true);
        option(b, bindings, confirmation.take, Act::Confirm);
        option(b, bindings, confirmation.keep, Act::Cancel);
        b.close();
    } else if let Some(listing) = listing {
        let head = crate::shared::list_head(listing);
        menu(b, head, Some(head), listing != Listing::Chats);
        match listing {
            Listing::Model => crate::choice::picker::terminal_models(b, root, w, bindings),
            Listing::Effort => crate::choice::picker::terminal_efforts(b, root, w, bindings),
            Listing::Chats => chat_options(b, root, w, bindings),
        }
        b.close();
    } else {
        let words = words(root);
        if !prefix(words) {
            return false;
        }
        let mut any = false;
        for command in Command::ALL {
            if command.name().starts_with(words) && crate::shared::offered(root, task, command) {
                any = true;
            }
        }
        if !any {
            return false;
        }
        menu(b, b"Commands", None, false);
        for command in Command::ALL {
            if command.name().starts_with(words) && crate::shared::offered(root, task, command) {
                b.open(Role::MenuItem);
                b.name(command.name());
                b.description(crate::shared::description(command));
                b.bind(bindings.insert(Act::Terminal(terminal::Act::Run { command })));
                b.close();
            }
        }
        b.close();
    }
    b.open(Role::Button);
    b.name(b"Close the list");
    b.style(look::TERMINAL_DISMISS);
    b.bind(bindings.insert(Act::Terminal(terminal::Act::Dismiss)));
    b.shortcut(Shortcut::ESCAPE);
    b.close();
    true
}
fn chat_options(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    for choice in &crate::shared::chats(root, w) {
        b.open(Role::MenuItem);
        b.name(&choice.name);
        b.description(&choice.words);
        b.bind(bindings.insert(choice.act.clone()));
        b.close();
    }
}

pub(super) fn notes(b: &mut Builder, root: &Domain) {
    for note in root.pages().notes() {
        let role = match note {
            Note::Help => Role::Code,
            Note::Unknown { .. } | Note::Font { .. } | Note::Closed => Role::Paragraph,
        };
        let words = crate::shared::note(note);
        b.open(role);
        b.style(look::TERMINAL_NOTE);
        b.name(&words);
        b.text(Run::Plain, &words);
        b.close();
    }
}
