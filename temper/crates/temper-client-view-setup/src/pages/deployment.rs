//! Deployment's read-only pages (temper/docs/design/client/pages/README.md, Members and Limits).
use super::settings::{self, Tab};
use crate::{
    bindings::Bindings,
    look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_ui::{Builder, Role, Run, Style, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::{builders, members::Read},
};

fn open(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, tab: Tab) {
    settings::main(b);
    if let Some(project) = root.pages().project() {
        settings::open(b, root, w, bindings, project, tab);
    } else {
        let (title, lede) = match tab {
            Tab::Members => (Word::MembersTab, Word::MembersLede),
            Tab::Limits => (Word::LimitsTab, Word::LimitsLede),
            Tab::HowWorkRuns | Tab::Repositories | Tab::Forges | Tab::Accounts | Tab::Models | Tab::Builders => {
                unreachable!("deployment pages open Members or Limits")
            }
        };
        settings::open_alone(b, &w.say(title, &[]), &w.say(lede, &[]));
    }
}
fn section(b: &mut Builder, title: &[u8]) {
    settings::section_head(b, title);
    b.close();
}
fn text(b: &mut Builder, role: Role, words: &[u8], style: Style) {
    b.open(role);
    b.style(style);
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
fn role(w: &Words, held: c::HeldRole) -> Box<[u8]> {
    let word = match held {
        c::HeldRole::Owner => Word::MemberOwner,
        c::HeldRole::Maintainer => Word::MemberMaintainer,
        c::HeldRole::Member => Word::MemberMember,
        c::HeldRole::Observer => Word::MemberObserver,
        c::HeldRole::Policy { role } => return w.say(Word::MemberPolicy, &[Hole::Number(u64::from(role))]),
    };
    w.say(word, &[])
}

pub(crate) fn members(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    open(b, root, w, bindings, Tab::Members);
    if let Some(me) = root.pages().me(root.client()) {
        section(b, &w.say(Word::MembersDeployment, &[]));
        let line = match me.deployment_role {
            Some(c::HeldRole::Owner) => w.say(Word::MembersOwner, &[]),
            None => w.say(Word::MembersNoRole, &[]),
            Some(held) => w.say(Word::MembersHeldRole, &[Hole::Name(role(w, held))]),
        };
        crate::cards::text(b, Role::Paragraph, &line);
        b.close();
        for project in &me.projects {
            let read = match root.pages().members() {
                Some(page) => page.read(project.project),
                None => &Read::Unread,
            };
            let title = match read {
                Read::Project(doc) => w.say(
                    Word::MembersCount,
                    &[Hole::Name(project.name.bytes().into()), Hole::Number(u64::from(doc.members.len()))],
                ),
                Read::Unread | Read::Refused => project.name.bytes().into(),
            };
            section(b, &title);
            match read {
                Read::Unread => {}
                Read::Refused => crate::cards::text(b, Role::Paragraph, &w.say(Word::MembersRefused, &[])),
                Read::Project(doc) => project_members(b, w, doc),
            }
            b.close();
        }
        text(b, Role::Paragraph, &w.say(Word::MembersFacts, &[]), look::SETTINGS_FACTS);
    }
    b.close();
    b.close();
}
fn project_members(b: &mut Builder, w: &Words, doc: &c::Project) {
    b.open(Role::Slot);
    b.style(look::SETTINGS_LINES);
    for member in &doc.members {
        b.open(Role::Slot);
        b.open(Role::Slot);
        let name = if !member.name.bytes().is_empty() {
            member.name.bytes().into()
        } else if !member.login.bytes().is_empty() {
            member.login.bytes().into()
        } else {
            w.say(Word::MemberUnnamed, &[Hole::Number(member.party)])
        };
        crate::cards::text(b, Role::Text, &name);
        let kind = match member.kind {
            c::PartyKind::Person => Word::MemberPerson,
            c::PartyKind::Service => Word::MemberService,
        };
        text(b, Role::Text, &w.say(kind, &[]), look::SETTINGS_DETAIL);
        b.close();
        text(b, Role::Text, &role(w, member.role), look::SETTINGS_WHEN);
        b.close();
    }
    b.close();
    if doc.cut {
        text(b, Role::Paragraph, &w.say(Word::MembersCut, &[]), look::SETTINGS_FACTS);
    }
}

pub(crate) fn limits(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings) {
    open(b, root, w, bindings, Tab::Limits);
    let read = match root.pages().builders() {
        Some(page) => page.read(root.client()),
        None => builders::Read::Unknown,
    };
    match read {
        builders::Read::NoRole => crate::cards::text(b, Role::Paragraph, &w.say(Word::LimitsNoRole, &[])),
        builders::Read::Unknown => {
            section(b, &w.say(Word::LimitsTab, &[]));
            b.close();
        }
        builders::Read::Hosts(doc) => capacities(b, root, w, doc),
    }
    b.close();
    b.close();
}
fn line(b: &mut Builder, title: &[u8], value: &[u8]) {
    b.open(Role::Slot);
    crate::cards::text(b, Role::Text, title);
    text(b, Role::Text, value, look::SETTINGS_WHEN);
    b.close();
}
fn capacities(b: &mut Builder, root: &Domain, w: &Words, doc: &c::Deployment) {
    section(b, &w.say(Word::LimitsCapacities, &[]));
    b.open(Role::Slot);
    b.style(look::SETTINGS_LINES);
    for (name, capacity, waiting) in [
        (Word::LimitsTasks, &doc.tasks, 0),
        (Word::SettingsBuildersTab, &doc.workers, 0),
        (Word::LimitsAttempts, &doc.attempts_capacity, doc.waiting_attempts),
        (Word::LimitsProposals, &doc.proposals, 0),
    ] {
        let mut value =
            w.say(Word::LimitsUsed, &[Hole::Number(u64::from(capacity.used)), Hole::Number(u64::from(capacity.limit))]);
        if waiting > 0 {
            value = w.say(
                Word::LimitsWaiting,
                &[
                    Hole::Number(u64::from(capacity.used)),
                    Hole::Number(u64::from(capacity.limit)),
                    Hole::Number(u64::from(waiting)),
                ],
            );
        }
        line(b, &w.say(name, &[]), &value);
    }
    let used = megabytes(w, doc.store.bytes);
    let value = if doc.store.mark == 0 {
        used
    } else {
        w.say(Word::LimitsStoreFill, &[Hole::Name(used), Hole::Name(megabytes(w, doc.store.mark))])
    };
    line(b, &w.say(Word::LimitsStore, &[]), &value);
    b.close();
    b.close();
    section(b, &w.say(Word::LimitsProjects, &[]));
    b.open(Role::Slot);
    b.style(look::SETTINGS_LINES);
    for project in &doc.projects {
        let name = match super::project_name(root.pages().me(root.client()), project.project) {
            Some(name) => name,
            None => w.say(Word::LimitsProject, &[Hole::Number(u64::from(project.project))]),
        };
        line(
            b,
            &name,
            &w.say(
                Word::LimitsProjectTasks,
                &[Hole::Number(u64::from(project.tasks.used)), Hole::Number(u64::from(project.tasks.limit))],
            ),
        );
    }
    b.close();
    if doc.cut {
        text(b, Role::Paragraph, &w.say(Word::LimitsCut, &[]), look::SETTINGS_FACTS);
    }
    text(b, Role::Paragraph, &w.say(Word::LimitsFacts, &[]), look::SETTINGS_FACTS);
    b.close();
}
fn megabytes(w: &Words, bytes: u64) -> Box<[u8]> {
    if bytes < 10_000_000 {
        let tenths = bytes
            .checked_div(100_000)
            .expect("nonzero divisor")
            .checked_add(u64::from(bytes.checked_rem(100_000).expect("nonzero divisor") >= 50_000))
            .expect("megabyte tenths fit");
        if tenths < 100 {
            return w.say(
                Word::LimitsDecimalMegabytes,
                &[
                    Hole::Number(tenths.checked_div(10).expect("nonzero divisor")),
                    Hole::Number(tenths.checked_rem(10).expect("nonzero divisor")),
                ],
            );
        }
    }
    let whole = bytes
        .checked_div(1_000_000)
        .expect("nonzero divisor")
        .checked_add(u64::from(bytes.checked_rem(1_000_000).expect("nonzero divisor") >= 500_000))
        .expect("megabytes fit");
    w.say(Word::LimitsMegabytes, &[Hole::Number(whole)])
}
