//! A session's path, bar and composer (temper/docs/design/client/pages/README.md).
use crate::{
    bindings::Bindings,
    look,
    words::{Word, Words},
};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{Address, Domain, pages::Act};

/// Where the session's path leads.
pub(crate) enum Path {
    Chats,
    Ask,
    Home,
}
/// A chat retains its numbered name while the browser shows its title.
pub(crate) enum Heading<'a> {
    Words(&'a [u8]),
    /// A chat's: named `whole`, "TN · TITLE"; showing `short` in a browser and `numbered`,
    /// "TN · SHORT", in the terminal; with `hint`, the whole title, where it is cut.
    Chat {
        whole: &'a [u8],
        short: &'a [u8],
        numbered: &'a [u8],
        hint: Option<&'a [u8]>,
    },
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
    b.name(&w.say(Word::AskWhere, &[]));
    b.style(look::CRUMBS);
    let address = match path {
        Path::Chats => Address::Chats { filter: temper_client_domain::ChatsFilter::All },
        Path::Ask => Address::Ask { project: project.expect("Ask has a project") },
        Path::Home => Address::Home { project: project.expect("a new Ask has a project") },
    };
    let label = w.say(
        match path {
            Path::Chats => Word::Chats,
            Path::Ask => Word::AskTitle,
            Path::Home => Word::RailHome,
        },
        &[],
    );
    link(b, bindings, &label, address.clone(), Some(look::BACK));
    b.open(Role::Slot);
    b.style(look::TRAIL);
    match path {
        Path::Chats | Path::Ask => link(b, bindings, &label, address, None),
        Path::Home => {
            if let Some(project) = project {
                let name = match super::project_name(root.pages().me(root.client()), project) {
                    Some(name) => name,
                    None => w.say(Word::Project, &[Hole::Number(u64::from(project))]),
                };
                link(b, bindings, &name, Address::Home { project }, None);
            }
        }
    }
    b.close();
    match heading {
        Heading::Words(words) => text(b, Role::Heading { level: 1 }, words, None),
        Heading::Chat { whole, short, numbered, hint } => {
            b.open(Role::Heading { level: 1 });
            b.name(whole);
            if let Some(hint) = hint {
                b.hint(hint);
            }
            b.text(
                Run::Plain,
                match w.shell {
                    crate::words::Shell::Browser => short,
                    crate::words::Shell::Terminal => numbered,
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
    form(b, name);
    composer(b);
}
/// Opens the shared composer form; the caller adds its lines and closes it.
pub(crate) fn form(b: &mut Builder, name: &[u8]) {
    b.open(Role::Form);
    b.name(name);
    b.style(look::SESSION_FOOT);
}
/// Opens the form's entry; the caller adds controls and closes it.
pub(crate) fn composer(b: &mut Builder) {
    b.open(Role::Slot);
    b.style(look::SESSION_COMPOSER);
}
/// Closes the entry and draws its row: hint and picker (client/pages/README.md).
pub(crate) fn foot_row(
    b: &mut Builder,
    root: &Domain,
    task: Option<&jig_client::contract::Task>,
    w: &Words,
    bindings: &mut Bindings,
    hint: Word,
) {
    b.close();
    b.open(Role::Slot);
    b.style(look::FOOT_ROW);
    b.open(Role::Paragraph);
    b.style(look::SESSION_HINT);
    if hint == Word::ChatSendResumes || hint == Word::DraftSendResumes {
        b.style(look::HINT);
    }
    let words = w.say(hint, &[]);
    b.name(&words);
    b.text(Run::Plain, &words);
    b.close();
    crate::choice::picker(b, root, task, w, bindings);
    b.close();
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
    crate::bindings::link(b, words, bindings.insert(Act::Go { address }));
    if let Some(style) = style {
        b.style(style);
    }
    b.text(Run::Plain, words);
    b.close();
}

/// The line under a composer past what one message holds (client/view.md, 4).
pub(crate) fn over(b: &mut Builder, w: &Words, characters: u64) {
    if characters == 0 {
        return;
    }
    let line = if characters == 1 {
        w.say(Word::ComposerOverOne, &[])
    } else {
        w.say(Word::ComposerOver, &[Hole::Name(crate::words::grouped(characters))])
    };
    text(b, Role::Status, &line, None);
}

/// The browser bar keeps the chat's number (client/pages/README.md).
pub(crate) fn number(b: &mut Builder, w: &Words, task: u64) {
    match w.shell {
        crate::words::Shell::Browser => {
            text(b, Role::Paragraph, &w.say(Word::Chat, &[Hole::Number(task)]), Some(look::BYLINE));
        }
        crate::words::Shell::Terminal => {}
    }
}

/// A closed chat's number and what it ran on, in both shells (client/pages/README.md).
pub(crate) fn closed_number(b: &mut Builder, w: &Words, task: u64, line: Option<&[u8]>) {
    match line {
        Some(line) => text(
            b,
            Role::Paragraph,
            &w.say(Word::BylineTask, &[Hole::Number(task), Hole::Name(skein_lib::bytes::copy_of(line))]),
            Some(look::BYLINE),
        ),
        None => number(b, w, task),
    }
}

/// Send feedback beneath the composer (temper/docs/design/client/pages/README.md).
pub(crate) fn refusals(b: &mut Builder, root: &Domain, w: &Words) {
    for line in &crate::shared::composer_refusals(root, w) {
        crate::confirm::feedback(b, line);
    }
}
