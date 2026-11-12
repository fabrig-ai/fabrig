//! Work's head, filters and cards (temper/docs/design/client/pages/README.md; client/web/look.md).
use crate::{
    frame::Bindings,
    look,
    words::{Shell, TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::{List, Wall};
use skein_ui::{Builder, Fraction, Role, Run, Style, phrases::Hole};
use temper_client_domain::{
    Address, ChatsFilter, Domain, GoalsFilter, WorkFilter,
    pages::{
        Act,
        changes::State,
        work::{self, Item, Landed, Work},
    },
};
use temper_client_protocol_wire::v1 as w;

pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    project: u32,
    times: &mut Times,
) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::WORK_PAGE);
    let placed = head(b, root, words, bindings, project, times);
    if let Some(placed) = placed {
        let drafts_only = root.pages().address() == &(Address::Work { project, filter: WorkFilter::Drafts });
        if !drafts_only && !placed.in_flight.is_empty() {
            section(b, words, Word::WorkFlightSection, placed.in_flight.len(), Some(Word::WorkNewest));
            for card in &placed.in_flight {
                card_view(b, root, words, bindings, times, project, card);
            }
            b.close();
        }
        if !placed.drafts.is_empty() {
            section(b, words, Word::WorkDraftsSection, placed.drafts.len(), Some(Word::WorkStartsEach));
            for card in &placed.drafts {
                card_view(b, root, words, bindings, times, project, card);
            }
            b.close();
        } else if drafts_only {
            text(b, Role::Paragraph, look::GOALS_EMPTY, &words.say(Word::GoalsEmpty, &[]));
        }
        if !drafts_only && !placed.landed.is_empty() {
            section(b, words, Word::WorkLandedSection, placed.landed.len(), None);
            // The quiet link belongs at the right of the section's head.
            link(b, words, bindings, &words.say(Word::GoalsAll, &[]), Address::Changes { project });
            b.style(look::WORK_BYLINE);
            b.close();
            b.close();
            b.open(Role::List);
            b.style(look::WORK_ROWS);
            for row in &placed.landed {
                landed(b, root, words, bindings, times, row);
            }
            b.close();
            if placed.goal_branches > 0 {
                let line = if placed.goal_branches == 1 {
                    words.say(Word::WorkGoalBranchOne, &[])
                } else {
                    words.say(
                        Word::WorkGoalBranchMany,
                        &[Hole::Name(super::goals::lowercase(&super::changes::count_named(
                            u64::from(placed.goal_branches),
                            words,
                        )))],
                    )
                };
                text(b, Role::Paragraph, look::WORK_BYLINE, &line);
            }
            b.close();
        }
    }
    b.close();
}

/// The same head and filters on each of Work's addresses; lists wait for their answers.
pub(crate) fn head(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    project: u32,
    times: &mut Times,
) -> Option<Work> {
    let placed = match work::facts(root, project) {
        Some(facts) => Some(work::place(&facts, times.now)),
        None => None,
    };
    let name = super::project_name(root.pages().me(root.client()), project)
        .unwrap_or(words.say(Word::Project, &[Hole::Number(u64::from(project))]));
    b.open(Role::Slot);
    b.style(look::WORK_INTRO);
    text(b, Role::Paragraph, look::WORK_KICKER, &words.say(Word::WorkKicker, &[Hole::Name(name.clone())]));
    if let Some(placed) = &placed {
        text(b, Role::Heading { level: 1 }, look::WORK_HEADLINE, &heading(placed, words));
    }
    text(b, Role::Paragraph, look::WORK_LEDE, &words.say(Word::WorkLede, &[Hole::Name(name)]));
    if let Some(page) = root.pages().changes() {
        match page.state() {
            State::Shown(panel) => {
                let mut red = List::uncapped();
                for queue in panel.queues() {
                    if queue.red() {
                        red.push(queue.branch()).expect("one signal per queue");
                    }
                }
                let facts = super::changes::LandingFacts::panel(panel, red.as_slice());
                b.open(Role::Paragraph);
                b.style(look::WORK_LANDING);
                super::changes::landing_parts(b, &super::changes::landing_line(&facts, words, times), words);
                b.close();
            }
            State::Loading | State::Ended => {}
        }
    }
    filters(b, root, words, bindings, project, placed.as_ref());
    b.close();
    placed
}

pub(crate) fn heading(placed: &Work, words: &Words) -> Box<[u8]> {
    let mut parts = List::with_capacity(5);
    for (count, word) in [
        (placed.in_flight.len(), Word::WorkFlightCount),
        (placed.drafts.len(), if placed.drafts.len() == 1 { Word::WorkDraftCount } else { Word::WorkDraftsCount }),
        (placed.landed.len(), Word::WorkLandedCount),
    ] {
        if count == 0 {
            continue;
        }
        let number = if count == 1 {
            words.say(Word::WorkOne, &[])
        } else {
            super::changes::count_named(u64::from(count), words)
        };
        let number = if parts.is_empty() { number } else { super::goals::lowercase(&number) };
        if !parts.is_empty() {
            parts.push(words.say(Word::GoalsWaySeparator, &[])).expect("three heading parts and separators");
        }
        parts.push(words.say(word, &[Hole::Name(number)])).expect("three heading parts and separators");
    }
    if parts.is_empty() {
        return words.say(Word::WorkNothing, &[]);
    }
    let mut heading = Box::default();
    for part in &parts {
        heading = crate::accounts::words::join(&[&heading, part]);
    }
    crate::accounts::words::join(&[&heading, &words.say(Word::GoalsWayEnd, &[])])
}

fn filters(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    project: u32,
    placed: Option<&Work>,
) {
    let counts = match placed {
        Some(placed) => placed.filters,
        None => work::Filters::default(),
    };
    let shown = work::shown(&counts, work::overrides(root, project));
    let current = root.pages().address();
    b.open(Role::Navigation);
    b.name(&words.say(Word::WorkFilters, &[]));
    b.style(look::WORK_FILTERS);
    for (show, count, word, unknown, address, active) in [
        (
            true,
            None,
            Word::GoalsAll,
            Word::GoalsAll,
            Address::Work { project, filter: WorkFilter::All },
            current == &(Address::Work { project, filter: WorkFilter::All }),
        ),
        (
            shown.drafts,
            Some(counts.drafts),
            Word::WorkDraftsFilter,
            Word::DraftsChats,
            Address::Work { project, filter: WorkFilter::Drafts },
            current == &(Address::Work { project, filter: WorkFilter::Drafts }),
        ),
        (
            shown.goals,
            Some(counts.goals),
            Word::WorkGoalsFilter,
            Word::Goals,
            Address::Goals { project, filter: GoalsFilter::All },
            goals_current(current, project),
        ),
        (
            shown.changes,
            Some(counts.changes),
            Word::WorkChangesFilter,
            Word::Changes,
            Address::Changes { project },
            current == &(Address::Changes { project }),
        ),
        (
            shown.chats,
            Some(counts.chats),
            Word::WorkChatsFilter,
            Word::Chats,
            Address::Chats { filter: ChatsFilter::All },
            false,
        ),
    ] {
        if !show && !active {
            continue;
        }
        let label = match count {
            Some(count) if placed.is_some() => words.say(word, &[Hole::Number(u64::from(count))]),
            Some(_) => words.say(unknown, &[]),
            None => words.say(word, &[]),
        };
        link(b, words, bindings, &label, address);
        b.style(look::WORK_CHIP);
        if active {
            b.state(skein_ui::State::Current);
            b.style(look::WORK_CURRENT);
        }
        b.close();
    }
    b.close();
}
fn goals_current(address: &Address, project: u32) -> bool {
    if let Address::Goals { project: open, .. } = address { *open == project } else { false }
}

// With a right-side link, the caller closes the head after adding it.
fn section(b: &mut Builder, words: &Words, word: Word, count: u32, quiet: Option<Word>) {
    let name = words.say(word, &[Hole::Number(u64::from(count))]);
    b.open(Role::Region);
    b.name(&name);
    b.style(look::WORK_SECTION);
    b.open(Role::Slot);
    b.style(look::WORK_SECTION_HEAD);
    text(b, Role::Heading { level: 2 }, look::GOALS_SECTION_TITLE, &name);
    if let Some(quiet) = quiet {
        text(b, Role::Paragraph, look::WORK_BYLINE, &words.say(quiet, &[]));
        b.close();
    }
}

fn card_view(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    project: u32,
    card: &work::Card,
) {
    b.open(Role::Slot);
    b.style(look::WORK_CARD);
    match card.item {
        Item::Goal(task) => goal(b, root, words, bindings, project, task),
        Item::Change(task) => change(b, root, words, bindings, task),
        Item::Chat(task) => chat(b, root, words, bindings, times, task, false),
        Item::Draft(task) => chat(b, root, words, bindings, times, task, true),
    }
    if !card.changes.is_empty() {
        b.open(Role::List);
        b.style(look::WORK_ROWS);
        for task in &card.changes {
            change_row(b, root, words, bindings, times, *task);
        }
        b.close();
    }
    if let Item::Goal(task) = card.item
        && let Some(page) = root.pages().goals()
        && let Some(report) = page.report(task)
        && let Some(at) = report.at
    {
        text(
            b,
            Role::Paragraph,
            look::WORK_BYLINE,
            &words.say(Word::WorkByline, &[Hole::Name(times.say(words, TimeForm::Clock, at))]),
        );
    }
    b.close();
}

fn goal(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, project: u32, task: u64) {
    let Some(page) = root.pages().goals() else { return };
    let Some(live) = page.live(root.client()) else { return };
    for line in live.rows.iter().chain(page.ended()) {
        if line.chip.task != task {
            continue;
        }
        let mut parts = List::with_capacity(2);
        if let Some(way) = goal_way(root, project, line.label, words) {
            parts.push(way).expect("way and budget");
        }
        let numbers = line.budget.numbers;
        if numbers.budget > 0 {
            parts
                .push(words.say(Word::GoalSpent, &[Hole::Number(numbers.spent), Hole::Number(numbers.budget)]))
                .expect("way and budget");
        }
        linked_from(
            b,
            words,
            bindings,
            Word::WorkGoalTag,
            look::WORK_AGENT,
            line.chip.title.bytes(),
            Address::Task { task },
            parts.as_slice(),
        );
        let report = page.report(task);
        let state = temper_client_domain::pages::goals::state(
            &line.chip,
            page.needs_person(task),
            report.is_some(),
            line.delegates_total,
        );
        text(b, Role::Paragraph, look::WORK_QUOTE, &super::goals::headline(words, line, state, report));
        if line.delegates_total > 0 {
            let label = words.say(
                Word::GoalCount,
                &[Hole::Number(u64::from(line.delegates_done)), Hole::Number(u64::from(line.delegates_total))],
            );
            match words.shell {
                Shell::Browser => {
                    b.open(Role::Progress);
                    b.style(look::WORK_BAR);
                    b.name(&label);
                    b.fraction(Fraction {
                        done: u64::from(line.delegates_done.min(line.delegates_total)),
                        total: u64::from(line.delegates_total),
                    });
                    b.run(Run::Plain, &label);
                    b.close();
                }
                Shell::Terminal => text(b, Role::Paragraph, look::WORK_BAR, &label),
            }
        }
        return;
    }
}
fn change(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, task: u64) {
    let Some(panel) = panel(root) else { return };
    for queue in panel.queues() {
        if let Some(flight) = queue.in_flight()
            && flight.member() == task
        {
            let status = super::change::say(words, super::changes::step_word(flight.step()), &[]);
            linked_from(
                b,
                words,
                bindings,
                Word::WorkChangeTag,
                look::WORK_CODE,
                flight.title(),
                Address::Task { task },
                &[status],
            );
            return;
        }
        for (index, row) in queue.changes().iter().enumerate() {
            if row.task() == task {
                linked_from(
                    b,
                    words,
                    bindings,
                    Word::WorkChangeTag,
                    look::WORK_CODE,
                    row.title(),
                    Address::Task { task },
                    &[queue_label(queue, index, words)],
                );
                return;
            }
        }
    }
}
fn chat(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    task: u64,
    draft: bool,
) {
    let Some(page) = root.pages().chats() else { return };
    let Some(line) = chat_line(root, task) else { return };
    if draft {
        from(b, words, Word::WorkDraftTag, look::WORK_PERSON);
        if let Some(document) = page.documents.get(&task) {
            let name = words.say(
                Word::WorkDraftName,
                &[Hole::Name(temper_client_domain::pages::chat::document::slug(document.bytes()))],
            );
            b.text(Run::Plain, &name);
        }
        separated(b, words, &words.say(Word::WorkAChatWithYou, &[]));
        separated(b, words, &times.say(words, TimeForm::Clock, line.last_activity));
        b.close();
        text(b, Role::Paragraph, look::WORK_QUOTE, line.chip.title.bytes());
        link(b, words, bindings, &words.say(Word::WorkOpenDraft, &[]), Address::Chat { task });
        b.close();
    } else {
        // The chat list carries no creation time; activity and phase times do not name its start.
        let label = crate::accounts::words::join(&[
            &words.say(Word::WorkWithYou, &[]),
            &words.say(Word::ToolSeparator, &[]),
            &words.say(Word::WorkChatTitle, &[Hole::Name(line.chip.title.bytes().into())]),
        ]);
        linked_from(b, words, bindings, Word::WorkChatTag, look::WORK_PERSON, &label, Address::Chat { task }, &[]);
    }
}
fn chat_line(root: &Domain, task: u64) -> Option<&c::TaskLine> {
    let chats = root.pages().chats()?;
    for line in &chats.live(root.client())?.rows {
        if line.chip.task == task {
            return Some(line);
        }
    }
    if let Some(page) = &chats.page {
        for line in &page.live {
            if line.chip.task == task {
                return Some(line);
            }
        }
        match &page.ended {
            c::EndedGroup::Rows { rows, .. } => {
                for line in rows {
                    if line.chip.task == task {
                        return Some(line);
                    }
                }
            }
            c::EndedGroup::None => {}
        }
    }
    None
}
fn panel(root: &Domain) -> Option<&w::ChangesPanel> {
    match root.pages().changes()?.state() {
        State::Shown(panel) => Some(panel),
        State::Loading | State::Ended => None,
    }
}
fn queue_word(index: usize, words: &Words) -> Box<[u8]> {
    if index == 0 {
        words.say(Word::ChangesChecking, &[])
    } else {
        words.say(
            Word::ChangesPlace,
            &[Hole::Name(super::changes::ordinal(
                skein_lib::len_u64(index).checked_add(1).expect("wire queue fits"),
                words,
            ))],
        )
    }
}
fn change_row(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, times: &mut Times, task: u64) {
    let Some(panel) = panel(root) else { return };
    for queue in panel.queues() {
        if let Some(flight) = queue.in_flight()
            && flight.member() == task
        {
            let status = super::change::say(words, super::changes::step_word(flight.step()), &[]);
            work_row(b, words, bindings, task, b"\xc2\xb7", flight.title(), &status);
            return;
        }
        for (index, row) in queue.changes().iter().enumerate() {
            if row.task() == task {
                work_row(b, words, bindings, task, b"\xc2\xb7", row.title(), &queue_label(queue, index, words));
                return;
            }
        }
    }
    for change in panel.landed() {
        if change.task() == task {
            let when = words
                .say(Word::GoalLanded, &[Hole::Name(times.say(words, TimeForm::Clock, Wall::from_nanos(change.at())))]);
            work_row(b, words, bindings, task, b"\xe2\x9c\x93", change.title(), &when);
            return;
        }
    }
}
fn landed(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, times: &mut Times, row: &Landed) {
    let Some(panel) = panel(root) else { return };
    for change in panel.landed_default() {
        if change.task() != row.change {
            continue;
        }
        let when = times.say(words, TimeForm::Clock, row.at);
        let origin = match root.pages().changes() {
            Some(page) => page.from(root.client(), row.change),
            None => None,
        };
        let (task, title) = match row.item {
            Item::Goal(goal) => {
                let title = match origin {
                    Some(temper_client_domain::pages::changes::From::Goal(title)) => title,
                    Some(temper_client_domain::pages::changes::From::Chat(_)) | None => change.title(),
                };
                (goal, title)
            }
            Item::Change(task) => (task, change.title()),
            Item::Chat(_) | Item::Draft(_) => unreachable!("landed rows are goals or changes"),
        };
        let commit = crate::public::short(&row.commit);
        let status = match origin {
            Some(temper_client_domain::pages::changes::From::Chat(_)) if row.item == Item::Change(row.change) => {
                crate::accounts::words::join(&[
                    &words.say(Word::WorkAChatWithYou, &[]),
                    &words.say(Word::ToolSeparator, &[]),
                    &commit,
                ])
            }
            Some(
                temper_client_domain::pages::changes::From::Chat(_)
                | temper_client_domain::pages::changes::From::Goal(_),
            )
            | None => commit.as_slice().into(),
        };
        work_row(b, words, bindings, task, &when, title, &status);
        return;
    }
}

fn work_row(
    b: &mut Builder,
    words: &Words,
    bindings: &mut Bindings,
    task: u64,
    mark: &[u8],
    title: &[u8],
    status: &[u8],
) {
    b.open(Role::Item);
    b.style(look::WORK_ROW);
    text(b, Role::Paragraph, look::WORK_MONO, mark);
    link(b, words, bindings, title, Address::Task { task });
    b.close();
    text(b, Role::Paragraph, look::WORK_BYLINE, status);
    b.close();
}
fn from(b: &mut Builder, words: &Words, tag: Word, style: Style) {
    b.open(Role::Paragraph);
    b.style(look::WORK_FROM);
    text(b, Role::Text, style, &words.say(tag, &[]));
    if words.shell == Shell::Terminal {
        b.text(Run::Plain, b" ");
    }
}
fn separated(b: &mut Builder, words: &Words, value: &[u8]) {
    words.text(b, Word::ToolSeparator, &[]);
    b.text(Run::Plain, value);
}
fn text(b: &mut Builder, role: Role, style: Style, value: &[u8]) {
    b.open(role);
    b.style(style);
    b.name(value);
    b.text(Run::Plain, value);
    b.close();
}
fn link(b: &mut Builder, _words: &Words, bindings: &mut Bindings, label: &[u8], address: Address) {
    crate::link(b, label, bindings.insert(Act::Go { address }));
    b.text(Run::Plain, label);
}

fn goal_way(root: &Domain, project: u32, label: u32, words: &Words) -> Option<Box<[u8]>> {
    for member in &root.pages().me(root.client())?.projects {
        if member.project != project {
            continue;
        }
        for template in &member.templates {
            if template.tracked.is_some() && template.label == label {
                return Some(super::goals::way_name(template.name.bytes(), words));
            }
        }
    }
    None
}

#[expect(clippy::too_many_arguments, reason = "one card from line carries its tag, destination and read metadata")]
fn linked_from(
    b: &mut Builder,
    words: &Words,
    bindings: &mut Bindings,
    tag: Word,
    style: Style,
    title: &[u8],
    address: Address,
    parts: &[Box<[u8]>],
) {
    match words.shell {
        Shell::Browser => {
            from(b, words, tag, style);
            link(b, words, bindings, title, address);
            b.close();
            for part in parts {
                separated(b, words, part);
            }
            b.close();
        }
        Shell::Terminal => {
            let mut label = crate::accounts::words::join(&[&words.say(tag, &[]), b" ", title]);
            for part in parts {
                label = crate::accounts::words::join(&[&label, &words.say(Word::ToolSeparator, &[]), part]);
            }
            link(b, words, bindings, &label, address);
            b.style(look::WORK_FROM);
            b.close();
        }
    }
}

fn queue_label(queue: &w::ChangesQueue, index: usize, words: &Words) -> Box<[u8]> {
    if *queue.way() == w::Way::Queue {
        return queue_word(index, words);
    }
    let place = skein_lib::len_u64(index)
        .checked_add(1)
        .expect("wire queue fits")
        .checked_add(u64::from(queue.in_flight().is_some()))
        .expect("wire queue fits");
    words.say(Word::ChangesPlace, &[Hole::Name(super::changes::ordinal(place, words))])
}
