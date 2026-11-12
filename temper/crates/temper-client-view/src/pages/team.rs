//! Team's page (temper/docs/design/client/pages/team.md, 1;
//! temper/docs/design/client/web/look.md, Team's look).
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
    if let team::Shown::Read(facts) = shown {
        text(b, Role::Paragraph, &lede(w, facts));
    }
    b.close();
    if let Some(facts) = shown.facts() {
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
                    person_box(b, w, person, parts.signed(person.party));
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

fn person_box(b: &mut Builder, w: &Words, person: &team::Person, signed: Option<&[u8]>) {
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
    paragraph(b, &role, look::TEAM_ROLE);
    if let Some(waits) = person.waits {
        let whom = if person.you { w.say(Word::TeamWhomYou, &[]) } else { person.name.clone() };
        paragraph(b, &line(w, &waits, whom), look::TEAM_LINE);
    }
    if let Some(signed) = signed {
        paragraph(b, signed, look::TEAM_LINE);
    }
    b.close();
}

fn line(w: &Words, waits: &team::Waits, whom: Box<[u8]>) -> Box<[u8]> {
    let total = waits.total();
    if total == 0 {
        return w.say(Word::TeamNothing, &[Hole::Name(whom)]);
    }
    let mut kind = None;
    let mut mixed = false;
    for (count, one, many) in [
        (waits.questions, Word::TeamQuestion, Word::TeamQuestions),
        (waits.reviews, Word::TeamReview, Word::TeamReviews),
        (waits.proposals, Word::TeamProposal, Word::TeamProposals),
        (waits.held, Word::TeamHeldTask, Word::TeamHeldTasks),
        (waits.person_tasks, Word::TeamPersonTask, Word::TeamPersonTasks),
    ] {
        if count > 0 {
            mixed |= kind.is_some();
            kind = Some(if total == 1 { one } else { many });
        }
    }
    let kind = if mixed { Word::TeamThings } else { kind.expect("a positive total has a kind") };
    let number = if waits.cut {
        w.say(Word::TeamCountCut, &[Hole::Number(u64::from(total))])
    } else if total == 1 {
        w.say(Word::TeamOne, &[])
    } else {
        match super::home::number_word(total) {
            Some(word) => w.say(word, &[]),
            None => w.say(Word::TeamCount, &[Hole::Number(u64::from(total))]),
        }
    };
    w.say(
        if total == 1 { Word::TeamWaits } else { Word::TeamWait },
        &[Hole::Name(number), Hole::Name(w.say(kind, &[])), Hole::Name(whom)],
    )
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
    if !parts.standing_roles {
        return;
    }
    super::settings::section_head(b, &w.say(Word::TeamStanding, &[]));
    b.close();
    b.open(Role::List);
    b.style(look::TEAM_ROWS);
    for role in &facts.roles {
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
        if let Some(now) = &role.now
            && !now.is_empty()
        {
            paragraph(b, now, look::TEAM_LINE);
        }
        text(b, Role::Text, &w.say(Word::TeamSpend, &[Hole::Number(role.spent), Hole::Number(role.budget)]));
        b.close();
    }
    b.close();
    if parts.roles_cut {
        paragraph(b, &w.say(Word::TeamRolesCut, &[]), look::SETTINGS_EMPTY);
    }
    b.close();
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
#[expect(clippy::too_many_arguments, reason = "card drawing joins project links, facts and build accumulators")]
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
            paragraph(
                b,
                &w.say(Word::TeamSince, &[Hole::Name(w.time_date(made, times.now)), Hole::Name(requester)]),
                look::SETTINGS_EMPTY,
            );
        }
        if let Some(hold) = &facts.hold {
            crate::cards::hold(b, facts.task, &hold.reason, w);
        }
    }
    b.close();
    if parts.pause || parts.resume {
        b.open(Role::Slot);
        b.style(look::TEAM_ACTS);
        b.open(Role::Button);
        b.style(look::GOAL_BTN);
        b.name(&w.say(if parts.pause { Word::Pause } else { Word::Resume }, &[]));
        b.bind(bindings.insert(if parts.pause { Act::Pause } else { Act::Resume }));
        b.close();
        b.close();
    }
    b.close();
    if (parts.now || parts.rules || parts.right)
        && let Some(facts) = facts
    {
        b.open(Role::Slot);
        b.style(look::TEAM_COLUMNS);
        if parts.now || parts.rules {
            b.open(Role::Slot);
            b.style(look::TEAM_LEFT);
            if parts.now
                && let Some(now) = &facts.now
            {
                draw_now(b, w, now, times);
            }
            if parts.rules
                && let Some(rules) = &facts.rules
            {
                wakes::rules(b, w, project, rules, bindings);
            }
            b.close();
        }
        if parts.right {
            b.open(Role::Slot);
            b.style(look::TEAM_RIGHT);
            if parts.wakes {
                wakes::draw(b, w, facts, parts.wakes_cut, bindings, times);
            }
            if parts.sessions {
                sessions::draw(b, w, &facts.sessions);
            }
            b.close();
        }
        b.close();
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
    b.close();
}
