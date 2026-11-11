//! Team's page (temper/docs/design/client/pages/team.md, 1;
//! temper/docs/design/client/web/look.md, Team's look).
use crate::{
    cards::text,
    look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract::HeldRole;
use skein_ui::{Builder, Role, Run, Style, phrases::Hole};
use temper_client_domain::{Domain, pages::team};

pub(crate) fn main(b: &mut Builder, root: &Domain, w: &Words, project: u32) {
    let name = match super::project_name(root.pages().me(root.client()), project) {
        Some(name) => name,
        None => w.say(Word::Project, &[Hole::Number(u64::from(project))]),
    };
    draw(b, w, &name, &team::shown(root, project));
}

/// The main column from what Team shows.
pub(crate) fn draw(b: &mut Builder, w: &Words, project_name: &[u8], shown: &team::Shown) {
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
        team::Shown::Reading => {}
        team::Shown::Unreadable => {
            paragraph(b, &w.say(Word::TeamUnreadable, &[]), look::SETTINGS_EMPTY);
        }
        team::Shown::Read(facts) => {
            if team::parts(facts).people {
                b.open(Role::Slot);
                b.style(look::TEAM_PEOPLE);
                for person in &facts.people {
                    person_box(b, w, person);
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

fn person_box(b: &mut Builder, w: &Words, person: &team::Person) {
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
