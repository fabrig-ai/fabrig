//! Team's page (temper/docs/design/client/pages/team.md, 1;
//! temper/docs/design/client/web/look.md, Team's look).
mod areas;
pub(crate) use hand::sheet as hand_sheet;
mod hand;
mod sessions;
pub(crate) mod wakes;
use crate::{
    cards::text,
    frame::Bindings,
    look,
    words::{TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract::HeldRole;
use skein_ui::{Builder, Role, Run, Style, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{Act, team},
};

pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    project: u32,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    let name = match super::project_name(root.pages().me(root.client()), project) {
        Some(name) => name,
        None => w.say(Word::Project, &[Hole::Number(u64::from(project))]),
    };
    draw(b, w, &name, &team::shown_at(root, project, times.now), project, bindings, times);
}

/// The main column from what Team shows.
pub(crate) fn draw(
    b: &mut Builder,
    w: &Words,
    project_name: &[u8],
    shown: &team::Shown,
    project: u32,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::TEAM);
    b.open(Role::Slot);
    b.style(look::TEAM_INTRO);
    text(b, Role::Paragraph, &w.say(Word::TeamKicker, &[Hole::Name(project_name.into())]));
    text(b, Role::Heading { level: 1 }, &w.say(Word::TeamHeadline, &[Hole::Name(project_name.into())]));
    if let Some(facts) = shown.facts() {
        let people = match shown {
            team::Shown::Read(_) => lede(w, facts),
            team::Shown::Reading | team::Shown::Unreadable | team::Shown::PeopleUnread { .. } => Box::default(),
        };
        let lede = if team::parts(facts).lede_areas {
            let separator = if people.is_empty() { b"".as_slice() } else { b" " };
            skein_lib::bytes::concat(&[&people, separator, &areas::lede(w, facts)])
        } else {
            people
        };
        if !lede.is_empty() {
            text(b, Role::Paragraph, &lede);
        }
    }
    b.close();
    if let Some(facts) = shown.facts() {
        let parts = team::parts(facts);
        areas::draw(b, w, project, project_name, facts, &parts, bindings);
        standing(b, w, project, facts, bindings, times);
    }
    super::settings::section_head(b, &w.say(Word::TeamPeople, &[]));
    if let team::Shown::Read(facts) = shown
        && let Some(role) = team::parts(facts).shared
    {
        let word = match role {
            HeldRole::Owner => Word::TeamOwners,
            HeldRole::Maintainer => Word::TeamMaintainers,
            HeldRole::Member => Word::TeamMembers,
            HeldRole::Observer => Word::TeamObservers,
            HeldRole::Policy { role: _ } => unreachable!("a shared role is built-in"),
        };
        b.open(Role::Text);
        b.style(look::TEAM_SHARED);
        let shared = w.say(word, &[Hole::Name(project_name.into())]);
        b.name(&shared);
        b.text(Run::Plain, &shared);
        b.close();
    }
    b.close();
    match shown {
        team::Shown::Reading | team::Shown::PeopleUnread { failed: false, .. } => {}
        team::Shown::Unreadable | team::Shown::PeopleUnread { failed: true, .. } => {
            paragraph(b, &w.say(Word::TeamUnreadable, &[]), look::SETTINGS_EMPTY);
        }
        team::Shown::Read(facts) => {
            let parts = team::parts(facts);
            if parts.people {
                b.open(Role::Slot);
                b.style(look::TEAM_PEOPLE);
                for person in &facts.people {
                    person_box(b, w, person, &parts);
                }
                b.close();
            }
        }
    }
    b.close();
    b.close();
}

fn lede(w: &Words, facts: &team::Facts) -> Box<[u8]> {
    let mut you = false;
    for person in &facts.people {
        you |= person.you;
    }
    let count = facts.people.len();
    if facts.cut {
        return w.say(if you { Word::TeamOfYouCut } else { Word::TeamPeopleCut }, &[Hole::Number(u64::from(count))]);
    }
    if count == 1 {
        return w.say(if you { Word::TeamJustYou } else { Word::TeamOnePerson }, &[]);
    }
    let count = match count {
        2 => w.say(Word::HomeTwo, &[]),
        3 => w.say(Word::HomeThree, &[]),
        4 => w.say(Word::HomeFour, &[]),
        0 | 1 | 5.. => w.say(Word::TeamCount, &[Hole::Number(u64::from(count))]),
    };
    w.say(if you { Word::TeamOfYou } else { Word::TeamPeopleCount }, &[Hole::Name(count)])
}

fn person_box(b: &mut Builder, w: &Words, person: &team::Person, parts: &team::Parts) {
    b.open(Role::Slot);
    b.style(look::TEAM_PERSON);
    let name = if person.you { w.say(Word::TeamYou, &[]) } else { person.name.clone() };
    b.open(Role::Text);
    b.name(&name);
    b.text(Run::Strong, &name);
    b.close();
    let role = match person.role {
        HeldRole::Owner => w.say(Word::TeamOwner, &[]),
        HeldRole::Maintainer => w.say(Word::TeamMaintainer, &[]),
        HeldRole::Member => w.say(Word::TeamMember, &[]),
        HeldRole::Observer => w.say(Word::TeamObserver, &[]),
        HeldRole::Policy { role } => w.say(Word::TeamPolicy, &[Hole::Number(u64::from(role))]),
    };
    let mut role = role;
    for (party, names) in &parts.box_areas {
        if *party == person.party {
            role = skein_lib::bytes::concat(&[&role, " · ".as_bytes(), &areas::names(w, names, b", ")]);
            break;
        }
    }
    paragraph(b, &role, look::TEAM_ROLE);
    if let Some(line) = crate::shared::person_line(w, person) {
        paragraph(b, &line, look::TEAM_LINE);
    }
    if let Some(signed) = parts.signed(person.party) {
        paragraph(b, signed, look::TEAM_LINE);
    }
    owner_follows(b, w, parts, team::follows::Owner::Person(person.party));
    b.close();
}

fn paragraph(b: &mut Builder, value: &[u8], style: Style) {
    b.open(Role::Paragraph);
    b.style(style);
    b.name(value);
    b.text(Run::Plain, value);
    b.close();
}

fn standing(b: &mut Builder, w: &Words, project: u32, facts: &team::Facts, bindings: &mut Bindings, times: &mut Times) {
    let parts = team::parts(facts);
    if parts.standing_roles {
        super::settings::section_head(b, &w.say(Word::TeamStanding, &[]));
        b.close();
        if !parts.standing.is_empty() {
            b.open(Role::List);
            b.style(look::TEAM_ROWS);
            for role in &facts.roles {
                if !parts.standing.as_slice().contains(&role.task) {
                    continue;
                }
                b.open(Role::Item);
                b.style(look::TEAM_ROW);
                crate::link(
                    b,
                    &role.name,
                    bindings.insert(Act::Go { address: Address::Team { project, role: Some(role.task) } }),
                );
                b.style(look::HOW_WHO_AGENT);
                if parts.card == Some(role.task) {
                    b.state(skein_ui::State::Current);
                }
                b.text(Run::Plain, &role.name);
                b.close();
                let owner = team::follows::Owner::Task(role.task);
                let mut has_follows = false;
                for follow in &parts.follows {
                    has_follows |= follow.owner == owner;
                }
                let now = match role.now.as_deref() {
                    Some(now) if !now.is_empty() => Some(now),
                    Some(_) | None => None,
                };
                if now.is_some() || has_follows {
                    b.open(Role::Slot);
                    if let Some(now) = now {
                        paragraph(b, now, look::TEAM_LINE);
                    }
                    owner_follows(b, w, &parts, owner);
                    b.close();
                }
                text(b, Role::Text, &w.say(Word::TeamSpend, &[Hole::Number(role.spent), Hole::Number(role.budget)]));
                b.close();
            }
            b.close();
        }
        if parts.roles_cut {
            paragraph(b, &w.say(Word::TeamRolesCut, &[]), look::SETTINGS_EMPTY);
        }
        b.close();
    }
    if let Some(task) = parts.card {
        for role in &facts.roles {
            if role.task == task {
                let detail = match facts.card.as_deref() {
                    Some(card) if card.task == role.task => Some(card),
                    Some(_) | None => None,
                };
                card(b, w, project, role, detail, &parts, bindings, times);
                break;
            }
        }
    }
}

/// The card frame; sibling parts join its left column and conditionally open its right column.
#[expect(
    clippy::too_many_arguments,
    clippy::too_many_lines,
    reason = "the card joins its ownership head, controls and independently read columns"
)]
fn card(
    b: &mut Builder,
    w: &Words,
    project: u32,
    role: &team::Role,
    facts: Option<&team::Card>,
    parts: &team::Parts,
    bindings: &mut Bindings,
    times: &mut Times,
) {
    b.open(Role::Slot);
    b.style(look::TEAM_CARD);
    b.key(super::inbox::card_key(jig_client::contract::ObjectKey::Escalation(role.task)));
    let name = if let Some(facts) = facts { &facts.name } else { &role.name };
    b.name(name);
    b.open(Role::Slot);
    b.style(look::TEAM_HEAD);
    b.open(Role::Slot);
    b.open(Role::Heading { level: 2 });
    b.name(name);
    crate::link(b, name, bindings.insert(Act::Go { address: Address::Task { task: role.task } }));
    b.text(Run::Plain, name);
    b.close();
    b.close();
    if let Some(facts) = facts {
        if let (Some(made), Some(requester)) = (facts.made, &facts.requester) {
            let requester = match requester {
                team::StartedBy::You => w.say(Word::TeamWhomYou, &[]),
                team::StartedBy::Member(name) => name.clone(),
                team::StartedBy::Requester(by) => {
                    let (word, number) = match by {
                        jig_client::contract::Requester::Person(n) => (Word::TeamStartedPerson, *n),
                        jig_client::contract::Requester::Task(n) => (Word::TeamStartedTask, *n),
                        jig_client::contract::Requester::Deployment { project } => {
                            (Word::TeamStartedDeployment, u64::from(*project))
                        }
                    };
                    w.say(word, &[Hole::Number(number)])
                }
            };
            let since = if parts.owns.is_empty() {
                w.say(Word::TeamSince, &[Hole::Name(w.time_date(made, times.now)), Hole::Name(requester)])
            } else {
                w.say(
                    Word::TeamOwnsSince,
                    &[
                        Hole::Name(areas::names(w, &parts.owns, b", and ")),
                        Hole::Name(w.time_date(made, times.now)),
                        Hole::Name(requester),
                    ],
                )
            };
            paragraph(b, &since, look::SETTINGS_EMPTY);
        }
        if let Some(hold) = &facts.hold {
            crate::cards::hold(b, facts.task, &hold.reason, w);
        }
    }
    b.close();
    if parts.talk || parts.pause || parts.resume || parts.give_areas > 0 {
        b.open(Role::Slot);
        b.style(look::TEAM_ACTS);
        if parts.talk {
            let talk = w.say(Word::TeamTalk, &[]);
            crate::link(b, &talk, bindings.insert(Act::Go { address: Address::Task { task: role.task } }));
            b.style(look::TEAM_TALK);
            b.text(Run::Plain, &talk);
            b.close();
        }
        if parts.pause || parts.resume {
            b.open(Role::Button);
            b.style(look::GOAL_BTN);
            b.name(&w.say(if parts.pause { Word::Pause } else { Word::Resume }, &[]));
            b.bind(bindings.insert(if parts.pause { Act::Pause } else { Act::Resume }));
            b.close();
        }
        if parts.give_areas > 0 {
            b.open(Role::Button);
            b.style(look::GOAL_BTN);
            b.name(&w.say(if parts.give_areas == 1 { Word::GiveArea } else { Word::GiveAreas }, &[]));
            b.bind(bindings.insert(Act::HandArea(team::hand::Act::GiveBack)));
            b.close();
        }
        b.close();
    }
    b.close();
    if parts.now || parts.rules || parts.right || !parts.card_follows.is_empty() {
        b.open(Role::Slot);
        b.style(look::TEAM_COLUMNS);
        if parts.now || parts.rules || !parts.card_follows.is_empty() {
            b.open(Role::Slot);
            b.style(look::TEAM_LEFT);
            if parts.now
                && let Some(facts) = facts
                && let Some(now) = &facts.now
            {
                draw_now(b, w, now, times);
            }
            card_follows(b, w, &parts.card_follows);
            if parts.rules
                && let Some(facts) = facts
                && let Some(rules) = &facts.rules
            {
                wakes::rules(b, w, project, rules, bindings);
            }
            b.close();
        }
        if parts.right
            && let Some(facts) = facts
        {
            b.open(Role::Slot);
            b.style(look::TEAM_RIGHT);
            if parts.wakes {
                wakes::draw(b, w, facts, parts.wakes_cut, bindings, times);
            }
            if parts.sessions {
                sessions::draw(b, w, &facts.sessions);
            }
            if parts.report
                && let Some(report) = &facts.report
            {
                draw_report(b, w, report);
            }
            b.close();
        }
        b.close();
    }
    b.close();
}
fn draw_report(b: &mut Builder, w: &Words, report: &team::Now) {
    b.open(Role::Region);
    b.style(look::TEAM_REPORT);
    let kicker = if let Some(changed) = report.changed {
        w.say(Word::TeamReportAt, &[Hole::Name(w.session_clock(changed))])
    } else {
        w.say(Word::TeamReport, &[])
    };
    paragraph(b, &kicker, look::TEAM_KICKER);
    draw_body(b, report);
    b.close();
}
fn follow_line(b: &mut Builder, w: &Words, follow: &team::follows::Follow) {
    paragraph(
        b,
        &w.say(Word::TeamFollow, &[Hole::Name(follow.source.clone()), Hole::Name(follow.status.clone())]),
        look::TEAM_ROLE,
    );
}
fn owner_follows(b: &mut Builder, w: &Words, parts: &team::Parts, owner: team::follows::Owner) {
    for follow in &parts.follows {
        if follow.owner == owner {
            follow_line(b, w, follow);
        }
    }
}
fn card_follows(b: &mut Builder, w: &Words, follows: &skein_lib::List<team::follows::Follow>) {
    if follows.is_empty() {
        return;
    }
    b.open(Role::Region);
    paragraph(b, &w.say(Word::TeamFollows, &[]), look::TEAM_KICKER);
    for follow in follows {
        follow_line(b, w, follow);
        for entry in &follow.entries {
            let name = w.say(
                Word::TeamFollowEntry,
                &[Hole::Name(entry.after.as_slice().into()), Hole::Name(entry.state.clone())],
            );
            b.open(Role::Paragraph);
            b.style(look::TEAM_ROLE);
            b.name(&name);
            b.text(Run::Code, &entry.after);
            b.text(
                Run::Plain,
                &w.say(Word::TeamFollowEntry, &[Hole::Name(Box::default()), Hole::Name(entry.state.clone())]),
            );
            b.close();
        }
        if follow.more > 0 {
            paragraph(b, &w.say(Word::TeamFollowsMore, &[Hole::Number(u64::from(follow.more))]), look::TEAM_ROLE);
        }
    }
    b.close();
}
fn draw_now(b: &mut Builder, w: &Words, now: &team::Now, times: &mut Times) {
    b.open(Role::Region);
    b.style(look::TEAM_NOW);
    let kicker = if let Some(changed) = now.changed {
        w.say(Word::TeamNowAt, &[Hole::Name(times.say(w, TimeForm::Clock, changed))])
    } else {
        w.say(Word::TeamNow, &[])
    };
    paragraph(b, &kicker, look::TEAM_KICKER);
    draw_body(b, now);
    b.close();
}
fn draw_body(b: &mut Builder, now: &team::Now) {
    let mut list = false;
    let mut start = 0;
    for end in 0..=now.body.len() {
        if end != now.body.len() && now.body.get(end) != Some(&b'\n') {
            continue;
        }
        let line = now.body.get(start..end).expect("Now line boundaries fit");
        start = end.saturating_add(1);
        if let Some(item) = line.strip_prefix(b"- ")
            && !item.is_empty()
        {
            if !list {
                b.open(Role::List);
                list = true;
            }
            text(b, Role::Item, item);
        } else {
            if list {
                b.close();
                list = false;
            }
            if !line.is_empty() {
                text(b, Role::Paragraph, line);
            }
        }
    }
    if list {
        b.close();
    }
    if now.cut {
        text(b, Role::Paragraph, "…".as_bytes());
    }
}
