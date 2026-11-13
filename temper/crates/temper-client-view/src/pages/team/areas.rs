//! Areas and Who routes (client/pages/team.md, 1).
use super::super::how::{Who, chip};
use super::paragraph;
use crate::{
    cards::text,
    frame::Bindings,
    look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::concat};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address,
    pages::{Act, team},
};
use temper_client_protocol_wire::v1 as w;

pub(super) fn name(words: &Words, name: &team::AreaName) -> Box<[u8]> {
    match name {
        team::AreaName::Named(name) => name.clone(),
        team::AreaName::AnythingElse => words.say(Word::TeamAnythingElse, &[]),
    }
}
pub(super) fn names(words: &Words, names: &List<team::AreaName>, last: &[u8]) -> Box<[u8]> {
    let mut joined = Box::default();
    for (index, item) in names.iter().enumerate() {
        let separator = if index == 0 {
            b"".as_slice()
        } else if index.saturating_add(1) == usize::try_from(names.len()).expect("names fit") {
            last
        } else {
            b", "
        };
        joined = concat(&[&joined, separator, &name(words, item)]);
    }
    joined
}
#[expect(clippy::manual_find, reason = "step code uses explicit loops, not closures")]
fn role(facts: &team::Facts, task: u64) -> Option<&team::Role> {
    for role in &facts.roles {
        if role.task == task {
            return Some(role);
        }
    }
    None
}
pub(super) fn holder_name(words: &Words, facts: &team::Facts, holder: &w::Holder) -> Box<[u8]> {
    let holder = match holder {
        w::Holder::Task(task) => team::Holder::Task(task.number()),
        w::Holder::Party(party) => team::Holder::Party(party.number()),
        w::Holder::Role(role) => team::Holder::Role(role.number()),
    };
    named_holder(words, Some(facts), &holder)
}
pub(crate) fn named_holder(words: &Words, facts: Option<&team::Facts>, holder: &team::Holder) -> Box<[u8]> {
    match holder {
        team::Holder::Task(task) => {
            if let Some(facts) = facts && let Some(role) = role(facts, *task) { return role.name.clone(); }
            words.say(Word::TeamTaskName, &[Hole::Number(*task)])
        }
        team::Holder::Party(party) => {
            if let Some(facts) = facts {
                for person in &facts.people {
                    if person.party == *party {
                        return if person.you { words.say(Word::TeamYou, &[]) } else { person.name.clone() };
                    }
                }
            }
            words.say(Word::MemberUnnamed, &[Hole::Number(*party)])
        }
        team::Holder::Role(role) => role_name(words, *role),
    }
}
fn role_name(words: &Words, number: u32) -> Box<[u8]> {
    let word = match number {
        0 => Word::TeamRoleOwners,
        1 => Word::TeamRoleMaintainers,
        2 => Word::TeamRoleMembers,
        3 => Word::TeamRoleObservers,
        4.. => return words.say(Word::TeamRolePeople, &[Hole::Number(u64::from(number))]),
    };
    words.say(word, &[])
}
fn reader(facts: &team::Facts, party: u64) -> bool {
    for person in &facts.people {
        if person.party == party {
            return person.you;
        }
    }
    false
}
fn plural(facts: &team::Facts, holder: &w::Holder) -> bool {
    match holder {
        w::Holder::Task(_) => false,
        w::Holder::Party(party) => reader(facts, party.number()),
        w::Holder::Role(_) => true,
    }
}
pub(super) fn lede(words: &Words, facts: &team::Facts) -> Box<[u8]> {
    let areas = facts.areas.as_ref().expect("areas lede has a document");
    let who = holder_name(words, facts, &areas.holder);
    let verb =
        words.say(if plural(facts, &areas.holder) { Word::TeamHand } else { Word::TeamHands }, &[Hole::Name(who)]);
    words.say(Word::TeamAreaLede, &[Hole::Name(verb)])
}
fn who(holder: &w::Holder) -> Who {
    match holder {
        w::Holder::Task(_) => Who::Agent,
        w::Holder::Party(_) | w::Holder::Role(_) => Who::Person,
    }
}
pub(super) fn absent(words: &Words, owner: &w::AreaOwner, project: &[u8]) -> Box<[u8]> {
    match owner {
        w::AreaOwner::Task(task) => words.say(Word::TeamOwnerEnded, &[Hole::Number(task.number())]),
        w::AreaOwner::Party(party) => {
            words.say(Word::TeamOwnerLeft, &[Hole::Number(party.number()), Hole::Name(project.into())])
        }
        w::AreaOwner::Charter(charter) => {
            words.say(Word::TeamOwnerNotStarted, &[Hole::Number(u64::from(charter.number()))])
        }
        w::AreaOwner::Role(role) => role_name(words, role.number()),
    }
}

pub(super) fn draw(
    b: &mut Builder,
    words: &Words,
    project: u32,
    project_name: &[u8],
    facts: &team::Facts,
    parts: &team::Parts,
    bindings: &mut Bindings,
) {
    if !parts.areas {
        return;
    }
    super::super::settings::section_head(b, &words.say(Word::TeamAreas, &[]));
    if parts.hand_area {
        b.open(Role::Button);
        b.style(look::GOAL_BTN);
        b.name(&words.say(Word::TeamHandArea, &[]));
        b.bind(bindings.insert(Act::HandArea(team::hand::Act::Open)));
        b.close();
    }
    b.close();
    if parts.areas_empty {
        paragraph(b, &words.say(Word::TeamNoAreas, &[]), look::SETTINGS_EMPTY);
    }
    if parts.areas_table {
        b.open(Role::Slot);
        b.style(look::TEAM_AREAS);
        b.open(Role::Table);
        b.style(look::HOW_TABLE);
        b.open(Role::Row);
        for word in [Word::TeamArea, Word::TeamCovers, Word::TeamOwner, Word::TeamTheirNow, Word::TeamThisWeek] {
            text(b, Role::ColumnHeader, &words.say(word, &[]));
        }
        b.close();
        for row in &parts.area_rows {
            b.open(Role::Row);
            if row.current {
                b.state(skein_ui::State::Current);
            }
            b.open(Role::Cell);
            let name = name(words, &row.name);
            b.name(&name);
            b.text(Run::Strong, &name);
            b.close();
            let covers = match row.name {
                team::AreaName::Named(_) => row.words.clone(),
                team::AreaName::AnythingElse => words.say(Word::TeamFitsNone, &[]),
            };
            text(b, Role::Cell, &covers);
            b.open(Role::Cell);
            if let Some(holder) = &row.holder {
                let name = holder_name(words, facts, holder);
                if let Some(task) = row.role {
                    crate::link(
                        b,
                        &name,
                        bindings.insert(Act::Go { address: Address::Team { project, role: Some(task) } }),
                    );
                    b.style(look::HOW_WHO_AGENT);
                    if row.current {
                        b.state(skein_ui::State::Current);
                    }
                    b.text(Run::Plain, &name);
                    b.close();
                } else {
                    chip(b, &name, who(holder));
                }
            } else if let Some(owner) = &row.owner {
                paragraph(b, &absent(words, owner, project_name), look::HOW_EMPTY);
            }
            b.close();
            now(b, words, facts, row);
            spend(b, words, facts, row);
            b.close();
        }
        b.close();
        b.close();
    }
    if parts.who_routes {
        routes(b, words, project, facts, bindings);
    }
    b.close();
}
fn empty(b: &mut Builder, words: &Words) {
    b.open(Role::Text);
    b.style(look::HOW_EMPTY);
    let value = words.say(Word::HowEmpty, &[]);
    b.name(&value);
    b.text(Run::Plain, &value);
    b.close();
}
fn now(b: &mut Builder, words: &Words, facts: &team::Facts, row: &team::AreaPart) {
    b.open(Role::Cell);
    if row.holder.is_none() {
        b.close();
        return;
    }
    let mut said = None;
    if let Some(task) = row.role
        && let Some(role) = role(facts, task)
        && let Some(now) = &role.now
        && !now.is_empty()
    {
        said = Some(now.clone());
    }
    if let Some(w::Holder::Party(party)) = &row.holder {
        for person in &facts.people {
            if person.party == party.number()
                && let Some(line) = crate::shared::person_line(words, person)
            {
                said = Some(line);
                break;
            }
        }
    }
    if let Some(said) = said {
        b.name(&said);
        b.text(Run::Plain, &said);
    } else {
        empty(b, words);
    }
    b.close();
}
fn spend(b: &mut Builder, words: &Words, facts: &team::Facts, row: &team::AreaPart) {
    b.open(Role::Cell);
    if row.holder.is_none() {
        b.close();
        return;
    }
    if let Some(task) = row.role
        && let Some(role) = role(facts, task)
    {
        if row.first {
            let spend = words.say(Word::TeamSpend, &[Hole::Number(role.spent), Hole::Number(role.budget)]);
            b.name(&spend);
            b.text(Run::Plain, &spend);
        } else {
            paragraph(b, &words.say(Word::TeamInRole, &[Hole::Name(role.name.clone())]), look::HOW_EMPTY);
        }
    } else {
        empty(b, words);
    }
    b.close();
}
fn routes(b: &mut Builder, words: &Words, project: u32, facts: &team::Facts, bindings: &mut Bindings) {
    let holder = &facts.areas.as_ref().expect("Who routes has areas").holder;
    b.open(Role::Paragraph);
    b.style(look::TEAM_ROUTING);
    chip(b, &holder_name(words, facts, holder), who(holder));
    let word = match holder {
        w::Holder::Task(_) => Word::TeamRoutesTask,
        w::Holder::Party(party) => {
            if reader(facts, party.number()) {
                Word::TeamRoutesYou
            } else {
                Word::TeamRoutesPerson
            }
        }
        w::Holder::Role(_) => Word::TeamRoutesPeople,
    };
    b.text(Run::Plain, &words.say(word, &[]));
    b.text(Run::Plain, b" ");
    let change = words.say(Word::TeamRoutingChange, &[]);
    crate::link(b, &change, bindings.insert(Act::Go { address: Address::Ask { project } }));
    b.text(Run::Plain, &change);
    b.close();
    b.close();
}
