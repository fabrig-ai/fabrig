//! A role's or a rule's page (temper/docs/design/client/pages/README.md).
use super::settings::{self, Tab};
use crate::{
    bindings::Bindings,
    look,
    words::{Word, Words},
};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{Act, roles::Read},
};
use temper_client_protocol_wire::v1 as w;

pub fn main(b: &mut Builder, root: &Domain, words: &Words, pages: &mut Bindings, project: u32) {
    settings::main(b);
    settings::open(b, root, words, pages, project, Tab::HowWorkRuns);
    if !settings::missing(b, root, words, pages, project) {
        b.open(Role::Slot);
        if words.shell == crate::words::Shell::Browser {
            b.open(Role::Slot);
            b.style(look::SETTINGS_PATH);
            b.open(Role::Navigation);
            b.style(look::CRUMBS);
            let name = words.say(Word::SettingsHowTab, &[]);
            crate::bindings::link(b, &name, pages.insert(Act::Go { address: Address::Settings { project } }));
            b.style(look::BACK);
            b.text(Run::Plain, &name);
            b.close();
            b.close();
            b.close();
        }
        let read = match root.pages().roles() {
            Some(page) => page.read(),
            None => Read::Reading,
        };
        match read {
            Read::Drawn(w::View::Charter(view)) => card(b, view.card()),
            Read::Drawn(w::View::Rule(view)) => card(b, view.card()),
            Read::Reading => message(b, root.pages().address(), words, true),
            Read::Unreadable
            | Read::Drawn(
                w::View::Roles(_) | w::View::NotDrawn(_) | w::View::Way(_) | w::View::Path(_) | w::View::Proposal(_),
            ) => {
                message(b, root.pages().address(), words, false);
            }
        }
        b.close();
    }
    b.close();
    b.close();
}
fn card(b: &mut Builder, card: &w::Card) {
    b.open(Role::Group);
    b.style(look::VIEW_CARD);
    let mut name = skein_lib::bytes::copy_of(card.name());
    if let Some(first) = name.first_mut() {
        first.make_ascii_uppercase();
    }
    b.open(Role::Heading { level: 1 });
    b.style(look::SETTINGS_HEADLINE);
    b.name(&name);
    b.text(Run::Plain, &name);
    b.close();
    b.open(Role::Paragraph);
    b.style(look::VIEW_MUTED);
    b.name(card.who());
    b.text(Run::Plain, card.who());
    b.close();
    crate::views::facts(b, card);
    b.close();
}
fn message(b: &mut Builder, address: &Address, words: &Words, reading: bool) {
    let (word, holes, count) = match address {
        Address::Role { charter, .. } => (
            if reading { Word::RoleReading } else { Word::RoleUnreadable },
            [Hole::Number(u64::from(*charter)), Hole::Number(0)],
            1,
        ),
        Address::Rule { kind, number, .. } => (
            if reading { Word::RoleRuleReading } else { Word::RoleRuleUnreadable },
            [Hole::Number(u64::from(*kind)), Hole::Number(u64::from(*number))],
            2,
        ),
        Address::AllHome
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Work { .. }
        // page-view: draw Remembered.
        | Address::Told
        | Address::Remembered
        | Address::Members
        | Address::Limits
        | Address::Accounts
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. }
        | Address::Welcome { .. }
        | Address::Chat { .. }
        | Address::Talk { .. }
        | Address::Task { .. }
        | Address::Home { .. }
        | Address::Goals { .. }
        | Address::Changes { .. }
        | Address::Team { .. }
        | Address::Settings { .. }
        | Address::Repositories { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Ask { .. } | Address::Tell { .. }
        | Address::Adopt { .. } => return,
    };
    b.open(Role::Paragraph);
    b.style(look::VIEW_MUTED);
    let text = words.say(word, holes.get(..count).expect("phrase holes"));
    b.name(&text);
    b.text(Run::Plain, &text);
    b.close();
}
