//! Shared milestone parts (temper/docs/design/client/pages/milestone.md, 2).
use super::goals::{self, Names};
use crate::{
    bindings::Bindings,
    look,
    words::{Shell, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::{List, bytes};
use skein_ui::{Builder, Role, Run, Style, phrases::Hole};
use temper_client_domain::{
    Address,
    pages::{
        Act,
        goals::Goals,
        milestones::{self, Count, Headline, Mark, Milestone, State},
        report::Report,
    },
};

/// One milestone links its top line, strip and headline to its task (home.md, 1, Goals).
pub(crate) fn row(
    b: &mut Builder,
    words: &Words,
    bindings: &mut Bindings,
    line: &c::TaskLine,
    milestone: &Milestone,
    goals: &Goals,
    names: &Names<'_>,
) {
    let mut needing = List::uncapped();
    for goal in &milestone.goals {
        if goals.needs_person(goal.chip.task) {
            needing.push(goal.chip.task).expect("held goals fit");
        }
    }
    let lines = milestone.goals.as_slice();
    let totals = milestone.totals.as_ref();
    let state = milestones::state(line, lines, totals, goals.needs_person(line.chip.task));
    let count_value = milestones::count(lines, totals);
    let (status, token) = state_word(words, state);
    let title = crate::common::chat::row_title(line.chip.title.bytes());
    let separator = words.say(Word::ToolSeparator, &[]);
    let progress = bytes::concat(&[&count(words, count_value), &separator]);
    let name = bytes::concat(&[&words.say(Word::MsTagTerminal, &[]), &title, &separator, &progress, &status]);
    if words.shell == Shell::Terminal {
        b.open(Role::Slot);
    }
    crate::bindings::link(b, &name, bindings.insert(Act::Go { address: Address::Task { task: line.chip.task } }));
    b.style(look::GOALS_ROW);
    b.style(look::MS_ROW);
    if line.chip.phase == c::Phase::Ended {
        b.style(look::GOALS_ROW_ENDED);
    }
    match words.shell {
        Shell::Browser => {
            b.open(Role::Text);
            b.style(look::GOALS_TOP);
            b.open(Role::Text);
            tag(b, words);
            b.text(Run::Plain, &title);
            b.close();
            b.open(Role::Text);
            b.style(look::GOALS_PROGRESS);
            b.open(Role::Text);
            b.run(Run::Plain, &progress);
            b.open(Role::Text);
            b.style(token);
            b.run(Run::Plain, &status);
            b.close();
            b.close();
            b.close();
            b.close();
        }
        Shell::Terminal => {
            b.text(Run::Plain, &name);
            b.close();
        }
    }
    strip(b, words, lines, needing.as_slice(), count_value);
    let value = milestones::headline(line, lines, totals, goals.needs_person(line.chip.task));
    let report = match value {
        Headline::Running { goal, .. } => goals.report(goal),
        Headline::Count(_)
        | Headline::Complete(_)
        | Headline::Open(_)
        | Headline::Closed(_)
        | Headline::Ended { .. } => None,
    };
    b.open(if words.shell == Shell::Browser { Role::Text } else { Role::Paragraph });
    b.style(look::GOALS_LINE);
    b.text(Run::Plain, &headline(words, value, lines, report, names));
    b.close();
    b.close();
}

pub(crate) fn state_word(words: &Words, state: State) -> (Box<[u8]>, Style) {
    let (word, style) = match state {
        State::Closed => (Word::GoalClosed, look::GOALS_STATE_CLOSED),
        State::Complete => (Word::MsComplete, look::GOALS_STATE_LANDED),
        State::Ended => (Word::MsEnded, look::GOALS_STATE_CLOSED),
        State::Done => (Word::MsDone, look::GOALS_STATE_LANDED),
        State::Needs => (Word::GoalNeedsYou, look::GOALS_STATE_NEEDS),
        State::Paused => (Word::GoalPaused, look::GOALS_STATE_PAUSED),
        State::Running => (Word::Running, look::GOALS_STATE_RUNNING),
        State::Waits => (Word::MsWaits, look::GOALS_STATE_WAITS),
        State::Open => (Word::MsOpen, look::GOALS_STATE_PLANNING),
    };
    (words.say(word, &[]), style)
}
pub(crate) fn count(words: &Words, count: Count) -> Box<[u8]> {
    words.say(Word::MsCount, &[Hole::Number(count.done), Hole::Number(count.total)])
}
pub(crate) fn tag(b: &mut Builder, words: &Words) {
    b.open(Role::Text);
    b.style(look::MS_TAG);
    words.text(b, if words.shell == Shell::Terminal { Word::MsTagTerminal } else { Word::MsTag }, &[]);
    b.close();
}
#[expect(clippy::manual_find, reason = "step code uses no closures")]
fn goal(lines: &[c::TaskLine], task: u64) -> Option<&c::TaskLine> {
    for line in lines {
        if line.chip.task == task {
            return Some(line);
        }
    }
    None
}
pub(crate) fn strip(b: &mut Builder, words: &Words, goals: &[c::TaskLine], needing: &[u64], count: Count) {
    let segments = milestones::strip(goals, needing);
    let name = words.say(Word::GoalsBarCount, &[Hole::Number(count.done), Hole::Number(count.total)]);
    if words.shell == Shell::Terminal {
        let mut marks = List::uncapped();
        for segment in &segments {
            let mark = match segment.mark {
                Mark::Done => "■",
                Mark::Needs | Mark::Live => "▒",
                Mark::Waits => "□",
                Mark::Failed => "×",
            };
            for byte in mark.as_bytes() {
                marks.push(*byte).expect("held segments fit");
            }
        }
        b.open(Role::Paragraph);
        b.text(Run::Plain, &bytes::concat(&[marks.as_slice(), b" ", &name]));
        b.close();
        return;
    }
    b.open(Role::Figure);
    b.style(look::MS_STRIP);
    b.name(&name);
    for segment in &segments {
        b.open(Role::Text);
        b.style(match segment.mark {
            Mark::Done => look::MS_DONE,
            Mark::Live => look::MS_LIVE,
            Mark::Needs => look::MS_NEEDS,
            Mark::Waits => look::MS_WAITS,
            Mark::Failed => look::MS_FAILED,
        });
        if let Some(line) = goal(goals, segment.task)
            && let Some(id) = milestones_id(line)
        {
            b.open(Role::Text);
            b.run(Run::Plain, id);
            b.close();
        }
        b.close();
    }
    b.close();
}
fn milestones_id(line: &c::TaskLine) -> Option<&[u8]> {
    temper_client_domain::pages::goals::id(line.chip.title.bytes())
}
fn name(line: &c::TaskLine) -> Box<[u8]> {
    match milestones_id(line) {
        Some(id) => bytes::copy_of(id),
        None => crate::common::chat::cut_title(line.chip.title.bytes(), 40, false),
    }
}
fn noun(words: &Words, total: u64) -> Box<[u8]> {
    words.say(if total == 1 { Word::MsGoalOne } else { Word::MsGoalMany }, &[])
}
fn sentence(words: &Words, count: Count) -> Box<[u8]> {
    let noun = Hole::Name(noun(words, count.total));
    if count.done == 0 {
        words.say(Word::MsNoneSentence, &[Hole::Number(count.total), noun])
    } else {
        words.say(Word::MsCountSentence, &[Hole::Number(count.done), Hole::Number(count.total), noun])
    }
}
fn without_stop(text: &[u8]) -> &[u8] {
    text.strip_suffix(b".").unwrap_or(text)
}
pub(crate) fn headline(
    words: &Words,
    headline: Headline,
    lines: &[c::TaskLine],
    report: Option<&Report>,
    names: &Names<'_>,
) -> Box<[u8]> {
    match headline {
        Headline::Count(count) => sentence(words, count),
        Headline::Complete(count) => {
            if count.total == 0 {
                words.say(Word::MsCompleteEmpty, &[])
            } else {
                words.say(Word::MsAll, &[Hole::Number(count.total), Hole::Name(noun(words, count.total))])
            }
        }
        Headline::Open(count) => {
            if count.total == 0 {
                words.say(Word::MsNoGoals, &[])
            } else {
                words.say(Word::MsOpenSentence, &[Hole::Number(count.done), Hole::Name(noun(words, count.done))])
            }
        }
        Headline::Closed(count) => words.say(Word::MsClosedSentence, &[Hole::Name(sentence(words, count))]),
        Headline::Ended { count, failed, first } => {
            let prefix = Hole::Name(bytes::copy_of(without_stop(&sentence(words, count))));
            if failed == 1
                && let Some(task) = first
                && let Some(line) = goal(lines, task)
            {
                words.say(Word::MsFailedGoal, &[prefix, Hole::Name(name(line))])
            } else {
                words.say(Word::MsFailedCount, &[prefix, Hole::Number(u64::from(failed))])
            }
        }
        Headline::Running { count, goal: task, more } => {
            let Some(line) = goal(lines, task) else { return sentence(words, count) };
            let state =
                temper_client_domain::pages::goals::state(&line.chip, false, report.is_some(), line.delegates_total);
            let mut detail = goals::headline(words, line, state, report, names);
            if more != 0 {
                let prefix = Hole::Name(bytes::copy_of(without_stop(&detail)));
                detail = if more == 1 {
                    words.say(Word::MsMoreOne, &[prefix])
                } else {
                    words.say(Word::MsMoreMany, &[prefix, Hole::Number(u64::from(more))])
                };
            }
            words
                .say(Word::MsRunning, &[Hole::Name(sentence(words, count)), Hole::Name(name(line)), Hole::Name(detail)])
        }
    }
}
