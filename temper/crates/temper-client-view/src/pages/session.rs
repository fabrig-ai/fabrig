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
/// Closes the entry and draws its row: hint and picker (client/README.md, 3.5).
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
    if hint == Word::ChatSendResumes {
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
    crate::link(b, words, bindings.insert(Act::Go { address }));
    if let Some(style) = style {
        b.style(style);
    }
    b.text(Run::Plain, words);
    b.close();
}

/// The line under a composer past what one message holds (client/README.md, 5.4).
pub(crate) fn over(b: &mut Builder, w: &Words, characters: u64) {
    if characters == 0 {
        return;
    }
    let line = if characters == 1 {
        w.say(Word::ComposerOverOne, &[])
    } else {
        let digits = skein_lib::Decimal::of(characters);
        let digits = digits.as_bytes();
        let commas = digits.len().checked_sub(1).expect("decimal has digits").checked_div(3).expect("three is nonzero");
        let mut number = skein_lib::Writer::new(digits.len().checked_add(commas).expect("grouped u64 fits"));
        for (index, digit) in digits.iter().enumerate() {
            if index != 0 && digits.len().checked_sub(index).expect("digit is in decimal").checked_rem(3) == Some(0) {
                number.put(b",").expect("grouped length counted");
            }
            number.put(core::slice::from_ref(digit)).expect("grouped length counted");
        }
        w.say(Word::ComposerOver, &[Hole::Name(number.finish())])
    };
    text(b, Role::Status, &line, None);
}

/// The browser bar keeps the chat's number (client/README.md, 3.5).
pub(crate) fn number(b: &mut Builder, w: &Words, task: u64) {
    match w.shell {
        crate::words::Shell::Browser => {
            text(b, Role::Paragraph, &w.say(Word::Chat, &[Hole::Number(task)]), Some(look::BYLINE));
        }
        crate::words::Shell::Terminal => {}
    }
}
