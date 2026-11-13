//! Add repositories over Settings (temper/docs/design/client/forge/family.md, 4;
//! temper/docs/design/client/forge/repositories.md, 1).
use super::{
    bindings::{Bindings, Meaning},
    words,
};
use crate::{look, words::Words};
use alloc::boxed::Box;
use skein_lib::bytes;
use skein_ui::{Builder, Role, Run, Shortcut, State, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    forge::{
        self,
        sheet::{self, Act, Source},
        words::Word,
    },
    pages,
};

pub(crate) fn draw(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut crate::bindings::Bindings,
    forge: &mut Bindings,
    project: u32,
) {
    let Some(sheet) = root.forge().sheet() else { return };
    let Some(form) = root.forge().form() else { return };
    let project_name = match crate::common::project_name(root.pages().me(root.client()), project) {
        Some(name) => name,
        None => words::say(w, Word::SheetProject, &[Hole::Number(u64::from(project))]),
    };
    let title = words::say(w, Word::SheetTitle, &[Hole::Name(project_name.clone())]);
    b.open(Role::Dialog);
    b.style(look::CONNECT_SHEET);
    b.name(&title);
    b.open(Role::Form);
    b.style(look::CONNECT_PANEL);
    b.open(Role::Slot);
    b.style(look::CONNECT_SHEET_HEAD);
    if w.shell == crate::words::Shell::Browser {
        crate::cards::text(b, Role::Heading { level: 2 }, &title);
    }
    let mut makes = false;
    for row in &forge::listed::rows(root.forges(), None) {
        makes |= sheet::makes(row);
    }
    if makes {
        b.open(Role::ChoiceGroup);
        b.style(look::CONNECT_SOURCES);
        b.name(&words::say(w, Word::Sources, &[]));
        for (source, word) in
            [(Source::Forge, Word::Adopt), (Source::Empty, Word::MakeEmpty), (Source::Copy, Word::CopyPublic)]
        {
            b.open(Role::Choice);
            b.name(&words::say(w, word, &[]));
            if sheet.source == source {
                b.state(State::Selected);
            }
            b.bind(forge.insert(Meaning::Sheet(Act::Source(source))));
            b.close();
        }
        b.close();
    }
    b.close();
    b.open(Role::Slot);
    b.style(look::CONNECT_SHEET_BODY);
    b.open(Role::Slot);
    b.style(look::CONNECT_SOURCE);
    match sheet.source {
        Source::Forge => {
            choices(b, root, sheet, w, forge);
            picker(b, root, sheet, &project_name, w, forge);
            not_listed(b, root, sheet, w, bindings, forge);
            more(b, sheet, form, w, forge);
            reading(b, sheet, w, bindings, forge);
        }
        Source::Empty | Source::Copy => making(b, root, sheet, w, forge),
    }
    b.close();
    if let sheet::Sent::Refused(text) = &sheet.sent {
        crate::cards::text(b, Role::Alert, text);
    }
    if let Some((request, progress)) = form.pending {
        let progress = match root.client().pending(jig_client::Key {
            asker: temper_client_domain::owner::Owner::Forge.asker(),
            number: request.raw(),
        }) {
            Some(pending) => pending.progress,
            None => match progress {
                skein_lib::RequestProgress::InFlight => jig_client::Progress::InFlight,
                skein_lib::RequestProgress::Retrying => jig_client::Progress::Retrying { why: jig_client::Retry::Lost },
                skein_lib::RequestProgress::Parked => jig_client::Progress::Parked,
                skein_lib::RequestProgress::Unknown => jig_client::Progress::Unknown,
            },
        };
        crate::cards::progress(b, w, progress);
    }
    if form.served == forge::Served::OutOfDate {
        text(b, w, Role::Status, Word::OutOfDate, &[]);
    }
    b.close();
    b.open(Role::Slot);
    b.style(look::CONNECT_SHEET_FOOT);
    b.open(Role::Button);
    b.style(look::CONNECT_QUIET);
    b.name(&w.say(crate::words::Word::Cancel, &[]));
    b.bind(bindings.insert(pages::Act::Go { address: Address::Repositories { project } }));
    b.shortcut(Shortcut::ESCAPE);
    b.close();
    let (button, enabled) = sheet::button(sheet, form, root.forges());
    let name = button_name(&button, w);
    b.open(Role::Button);
    b.style(look::CONNECT_PRIMARY);
    b.name(&name);
    b.bind(forge.insert(Meaning::Sheet(Act::Send)));
    if !enabled {
        b.state(State::Disabled);
    }
    b.close();
    b.close();
    b.close();
    b.close();
}
fn text(b: &mut Builder, w: &Words, role: Role, word: Word, holes: &[Hole]) {
    crate::cards::text(b, role, &words::say(w, word, holes));
}
fn making(b: &mut Builder, root: &Domain, sheet: &sheet::Sheet, w: &Words, bindings: &mut Bindings) {
    let rows = forge::listed::rows(root.forges(), None);
    let mut count = 0_u32;
    for row in &rows {
        if sheet::makes(row) {
            count = count.checked_add(1).expect("bounded forges");
        }
    }
    if count > 1 {
        b.open(Role::Slot);
        b.open(Role::Select);
        b.style(look::SETTINGS_TEXT);
        b.name(&words::say(w, Word::SheetForge, &[]));
        for row in &rows {
            if !sheet::makes(row) {
                continue;
            }
            b.open(Role::Option);
            b.name(&super::connect::name(w, row, root));
            if sheet.target == Some(row.number) {
                b.state(State::Selected);
            }
            b.bind(bindings.insert(Meaning::Sheet(Act::Target(row.number))));
            b.close();
        }
        b.close();
        b.close();
    }
    match sheet.source {
        Source::Empty => {
            b.open(Role::Slot);
            b.style(look::CONNECT_PAIR);
            field(b, &sheet.org, Word::Organization, Meaning::Sheet(Act::Org(Box::default())), w, bindings);
            field(b, &sheet.name, Word::Name, Meaning::Sheet(Act::Name(Box::default())), w, bindings);
            b.close();
        }
        Source::Copy => {
            b.open(Role::Slot);
            if w.shell == crate::words::Shell::Browser {
                b.open(Role::Paragraph);
                b.style(look::CONNECT_TITLE);
                words::text(w, b, Word::Addresses, &[]);
                b.close();
            }
            b.open(Role::TextArea);
            b.style(look::SETTINGS_TEXT);
            b.style(look::CONNECT_ADDRESSES);
            b.name(&words::say(w, Word::Addresses, &[]));
            b.placeholder(&words::say(w, Word::AddressExample, &[]));
            b.value(&sheet.addresses.text, sheet.addresses.written);
            b.bind(bindings.insert(Meaning::Sheet(Act::Addresses(Box::default()))));
            if sheet.addresses.broken.is_some() {
                b.state(State::Invalid);
            }
            b.close();
            broken(b, sheet.addresses.broken, w);
            b.close();
            line_alerts(b, sheet, root.forges(), w, bindings);
        }
        Source::Forge => return,
    }
    if let Some(row) = sheet::row(root.forges(), sheet.target) {
        b.open(Role::Paragraph);
        b.style(look::CONNECT_MUTED);
        let word = match sheet.source {
            Source::Empty => Word::MakeOn,
            Source::Copy => Word::CopiedInto,
            Source::Forge => return,
        };
        words::text(w, b, word, &[Hole::Name(super::connect::name(w, row, root))]);
        b.close();
    }
}
fn line_alerts(b: &mut Builder, sheet: &sheet::Sheet, rows: &forge::forges::State, w: &Words, bindings: &mut Bindings) {
    let lines = sheet::lines(sheet, rows);
    let mut opened = false;
    for line in &lines {
        let Some(words) = forge::words::sheet_line(line) else { continue };
        let fix = match line {
            forge::making::Line::To { .. } => continue,
            forge::making::Line::Bad { .. } => None,
            forge::making::Line::NotAHost { line, .. } => Some((Word::UseIt, Act::UseIt { line: line.clone() })),
            forge::making::Line::Taken { org, name, .. } => {
                Some((Word::AdoptIt, Act::AdoptIt { org: org.clone(), name: name.clone() }))
            }
        };
        if !opened {
            b.open(Role::Alert);
            b.open(Role::List);
            b.style(look::CONNECT_ALERTS);
            opened = true;
        }
        b.open(Role::Item);
        b.text(Run::Plain, &words);
        if let Some((word, act)) = fix {
            b.text(Run::Plain, b" ");
            b.open(Role::Button);
            b.style(look::CONNECT_QUIET);
            b.name(&words::say(w, word, &[]));
            b.bind(bindings.insert(Meaning::Sheet(act)));
            b.close();
        }
        b.close();
    }
    if opened {
        b.close();
        b.close();
    }
}
fn choices(b: &mut Builder, root: &Domain, sheet: &sheet::Sheet, w: &Words, bindings: &mut Bindings) {
    let rows = forge::listed::rows(root.forges(), None);
    let mut installation_count = 0_u32;
    if let Some(row) = sheet::row(root.forges(), sheet.forge) {
        for installation in &row.installations {
            if !installation.stray && installation.account.is_some() {
                installation_count = installation_count.checked_add(1).expect("bounded installations");
            }
        }
    }
    if rows.len() <= 1 && installation_count <= 1 {
        return;
    }
    b.open(Role::Slot);
    b.style(look::CONNECT_PAIR);
    if rows.len() <= 1 || installation_count <= 1 {
        b.style(look::CONNECT_ONE);
    }
    if rows.len() > 1 {
        b.open(Role::Slot);
        b.open(Role::Paragraph);
        b.style(look::CONNECT_TITLE);
        words::text(w, b, Word::SheetForge, &[]);
        b.close();
        b.open(Role::Select);
        b.style(look::SETTINGS_TEXT);
        b.name(&words::say(w, Word::SheetForge, &[]));
        for row in &rows {
            b.open(Role::Option);
            b.name(&super::connect::name(w, row, root));
            if sheet.forge == Some(row.number) {
                b.state(State::Selected);
            }
            b.bind(bindings.insert(Meaning::Sheet(Act::Forge(row.number))));
            b.close();
        }
        b.close();
        b.close();
    }
    if let Some(row) = sheet::row(root.forges(), sheet.forge) {
        let mut count = 0_u32;
        for installation in &row.installations {
            if !installation.stray && installation.account.is_some() {
                count = count.checked_add(1).expect("bounded installations");
            }
        }
        if count > 1 {
            b.open(Role::Slot);
            b.open(Role::Paragraph);
            b.style(look::CONNECT_TITLE);
            words::text(w, b, Word::Account, &[]);
            b.close();
            b.open(Role::Select);
            b.style(look::SETTINGS_TEXT);
            b.name(&words::say(w, Word::Account, &[]));
            for installation in &row.installations {
                if installation.stray {
                    continue;
                }
                let Some(account) = &installation.account else { continue };
                let mark =
                    if installation.suspended { words::say(w, Word::AccountSuspended, &[]) } else { Box::default() };
                b.open(Role::Option);
                b.name(&bytes::concat(&[&account.login, &mark]));
                if sheet.installation == Some(installation.id) {
                    b.state(State::Selected);
                }
                b.bind(bindings.insert(Meaning::Sheet(Act::Installation(installation.id))));
                b.close();
            }
            b.close();
            b.close();
        }
    }
    b.close();
}
fn picker(b: &mut Builder, root: &Domain, sheet: &sheet::Sheet, project: &[u8], w: &Words, bindings: &mut Bindings) {
    if sheet::long(sheet) {
        b.open(Role::Search);
        b.name(&words::say(w, Word::Find, &[]));
        field(b, &sheet.find, Word::Find, Meaning::Sheet(Act::Find(Box::default())), w, bindings);
        b.close();
    }
    b.open(Role::List);
    b.style(look::CONNECT_LIST);
    if sheet::long(sheet) {
        b.style(look::CONNECT_BOUNDED);
    }
    b.name(&words::say(w, Word::Repositories, &[]));
    b.bind(bindings.insert(Meaning::Sheet(Act::NearEnd)));
    if sheet.more {
        b.state(State::Continues);
    }
    if let sheet::Reading::Asked { .. } = sheet.reading {
        b.state(State::Busy);
    }
    let shown = sheet::shown(sheet, sheet.project, root.forges());
    for shown in &shown {
        b.open(Role::Item);
        b.open(Role::Check);
        b.name(&forge::words::full_name(shown.row.owner().login(), shown.row.name()));
        b.bind(bindings.insert(Meaning::Sheet(Act::Pick { id: shown.row.id(), on: !shown.picked })));
        if shown.picked {
            b.state(State::Pressed);
        }
        match shown.mark {
            sheet::Mark::InProject | sheet::Mark::Copying => b.state(State::Disabled),
            sheet::Mark::None | sheet::Mark::Private => {}
        }
        b.close();
        let mark = match shown.mark {
            sheet::Mark::None => None,
            sheet::Mark::Private => Some(words::say(w, Word::Private, &[])),
            sheet::Mark::InProject => Some(words::say(w, Word::InProject, &[Hole::Name(project.into())])),
            sheet::Mark::Copying => Some(words::say(w, Word::Copying, &[])),
        };
        if let Some(mark) = mark {
            b.text(Run::Emphasis, &mark);
        }
        b.close();
    }
    if shown.is_empty() {
        b.open(Role::Item);
        b.style(look::CONNECT_NONE);
        let word = if sheet.more {
            Word::Reading
        } else if sheet.find.text.trim_ascii().is_empty() {
            Word::NoneReachable
        } else {
            Word::NoneFound
        };
        let holes =
            if word == Word::NoneFound { Some([Hole::Name(sheet.find.text.trim_ascii().into())]) } else { None };
        words::text(
            w,
            b,
            word,
            match &holes {
                Some(holes) => holes,
                None => &[],
            },
        );
        b.close();
    }
    b.close();
    let count = match sheet::count(sheet, root.forges()) {
        sheet::Count::Hidden => None,
        sheet::Count::SoFar(n) => Some(words::say(w, Word::SoFar, &[Hole::Number(u64::from(n))])),
        sheet::Count::Searching(n) => Some(words::say(w, Word::Searching, &[Hole::Number(u64::from(n))])),
        sheet::Count::All(n) => Some(words::say(w, Word::All, &[Hole::Number(u64::from(n))])),
        sheet::Count::Found { found, of } => {
            Some(words::say(w, Word::Found, &[Hole::Number(u64::from(found)), Hole::Number(u64::from(of))]))
        }
        sheet::Count::Suspended => Some(words::say(w, Word::Suspended, &[])),
        sheet::Count::Full => Some(words::say(w, Word::Full, &[])),
    };
    if let Some(mut count) = count {
        if !sheet.picks.is_empty() {
            count =
                bytes::concat(&[&count, &words::say(w, Word::Picked, &[Hole::Number(u64::from(sheet.picks.len()))])]);
        }
        b.open(Role::Paragraph);
        b.style(look::CONNECT_COUNT);
        b.text(Run::Plain, &count);
        b.close();
    }
}
fn not_listed(
    b: &mut Builder,
    root: &Domain,
    sheet: &sheet::Sheet,
    w: &Words,
    pages: &mut crate::bindings::Bindings,
    bindings: &mut Bindings,
) {
    b.open(Role::Disclosure);
    b.name(&words::say(w, Word::NotListed, &[]));
    if !sheet.more && !sheet.find.text.is_empty() && sheet::shown(sheet, sheet.project, root.forges()).is_empty() {
        b.state(State::Expanded);
    }
    b.open(Role::Slot);
    b.open(Role::Slot);
    b.style(look::CONNECT_PAIR);
    field(
        b,
        &sheet.not_listed.0,
        Word::Owner,
        Meaning::Sheet(Act::NotListed { owner: Some(Box::default()), name: None }),
        w,
        bindings,
    );
    field(
        b,
        &sheet.not_listed.1,
        Word::Name,
        Meaning::Sheet(Act::NotListed { owner: None, name: Some(Box::default()) }),
        w,
        bindings,
    );
    b.close();
    if let Some(row) = sheet::row(root.forges(), sheet.forge)
        && let Some(remedy) = sheet::remedy(sheet, row)
    {
        match remedy {
            sheet::Remedy::Grant { owner, name, page } => {
                let name = words::say(w, Word::Grant, &[Hole::Name(forge::words::full_name(&owner, &name))]);
                if page.is_empty() {
                    crate::cards::text(b, Role::Paragraph, &name);
                } else {
                    b.open(Role::Link);
                    b.name(&name);
                    b.external(&page);
                    b.text(Run::Plain, &name);
                    b.close();
                }
            }
            sheet::Remedy::NotInstalled { owner } => {
                text(b, w, Role::Paragraph, Word::NotInstalled, &[Hole::Name(owner)]);
                crate::bindings::link(
                    b,
                    &words::say(w, Word::InstallAnother, &[]),
                    pages.insert(pages::Act::Go { address: Address::Forges }),
                );
                b.close();
            }
            sheet::Remedy::Collaborator { login, owner, name } => text(
                b,
                w,
                Role::Paragraph,
                Word::Collaborator,
                &[Hole::Name(login), Hole::Name(forge::words::full_name(&owner, &name))],
            ),
        }
    }
    b.close();
    b.close();
}
fn more(b: &mut Builder, sheet: &sheet::Sheet, form: &forge::Form, w: &Words, bindings: &mut Bindings) {
    let role = match form.role {
        forge::Role::Owned | forge::Role::Adopted => Word::OwnedSummary,
        forge::Role::Fork => Word::ForkSummary,
        forge::Role::Context => Word::ContextSummary,
    };
    let summary = bytes::concat(&[
        b"\xc2\xb7 ",
        &words::say(w, role, &[]),
        &words::say(w, if form.ci.on { Word::WithCi } else { Word::WithoutCi }, &[]),
    ]);
    b.open(Role::Disclosure);
    b.name(&words::say(w, Word::More, &[]));
    b.description(&summary);
    b.open(Role::Slot);
    b.open(Role::Paragraph);
    b.style(look::CONNECT_TITLE);
    words::text(w, b, Word::What, &[]);
    b.close();
    b.open(Role::ChoiceGroup);
    b.style(look::CONNECT_HOW);
    b.name(&words::say(w, Word::Role, &[]));
    for (role, word, detail) in [
        (forge::Role::Owned, Word::OwnedSummary, Word::OwnedChoice),
        (forge::Role::Fork, Word::ForkSummary, Word::ForkChoice),
        (forge::Role::Context, Word::ContextSummary, Word::ContextChoice),
    ] {
        b.open(Role::Slot);
        b.open(Role::Choice);
        b.name(&words::say(w, word, &[]));
        if form.role == role {
            b.state(State::Selected);
        }
        b.bind(bindings.insert(Meaning::Role(role)));
        b.close();
        text(b, w, Role::Text, detail, &[]);
        b.close();
    }
    b.close();
    b.open(Role::Slot);
    b.style(look::CONNECT_PAIR);
    for (field_id, word) in [(forge::Text::Prefix, Word::Prefix), (forge::Text::Landing, Word::Landing)] {
        field(b, form.texts.get(field_id.index()).expect("six fields"), word, Meaning::Field(field_id), w, bindings);
    }
    b.close();
    field(
        b,
        form.texts.get(forge::Text::Checks.index()).expect("six fields"),
        Word::Checks,
        Meaning::Field(forge::Text::Checks),
        w,
        bindings,
    );
    b.open(Role::Slot);
    b.style(look::CONNECT_HOW);
    b.open(Role::Slot);
    check(b, form.ci, Word::Ci, Meaning::Ci, w, bindings);
    b.close();
    let (button, _) = sheet::button(sheet, form, &forge::forges::State::Unknown);
    let single = match button {
        sheet::Button::Adopt { count, .. } | sheet::Button::Copy { count, .. } => count <= 1,
        sheet::Button::Make { .. } => true,
    };
    if single {
        b.open(Role::Slot);
        check(b, form.home, Word::Home, Meaning::Home, w, bindings);
        b.close();
    }
    b.close();
    b.close();
    b.close();
}
fn field(b: &mut Builder, field: &forge::Field, word: Word, meaning: Meaning, w: &Words, bindings: &mut Bindings) {
    b.open(Role::Slot);
    b.open(Role::TextField);
    b.style(look::SETTINGS_TEXT);
    b.name(&words::say(w, word, &[]));
    if word == Word::Find {
        b.placeholder(&words::say(w, word, &[]));
    }
    b.value(&field.text, field.written);
    b.bind(bindings.insert(meaning));
    if field.broken.is_some() {
        b.state(State::Invalid);
    }
    b.close();
    broken(b, field.broken, w);
    b.close();
}
fn check(b: &mut Builder, check: forge::Check, word: Word, meaning: Meaning, w: &Words, bindings: &mut Bindings) {
    b.open(Role::Check);
    b.name(&words::say(w, word, &[]));
    b.bind(bindings.insert(meaning));
    if check.on {
        b.state(State::Pressed);
    }
    if check.broken.is_some() {
        b.state(State::Invalid);
    }
    b.close();
    broken(b, check.broken, w);
}
fn broken(b: &mut Builder, broken: Option<forge::Broken>, w: &Words) {
    if let Some((word, holes)) = forge::words::broken(broken) {
        text(b, w, Role::Status, word, &holes);
    }
}
fn button_name(button: &sheet::Button, w: &Words) -> Box<[u8]> {
    match button {
        sheet::Button::Adopt { count: 0, .. } => words::say(w, Word::Adopt, &[]),
        sheet::Button::Adopt { count: 1, first } => words::say(w, Word::AdoptOne, &[Hole::Name(first.clone())]),
        sheet::Button::Adopt { count, .. } => words::say(w, Word::AdoptMany, &[Hole::Number(u64::from(*count))]),
        sheet::Button::Make { full } => {
            if full.starts_with(b"/") || full.ends_with(b"/") {
                words::say(w, Word::MakeIt, &[])
            } else {
                words::say(w, Word::MakeOne, &[Hole::Name(full.clone())])
            }
        }
        sheet::Button::Copy { count: 0, .. } => words::say(w, Word::Copy, &[]),
        sheet::Button::Copy { count: 1, first } => words::say(w, Word::CopyOne, &[Hole::Name(first.clone())]),
        sheet::Button::Copy { count, .. } => words::say(w, Word::CopyMany, &[Hole::Number(u64::from(*count))]),
    }
}
fn reading(
    b: &mut Builder,
    sheet: &sheet::Sheet,
    w: &Words,
    pages: &mut crate::bindings::Bindings,
    bindings: &mut Bindings,
) {
    let sheet::Reading::Refused(why) = &sheet.reading else { return };
    let number = Hole::Number(u64::from(sheet.forge.unwrap_or(0)));
    let (word, holes, retry): (Word, Box<[Hole]>, bool) = match why {
        sheet::Refused::NotConnected => (Word::ReadNotConnected, Box::new([number]), false),
        sheet::Refused::Busy => (Word::ReadBusy, Box::new([number]), true),
        sheet::Refused::NoAnswer => (Word::ReadNoAnswer, Box::new([number]), true),
        sheet::Refused::AdminMissing => (Word::ReadAdminMissing, Box::new([number]), false),
        sheet::Refused::NotAdmin { name } => (Word::ReadNotAdmin, Box::new([number, Hole::Name(name.clone())]), false),
        sheet::Refused::Role => (Word::ReadRole, Box::<[Hole]>::default(), false),
        sheet::Refused::Words(words) => {
            crate::cards::text(b, Role::Alert, words);
            return;
        }
    };
    text(b, w, Role::Alert, word, &holes);
    if retry {
        b.open(Role::Button);
        b.name(&words::say(w, Word::TryAgain, &[]));
        b.bind(bindings.insert(Meaning::Sheet(Act::NearEnd)));
        b.close();
    } else if *why != sheet::Refused::Role {
        crate::bindings::link(
            b,
            &w.say(crate::words::Word::SettingsForgesHeading, &[]),
            pages.insert(pages::Act::Go { address: Address::Forges }),
        );
        b.close();
    }
}

#[cfg(test)]
#[path = "../tests/sheet.rs"]
mod tests;
