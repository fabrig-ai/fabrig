//! A milestone's Work card (client/pages/work.md, 1).
use super::{
    Address, Bindings, Box, Builder, Domain, Hole, Landed, List, Role, Shell, Style, TimeForm, Times, Word, Words, c,
    change_row, linked_from, look, text, work, work_row,
};
use core::cmp::Reverse;
use skein_lib::{Map, Wall};
use temper_client_domain::pages::{goals, milestones};

pub(super) fn byline(words: &Words, row: &Landed) -> Option<Box<[u8]>> {
    let Some(milestone) = &row.milestone else { return row.signed.clone() };
    let label = words.say(
        if milestone.completed.is_some() { Word::WorkMilestoneComplete } else { Word::WorkInMilestone },
        &[Hole::Name(milestone.name.clone())],
    );
    Some(match &row.signed {
        Some(signed) => crate::accounts::words::join(&[signed, &words.say(Word::ToolSeparator, &[]), &label]),
        None => label,
    })
}

pub(super) fn card(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    task: u64,
    card: &work::Card,
) {
    let Some(reads) = root.pages().milestones() else { return };
    let Some(milestone) = reads.milestone(task) else { return };
    let mut own = None;
    for line in reads.lines() {
        if line.chip.task == task {
            own = Some(line);
            break;
        }
    }
    let Some(line) = own else { return };
    b.style(look::MS_CARD);
    let count = milestones::count(milestone.goals.as_slice(), milestone.totals.as_ref());
    from_line(b, words, bindings, line, milestone, count);
    let mut needing = List::uncapped();
    let goals_read = root.pages().goals();
    for goal in &milestone.goals {
        if let Some(goals) = goals_read
            && goals.needs_person(goal.chip.task)
        {
            needing.push(goal.chip.task).expect("one need per goal");
        }
    }
    let names = super::super::goals::Names::of(root);
    let needs = match goals_read {
        Some(read) => read.needs_person(task),
        None => false,
    };
    if milestone.totals.is_some() {
        let headline = milestones::headline(line, milestone.goals.as_slice(), milestone.totals.as_ref(), needs);
        let report = match headline {
            milestones::Headline::Running { goal, .. } => milestone.headline(goal),
            milestones::Headline::Count(_)
            | milestones::Headline::Complete(_)
            | milestones::Headline::Open(_)
            | milestones::Headline::Closed(_)
            | milestones::Headline::Ended { .. } => None,
        };
        text(
            b,
            Role::Paragraph,
            look::WORK_QUOTE,
            &super::super::milestones::headline(words, headline, milestone.goals.as_slice(), report, &names),
        );
        super::super::milestones::strip(b, words, milestone.goals.as_slice(), needing.as_slice(), count);
    }
    let mut waiting = List::uncapped();
    let mut ended = Map::uncapped();
    b.open(Role::List);
    b.style(look::WORK_ROWS);
    for placed in &card.goals {
        let Some(goal) = held_goal(root, placed.task) else { continue };
        if goal.chip.phase == c::Phase::Waiting {
            waiting.push(goal).expect("one waiting line per placed goal");
        } else {
            goal_line(b, root, words, bindings, times, goal, &names, look::MS_LIVE);
            changes(b, root, words, bindings, times, card, goal.chip.task);
        }
    }
    for goal in &milestone.goals {
        if goal.chip.phase == c::Phase::Ended && !placed_goal(card, goal.chip.task) {
            ended.insert((Reverse(goal.ended_at), goal.chip.task), goal).expect("one ended line per goal");
        }
    }
    b.close();
    if !waiting.is_empty() {
        let label = if waiting.len() == 1 {
            words.say(Word::MsWaitOne, &[])
        } else {
            words.say(Word::MsWaitMany, &[Hole::Number(u64::from(waiting.len()))])
        };
        fold(b, words, &label);
        for goal in &waiting {
            goal_line(b, root, words, bindings, times, goal, &names, look::MS_WAITS);
            changes(b, root, words, bindings, times, card, goal.chip.task);
        }
        b.close();
        b.close();
    }
    ended_rows(b, root, words, bindings, times, &ended);
}

fn placed_goal(card: &work::Card, task: u64) -> bool {
    for goal in &card.goals {
        if goal.task == task {
            return true;
        }
    }
    false
}

fn held_goal(root: &Domain, task: u64) -> Option<&c::TaskLine> {
    if let Some(goals) = root.pages().goals()
        && let Some(live) = goals.live(root.client())
    {
        for line in &live.rows {
            if line.chip.task == task && line.chip.phase != c::Phase::Ended {
                return Some(line);
            }
        }
    }
    for milestone in root.pages().milestones()?.drawn() {
        for line in &milestone.goals {
            if line.chip.task == task && line.chip.phase != c::Phase::Ended {
                return Some(line);
            }
        }
    }
    None
}

fn from_line(
    b: &mut Builder,
    words: &Words,
    bindings: &mut Bindings,
    line: &c::TaskLine,
    milestone: &milestones::Milestone,
    count: milestones::Count,
) {
    let mut parts = List::with_capacity(2);
    let noun = words.say(if count.total == 1 { Word::MsGoalOne } else { Word::MsGoalMany }, &[]);
    let mut amount = words.say(Word::MsGoals, &[Hole::Number(count.total), Hole::Name(noun)]);
    if let Some(facts) = &milestone.facts {
        let separator =
            if facts.whole.is_some() { words.say(Word::ToolSeparator, &[]) } else { b" ".as_slice().into() };
        amount = crate::accounts::words::join(&[
            &amount,
            &separator,
            &words.say(if facts.whole.is_some() { Word::MsWhole } else { Word::MsSoFar }, &[]),
        ]);
    }
    if milestone.totals.is_some() {
        parts.push(amount).expect("count and spend");
    }
    if let Some(totals) = &milestone.totals
        && line.budget.numbers.budget > 0
    {
        parts
            .push(words.say(Word::MsSpent, &[Hole::Number(totals.spent), Hole::Number(line.budget.numbers.budget)]))
            .expect("count and spend");
    }
    linked_from(
        b,
        words,
        bindings,
        Word::MsTag,
        look::WORK_MILESTONE,
        line.chip.title.bytes(),
        Address::Task { task: line.chip.task },
        parts.as_slice(),
    );
}

fn ended_rows(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    ended: &Map<(Reverse<Option<Wall>>, u64), &c::TaskLine>,
) {
    let names = super::super::goals::Names::of(root);
    if !ended.is_empty() {
        let mut done = 0_u64;
        let mut failed = 0_u64;
        let mut closed = 0_u64;
        let mut last = None;
        for (_, goal) in ended {
            match goals::state(&goal.chip, false, false, goal.delegates_total) {
                goals::State::Landed => {
                    done = done.saturating_add(1);
                    if last.is_none() {
                        last = Some(*goal);
                    }
                }
                goals::State::Failed => failed = failed.saturating_add(1),
                goals::State::Closed => closed = closed.saturating_add(1),
                goals::State::Needs { .. }
                | goals::State::Paused
                | goals::State::Waits(_)
                | goals::State::WaitsForRoom
                | goals::State::Planning
                | goals::State::Running => {}
            }
        }
        let mut label: Box<[u8]> = Box::default();
        for (count, word) in [(done, Word::MsFoldDone), (failed, Word::MsFoldFailed), (closed, Word::MsFoldClosed)] {
            if count == 0 {
                continue;
            }
            let mut part = words.say(word, &[Hole::Number(count)]);
            if word == Word::MsFoldDone
                && let Some(last) = last
                && let Some(at) = last.ended_at
            {
                let name = match goals::id(last.chip.title.bytes()) {
                    Some(id) => id.into(),
                    None => crate::common::chat::cut_title(last.chip.title.bytes(), 40, false),
                };
                part = words.say(
                    Word::MsLastDone,
                    &[Hole::Name(part), Hole::Name(name), Hole::Name(times.say(words, TimeForm::Clock, at))],
                );
            }
            label = if label.is_empty() {
                part
            } else {
                crate::accounts::words::join(&[&label, &words.say(Word::ToolSeparator, &[]), &part])
            };
        }
        fold(b, words, &label);
        for (_, goal) in ended {
            goal_line(b, root, words, bindings, times, goal, &names, look::MS_LANDED);
        }
        b.close();
        b.close();
    }
}

fn fold(b: &mut Builder, words: &Words, label: &[u8]) {
    match words.shell {
        Shell::Browser => {
            b.open(Role::Disclosure);
            b.name(label);
            b.style(look::MS_FOLD);
        }
        Shell::Terminal => {
            b.open(Role::Slot);
            text(b, Role::Paragraph, look::MS_FOLD, label);
        }
    }
    b.open(Role::List);
    b.style(look::WORK_ROWS);
}

#[expect(clippy::too_many_arguments, reason = "a milestone goal line shares the page names and row token")]
fn goal_line(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    line: &c::TaskLine,
    names: &super::super::goals::Names<'_>,
    style: Style,
) {
    let read = root.pages().goals();
    let needs = match read {
        Some(read) => read.needs_person(line.chip.task),
        None => false,
    };
    let mut has_report = false;
    if let Some(reads) = root.pages().milestones() {
        for milestone in reads.drawn() {
            has_report |= milestone.headline(line.chip.task).is_some();
        }
    }
    if let Some(read) = read {
        has_report |= read.report(line.chip.task).is_some();
    }
    let state = goals::state(&line.chip, needs, has_report, line.delegates_total);
    let (mut status, _) = super::super::goals::state_word(words, state, names, &line.chip);
    let mark: &[u8] = match state {
        goals::State::Landed => {
            if let Some(at) = line.ended_at {
                status = words.say(Word::GoalLanded, &[Hole::Name(times.say(words, TimeForm::Clock, at))]);
            }
            "✓".as_bytes()
        }
        goals::State::Failed => "×".as_bytes(),
        goals::State::Closed => "–".as_bytes(),
        goals::State::Needs { .. }
        | goals::State::Paused
        | goals::State::Waits(_)
        | goals::State::WaitsForRoom
        | goals::State::Planning
        | goals::State::Running => {
            if line.chip.phase == c::Phase::Waiting {
                "·".as_bytes()
            } else {
                "▸".as_bytes()
            }
        }
    };
    if line.chip.phase != c::Phase::Ended && line.chip.phase != c::Phase::Waiting && line.delegates_total > 0 {
        status = crate::accounts::words::join(&[
            &status,
            &words.say(Word::ToolSeparator, &[]),
            &words.say(
                Word::GoalCount,
                &[Hole::Number(u64::from(line.delegates_done)), Hole::Number(u64::from(line.delegates_total))],
            ),
        ]);
    }
    work_row(b, words, bindings, line.chip.task, mark, line.chip.title.bytes(), &status, None, style);
}

fn changes(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    card: &work::Card,
    task: u64,
) {
    for goal in &card.goals {
        if goal.task != task || goal.changes.is_empty() {
            continue;
        }
        for row in &goal.changes {
            change_row(b, root, words, bindings, times, row, look::MS_CHANGE);
        }
    }
}
