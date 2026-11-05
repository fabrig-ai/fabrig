//! A session's path, bar and composer (temper/docs/design/client/README.md, 3.5).
use crate::{
    frame::Bindings,
    look,
    words::{Word, Words},
};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{Address, Domain, pages::Act};

/// Where the session's path leads.
pub(crate) enum Path {
    Chats,
    Settings,
}
/// A chat retains its numbered name while the browser shows its title.
pub(crate) enum Heading<'a> {
    Words(&'a [u8]),
    Chat { whole: &'a [u8], title: &'a [u8] },
}
/// Opens the bar and its path; the caller adds its right part and closes it.
pub(crate) fn top(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    project: Option<u32>,
    path: Path,
    heading: Heading<'_>,
) {
    // Banner is a page landmark in skein-ui; this bar belongs inside the main.
    b.open(Role::Slot);
    b.style(look::SESSION_TOP);
    b.open(Role::Navigation);
    b.name(&w.say(Word::ConfigureWhere, &[]));
    b.style(look::CRUMBS);
    let address = match path {
        Path::Chats => Address::Chats { filter: temper_client_domain::ChatsFilter::All },
        Path::Settings => Address::Settings { project: project.expect("Settings has a project") },
    };
    let label = w.say(
        match path {
            Path::Chats => Word::Chats,
            Path::Settings => Word::RailSettings,
        },
        &[],
    );
    link(b, bindings, &label, address.clone(), Some(look::BACK));
    b.open(Role::Slot);
    b.style(look::TRAIL);
    if let Some(project) = project {
        let name = match super::project_name(root.pages().me(root.client()), project) {
            Some(name) => name,
            None => w.say(Word::Project, &[Hole::Number(u64::from(project))]),
        };
        link(b, bindings, &name, Address::Home { project }, None);
    }
    link(b, bindings, &label, address, None);
    b.close();
    match heading {
        Heading::Words(words) => text(b, Role::Heading { level: 1 }, words, None),
        Heading::Chat { whole, title } => {
            b.open(Role::Heading { level: 1 });
            b.name(whole);
            b.text(
                Run::Plain,
                match w.shell {
                    crate::words::Shell::Browser => title,
                    crate::words::Shell::Terminal => whole,
                },
            );
            b.close();
        }
    }
    b.close();
}
/// Opens the bar's right part; the caller closes it.
pub(crate) fn right(b: &mut Builder) {
    b.open(Role::Slot);
    b.style(look::SESSION_RIGHT);
}
/// Opens the shared composer form and its entry; the caller adds controls and closes it.
pub(crate) fn foot(b: &mut Builder, name: &[u8]) {
    b.open(Role::Form);
    b.name(name);
    b.style(look::SESSION_FOOT);
    b.open(Role::Slot);
    b.style(look::SESSION_COMPOSER);
}
/// Closes the entry and follows it with the shared keyboard hint.
pub(crate) fn hint(b: &mut Builder, w: &Words, document: bool) {
    b.close();
    text(
        b,
        Role::Paragraph,
        &w.say(if document { Word::DraftSendHint } else { Word::ChatSendHint }, &[]),
        Some(look::SESSION_HINT),
    );
}

fn text(b: &mut Builder, role: Role, words: &[u8], style: Option<skein_ui::Style>) {
    b.open(role);
    if let Some(style) = style {
        b.style(style);
    }
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
fn link(b: &mut Builder, bindings: &mut Bindings, words: &[u8], address: Address, style: Option<skein_ui::Style>) {
    crate::link(b, words, bindings.insert(Act::Go { address }));
    if let Some(style) = style {
        b.style(style);
    }
    b.text(Run::Plain, words);
    b.close();
}
