//! A project's queues and recent landings (temper/docs/design/client/pages/README.md, Changes).
use crate::{
    frame::Bindings,
    look,
    words::{Shell, TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use skein_lib::{List, Wall};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::forge::words::ChangeWord as W;
use temper_client_domain::{
    Address, Domain,
    pages::{
        Act,
        changes::{From, State},
    },
};
use temper_client_protocol_wire::v1 as w;

#[derive(Debug)]
pub(crate) struct LinePart {
    pub words: Box<[u8]>,
    pub red: bool,
}
fn queued(panel: &w::ChangesPanel) -> u64 {
    let mut count = u64::from(panel.more());
    for queue in panel.queues() {
        count = count.checked_add(u64::from(queue.changes().len())).expect("wire rows fit");
        count = count.checked_add(u64::from(queue.in_flight().is_some())).expect("wire rows fit");
    }
    count
}
/// The one branch name among queues with rows, or the landing when none has rows.
fn branch_named(panel: &w::ChangesPanel) -> Option<&[u8]> {
    let mut name = None;
    for queue in panel.queues() {
        if !queue.changes().is_empty() || queue.in_flight().is_some() {
            if let Some(previous) = name
                && previous != queue.branch()
            {
                return None;
            }
            name = Some(queue.branch());
        }
    }
    let name = name.unwrap_or(panel.landing());
    if name.is_empty() { None } else { Some(name) }
}
fn branch_words(panel: &w::ChangesPanel, words: &Words, fallback: Word) -> Box<[u8]> {
    match branch_named(panel) {
        Some(branch) => branch.into(),
        None => words.say(fallback, &[]),
    }
}
/// The facts shared by Home and Changes (client/pages/README.md).
#[derive(Debug)]
pub(crate) struct LandingFacts<'a> {
    pub queued: u64,
    pub last: Option<Wall>,
    pub branch: Option<&'a [u8]>,
    pub red: &'a [&'a [u8]],
}
impl<'a> LandingFacts<'a> {
    pub(crate) fn panel(panel: &'a w::ChangesPanel, red: &'a [&'a [u8]]) -> Self {
        Self {
            queued: queued(panel),
            last: match panel.landed().get(0) {
                Some(last) => Some(Wall::from_nanos(last.at())),
                None => None,
            },
            branch: branch_named(panel),
            red,
        }
    }
}
/// The landing's line for Home and Changes, with each signal part marked.
pub(crate) fn landing_line(facts: &LandingFacts<'_>, words: &Words, times: &mut Times) -> Box<[LinePart]> {
    let mut parts = List::uncapped();
    let branch = match facts.branch {
        Some(branch) => branch.into(),
        None => words.say(Word::ChangesLanding, &[]),
    };
    if facts.queued > 0 {
        parts
            .push(LinePart { words: words.say(Word::ChangesQueued, &[Hole::Number(facts.queued)]), red: false })
            .expect("line fits");
        if let Some(last) = facts.last {
            parts
                .push(LinePart {
                    words: words.say(Word::ChangesLast, &[Hole::Name(times.say(words, TimeForm::Ago, last))]),
                    red: false,
                })
                .expect("line fits");
        }
    } else if let Some(last) = facts.last {
        parts
            .push(LinePart {
                words: words
                    .say(Word::ChangesMoved, &[Hole::Name(branch), Hole::Name(times.say(words, TimeForm::Ago, last))]),
                red: false,
            })
            .expect("line fits");
        parts.push(LinePart { words: words.say(Word::ChangesNothingQueued, &[]), red: false }).expect("line fits");
    } else {
        parts.push(LinePart { words: words.say(Word::ChangesNothingLanded, &[]), red: false }).expect("line fits");
    }
    for branch in facts.red {
        let branch = if branch.is_empty() { words.say(Word::ChangesLanding, &[]) } else { (*branch).into() };
        parts
            .push(LinePart { words: words.say(Word::ChangesRed, &[Hole::Name(branch)]), red: true })
            .expect("line fits");
    }
    parts.into_boxed()
}
pub(crate) fn landing_parts(b: &mut Builder, parts: &[LinePart], words: &Words) {
    for (index, part) in parts.iter().enumerate() {
        b.open(Role::Text);
        if part.red {
            b.style(look::CHANGES_RED);
        }
        if index > 0 {
            b.run(Run::Plain, &words.say(Word::ToolSeparator, &[]));
        }
        b.run(Run::Plain, &part.words);
        b.close();
    }
}
pub(crate) fn count_named(count: u64, words: &Words) -> Box<[u8]> {
    let word = match count {
        2 => Word::ChangesTwo,
        3 => Word::ChangesThree,
        4 => Word::ChangesFour,
        5 => Word::ChangesFive,
        6 => Word::ChangesSix,
        7 => Word::ChangesSeven,
        8 => Word::ChangesEight,
        9 => Word::ChangesNine,
        0 | 1 | 10..=u64::MAX => return skein_lib::Decimal::of(count).as_bytes().into(),
    };
    words.say(word, &[])
}
fn heading(panel: &w::ChangesPanel, words: &Words) -> Box<[u8]> {
    let branch = branch_words(panel, words, Word::ChangesLanding);
    match queued(panel) {
        0 => words.say(Word::ChangesNone, &[Hole::Name(branch)]),
        1 => words.say(Word::ChangesOne, &[Hole::Name(branch)]),
        count @ 2..=u64::MAX => {
            words.say(Word::ChangesMany, &[Hole::Name(count_named(count, words)), Hole::Name(branch)])
        }
    }
}
pub(crate) fn ordinal(number: u64, words: &Words) -> Box<[u8]> {
    let last = number.checked_rem(10).expect("nonzero divisor");
    let hundred = number.checked_rem(100).expect("nonzero divisor");
    let word = if (11..=13).contains(&hundred) {
        Word::ChangesOrdinalTh
    } else {
        match last {
            1 => Word::ChangesOrdinalSt,
            2 => Word::ChangesOrdinalNd,
            3 => Word::ChangesOrdinalRd,
            0 | 4..=u64::MAX => Word::ChangesOrdinalTh,
        }
    };
    words.say(word, &[Hole::Number(number)])
}
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    project: u32,
    times: &mut Times,
) {
    b.open(Role::Main);
    b.style(look::HOME);
    b.style(look::CHANGES);
    super::work::head(b, root, words, bindings, project, times);
    b.open(Role::Slot);
    b.style(look::HOME_INTRO);
    let state = match root.pages().changes() {
        Some(page) => Some(page.state()),
        None => None,
    };
    let panel = match state {
        Some(State::Shown(panel)) => panel,
        Some(State::Loading) | None => {
            b.close();
            b.close();
            return;
        }
        Some(State::Ended) => {
            b.open(Role::Paragraph);
            words.resource_text(b, temper_client_domain::forge::words::ResourceWord::Shown, &[]);
            b.close();
            b.close();
            b.close();
            return;
        }
    };
    text(b, Role::Heading { level: 2 }, &heading(panel, words), None);
    b.open(Role::Paragraph);
    b.style(look::HOME_LEDE);
    let branch = branch_words(panel, words, Word::ChangesItsBranch);
    b.text(Run::Plain, &words.say(Word::ChangesLede, &[Hole::Name(branch)]));
    b.close();
    b.close();
    queues(b, root, words, bindings, panel, times);
    section(b, &words.say(Word::ChangesLanded, &[]));
    b.open(Role::List);
    b.style(look::HOME_ROWS);
    for row in panel.landed() {
        let when = times.say(words, TimeForm::Clock, Wall::from_nanos(row.at()));
        row_start(b, root, words, bindings, row.task(), row.title(), &when);
        row_when(b, words, &when);
        b.close();
        b.close();
    }
    b.close();
    b.close();
    b.close();
}
#[expect(clippy::too_many_lines, reason = "train flights and ordered waiters share the queue row layout")]
fn queues(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    panel: &w::ChangesPanel,
    times: &mut Times,
) {
    let mut sections = 0_usize;
    for queue in panel.queues() {
        if !queue.changes().is_empty() || queue.in_flight().is_some() {
            sections = sections.checked_add(1).expect("wire queues fit");
        }
    }
    if sections == 0 {
        section(b, &words.say(Word::ChangesQueue, &[]));
        b.open(Role::Slot);
        b.style(look::HOME_ROWS);
        text(b, Role::Paragraph, &words.say(Word::ChangesEmpty, &[]), Some(look::HOME_EMPTY));
        b.close();
        b.close();
        return;
    }
    let mut shown = 0_usize;
    for queue in panel.queues() {
        if queue.changes().is_empty() && queue.in_flight().is_none() {
            continue;
        }
        shown = shown.checked_add(1).expect("wire queues fit");
        let heading = if *queue.way() == w::Way::Turn {
            super::change::say(
                words,
                W::Train,
                &[Hole::Name(queue.repository().into()), Hole::Name(queue.branch().into())],
            )
        } else if sections == 1 {
            words.say(Word::ChangesQueue, &[])
        } else {
            words.say(
                Word::ChangesQueueNamed,
                &[Hole::Name(queue.repository().into()), Hole::Name(queue.branch().into())],
            )
        };
        section(b, &heading);
        b.open(Role::List);
        b.style(look::HOME_ROWS);
        if let Some(flight) = queue.in_flight() {
            let identity = super::change::say(
                words,
                W::TrainFlight,
                &[Hole::Number(flight.task()), Hole::Number(flight.member()), Hole::Number(u64::from(flight.round()))],
            );
            let status = super::change::say(words, step_word(flight.step()), &[]);
            let label = crate::accounts::words::join(&[&identity, &words.say(Word::ToolSeparator, &[]), &status]);
            row_start(b, root, words, bindings, flight.member(), flight.title(), &label);
            row_when(b, words, &label);
            b.close();
            b.close();
        }
        for (index, row) in queue.changes().iter().enumerate() {
            let (checking, when) = if *queue.way() == w::Way::Turn {
                let place = skein_lib::len_u64(index)
                    .checked_add(1)
                    .expect("wire ordinal fits")
                    .checked_add(u64::from(queue.in_flight().is_some()))
                    .expect("wire rows fit");
                (false, words.say(Word::ChangesPlace, &[Hole::Name(ordinal(place, words))]))
            } else {
                match (index, row.checks_since()) {
                    (0, Some(since)) => (
                        true,
                        words.say(
                            Word::ChangesChecks,
                            &[Hole::Name(times.say(words, TimeForm::Ago, Wall::from_nanos(*since)))],
                        ),
                    ),
                    (0, None) => (true, Box::default()),
                    (1.., _) => (
                        false,
                        words.say(
                            Word::ChangesPlace,
                            &[Hole::Name(ordinal(
                                skein_lib::len_u64(index).checked_add(1).expect("wire ordinal fits"),
                                words,
                            ))],
                        ),
                    ),
                }
            };
            let when = if row.alone() {
                crate::accounts::words::join(&[
                    &when,
                    &words.say(Word::ToolSeparator, &[]),
                    &super::change::say(words, W::LandAlone, &[]),
                ])
            } else {
                when
            };
            let label = if checking {
                crate::accounts::words::join(&[&words.say(Word::ChangesChecking, &[]), &when])
            } else {
                when.clone()
            };
            row_start(b, root, words, bindings, row.task(), row.title(), &label);
            b.open(Role::Text);
            b.style(look::HOME_WHEN);
            if words.shell == Shell::Terminal {
                b.run(Run::Plain, &words.say(Word::ToolSeparator, &[]));
            }
            if checking {
                b.open(Role::Text);
                b.style(look::CHANGES_RUNNING);
                b.run(Run::Plain, &words.say(Word::ChangesChecking, &[]));
                b.close();
            }
            b.text(Run::Plain, &when);
            b.close();
            b.close();
            b.close();
        }
        if shown == sections && panel.more() > 0 {
            text(
                b,
                Role::Item,
                &words.say(Word::ChangesMore, &[Hole::Number(u64::from(panel.more()))]),
                Some(look::CHANGES_MORE),
            );
        }
        b.close();
        b.close();
    }
}
fn row_start(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    task: u64,
    title: &[u8],
    when: &[u8],
) {
    let from = match root.pages().changes() {
        Some(page) => page.from(root.client(), task),
        None => None,
    };
    let from = match from {
        Some(From::Goal(title)) => title.into(),
        Some(From::Chat(task)) => words.say(Word::ChangesChat, &[Hole::Number(task)]),
        None => Box::default(),
    };
    let label = crate::accounts::words::join(&[title, b" ", &from, &words.say(Word::ToolSeparator, &[]), when]);
    b.open(Role::Item);
    crate::link(b, &label, bindings.insert(Act::Go { address: Address::Task { task } }));
    b.open(Role::Text);
    b.run(Run::Plain, title);
    if !from.is_empty() {
        b.run(Run::Plain, b" ");
        b.open(Role::Text);
        b.style(look::CHANGES_MONO);
        b.run(Run::Plain, &from);
        b.close();
    }
    b.close();
}
fn row_when(b: &mut Builder, words: &Words, when: &[u8]) {
    b.open(Role::Text);
    b.style(look::HOME_WHEN);
    if words.shell == Shell::Terminal {
        b.run(Run::Plain, &words.say(Word::ToolSeparator, &[]));
    }
    b.run(Run::Plain, when);
    b.close();
}
fn section(b: &mut Builder, heading: &[u8]) {
    b.open(Role::Region);
    b.name(heading);
    text(b, Role::Heading { level: 2 }, heading, None);
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

pub(crate) fn step_word(step: &w::LandingStep) -> W {
    match step {
        w::LandingStep::Waiting => W::Waiting,
        w::LandingStep::Preparing => W::HeldPreparing,
        w::LandingStep::Proving => W::Checking,
        w::LandingStep::Proved => W::Proved,
        w::LandingStep::Turn => W::HeldLanding,
        w::LandingStep::Landed => W::Landed,
        w::LandingStep::Held => W::LeftHeld,
        w::LandingStep::Cancelled => W::Cancelled,
        w::LandingStep::Judging => W::Judging,
        w::LandingStep::Leaving => W::Leaving,
        w::LandingStep::Recording => W::Recording,
        w::LandingStep::Telling => W::Telling,
    }
}
