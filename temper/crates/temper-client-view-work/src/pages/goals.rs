//! The project goals' page (temper/docs/design/client/pages/README.md).
pub(crate) use crate::common::goals::way_name;
pub use crate::common::goals::{Way, ways};
use crate::{
    bindings::Bindings,
    look,
    words::{Shell, TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::List;
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain, GoalsFilter,
    pages::{
        Act,
        goals::{self, State},
        report::Report,
    },
};

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
    b.style(look::GOALS);
    super::work::head(b, root, words, bindings, project, times);
    intro(b, root, words, project);
    list(b, root, words, bindings, project, times);
    b.close();
}

fn intro(b: &mut Builder, root: &Domain, words: &Words, project: u32) {
    let me = root.pages().me(root.client());
    let page = root.pages().goals();
    b.open(Role::Slot);
    b.style(look::GOALS_INTRO);
    if me.is_some()
        && let Some(page) = page
        && let Some(live) = page.live(root.client())
    {
        let mut underway = 0_u32;
        for line in live.rows.iter().chain(page.ended()) {
            let state = goals::state(
                &line.chip,
                page.needs_person(line.chip.task),
                page.report(line.chip.task).is_some(),
                line.delegates_total,
            );
            match state {
                State::Needs { .. } | State::Running | State::Planning | State::Paused => {
                    underway = underway.checked_add(1).expect("goal rows fit");
                }
                State::Landed | State::Closed | State::Failed => {}
            }
        }
        let count = page.count(root.client());
        let word = if page.unread(root.client()) {
            if count == 1 { Word::GoalsMoreOne } else { Word::GoalsMoreMany }
        } else if count == 0 {
            Word::GoalsNone
        } else if count == 1 {
            Word::GoalsOne
        } else {
            Word::GoalsMany
        };
        let holes = [Hole::Number(u64::from(count)), Hole::Number(u64::from(underway))];
        if word != Word::GoalsNone || page.known(root.client()) {
            text(
                b,
                Role::Heading { level: 2 },
                look::GOALS_HEADLINE,
                &words.say(word, if word == Word::GoalsNone { &[] } else { &holes }),
            );
        }
    }
    b.open(Role::Paragraph);
    b.style(look::GOALS_LEDE);
    let ways = match me {
        Some(me) => ways(me, project, words),
        None => List::uncapped(),
    };
    if ways.is_empty() {
        words.text(b, Word::GoalsLede, &[]);
    } else {
        words.text(b, Word::GoalsWays, &[]);
        for (index, way) in ways.iter().enumerate() {
            if index != 0 {
                words.text(b, Word::GoalsWaySeparator, &[]);
            }
            b.text(Run::Plain, &lowercase(&way.name));
        }
        words.text(b, Word::GoalsWayEnd, &[]);
    }
    b.close();
    b.close();
}

#[expect(clippy::disallowed_types, reason = "Unicode lowercasing owns its temporary UTF-8 string")]
pub(crate) fn lowercase(name: &[u8]) -> Box<[u8]> {
    use alloc::string::String;
    String::from_utf8_lossy(name).to_lowercase().into_bytes().into_boxed_slice()
}

fn list(b: &mut Builder, root: &Domain, words: &Words, bindings: &mut Bindings, project: u32, times: &mut Times) {
    let me = root.pages().me(root.client());
    let page = root.pages().goals();
    if me.is_some()
        && let Some(page) = page
        && let Some(live) = page.live(root.client())
    {
        b.open(Role::Slot);
        b.style(look::GOALS_SECTION);
        b.open(Role::Slot);
        b.style(look::GOALS_SECTION_HEAD);
        b.open(Role::Slot);
        b.style(look::GOALS_FILTERS);
        for (filter, word) in [
            (GoalsFilter::All, Word::GoalsAll),
            (GoalsFilter::Needs, Word::GoalsNeeds),
            (GoalsFilter::Live, Word::GoalsLive),
            (GoalsFilter::Ended, Word::GoalsEnded),
        ] {
            b.open(Role::Button);
            b.name(&words.say(word, &[]));
            if page.filter == filter {
                b.state(skein_ui::State::Pressed);
            }
            b.bind(bindings.insert(Act::GoalsFilter { filter }));
            b.close();
        }
        b.close();
        if let Some(me) = me
            && !ways(me, project, words).is_empty()
        {
            let name = words.say(Word::GoalsStartDraft, &[]);
            b.open(Role::Button);
            b.name(&name);
            b.bind(bindings.insert(Act::OpenStart { draft: None, chat: None }));
            b.style(look::SHEET_LINK);
            b.close();
        }
        b.close();
        let mut any = false;
        for line in live.rows.iter().chain(page.ended()) {
            let report = page.report(line.chip.task);
            let state =
                goals::state(&line.chip, page.needs_person(line.chip.task), report.is_some(), line.delegates_total);
            if goals::shown(page.filter, state) {
                if !any {
                    b.open(Role::Slot);
                    b.style(look::GOALS_ROWS);
                    any = true;
                }
                row(b, words, bindings, times, line, state, report);
            }
        }
        if any {
            b.close();
        } else if page.known(root.client()) {
            text(b, Role::Paragraph, look::GOALS_EMPTY, &words.say(Word::GoalsEmpty, &[]));
        }
        if page.more() {
            b.open(Role::Button);
            b.style(look::GOALS_MORE);
            b.name(&words.say(Word::GoalsMore, &[]));
            b.bind(bindings.insert(Act::MoreGoals));
            b.close();
        }
        b.close();
    }
}

fn text(b: &mut Builder, role: Role, style: skein_ui::Style, value: &[u8]) {
    b.open(role);
    b.style(style);
    b.text(Run::Plain, value);
    b.close();
}

/// A goal's state word and token (client/pages/README.md, client/pages/goal.md, 2).
pub(crate) fn state_word(state: State) -> (Word, skein_ui::Style) {
    match state {
        State::Needs { funds: true } => (Word::GoalNeedsFunds, look::GOALS_STATE_NEEDS),
        State::Needs { funds: false } => (Word::GoalNeedsYou, look::GOALS_STATE_NEEDS),
        State::Paused => (Word::GoalPaused, look::GOALS_STATE_PAUSED),
        State::Planning => (Word::GoalPlanning, look::GOALS_STATE_PLANNING),
        State::Running => (Word::Running, look::GOALS_STATE_RUNNING),
        State::Landed => (Word::GoalLandedAlone, look::GOALS_STATE_LANDED),
        State::Closed => (Word::GoalClosed, look::GOALS_STATE_CLOSED),
        State::Failed => (Word::GoalFailed, look::GOALS_STATE_CLOSED),
    }
}

/// One goal links its title, state and report to its task (client/pages/README.md).
pub(crate) fn row(
    b: &mut Builder,
    words: &Words,
    bindings: &mut Bindings,
    times: &mut Times,
    line: &c::TaskLine,
    state: State,
    report: Option<&Report>,
) {
    if words.shell == Shell::Terminal {
        b.open(Role::Slot);
    }
    let (word, token) = state_word(state);
    let status = if state == State::Landed {
        match line.ended_at {
            Some(at) => words.say(Word::GoalLanded, &[Hole::Name(times.say(words, TimeForm::Clock, at))]),
            None => words.say(word, &[]),
        }
    } else {
        words.say(word, &[])
    };
    let count = if line.delegates_total == 0 {
        Box::default()
    } else {
        crate::accounts::words::join(&[
            &words.say(
                Word::GoalCount,
                &[Hole::Number(u64::from(line.delegates_done)), Hole::Number(u64::from(line.delegates_total))],
            ),
            &words.say(Word::ToolSeparator, &[]),
        ])
    };
    let title = line.chip.title.bytes();
    let name = crate::accounts::words::join(&[title, &words.say(Word::ToolSeparator, &[]), &count, &status]);
    crate::bindings::link(b, &name, bindings.insert(Act::Go { address: Address::Task { task: line.chip.task } }));
    b.style(look::GOALS_ROW);
    match state {
        State::Landed | State::Closed | State::Failed => b.style(look::GOALS_ROW_ENDED),
        State::Needs { .. } | State::Paused | State::Planning | State::Running => {}
    }
    match words.shell {
        Shell::Browser => {
            b.open(Role::Text);
            b.style(look::GOALS_TOP);
            b.text(Run::Plain, title);
            b.open(Role::Text);
            b.run(Run::Plain, &count);
            b.open(Role::Text);
            b.style(token);
            b.run(Run::Plain, &status);
            b.close();
            b.close();
            b.close();
        }
        Shell::Terminal => {
            b.text(Run::Plain, &name);
            b.close();
        }
    }
    b.open(if words.shell == Shell::Browser { Role::Text } else { Role::Paragraph });
    b.style(look::GOALS_LINE);
    let headline = headline(words, line, state, report);
    match words.shell {
        Shell::Browser => b.run(Run::Plain, &headline),
        Shell::Terminal => b.text(Run::Plain, &headline),
    }
    b.close();
    b.close();
}

pub(crate) fn headline(words: &Words, line: &c::TaskLine, state: State, report: Option<&Report>) -> Box<[u8]> {
    if let Some(report) = report {
        return report.headline.clone();
    }
    let word = match state {
        State::Planning => Word::GoalPlanningLine,
        State::Needs { .. } => {
            if line.delegates_total == 0 {
                Word::GoalPlanningLine
            } else {
                Word::GoalTasksDone
            }
        }
        State::Paused | State::Running => Word::GoalTasksDone,
        State::Landed => Word::GoalLandedLine,
        State::Closed => Word::GoalClosedLine,
        State::Failed => Word::GoalFailedLine,
    };
    if word == Word::GoalTasksDone {
        words.say(word, &[Hole::Number(u64::from(line.delegates_done)), Hole::Number(u64::from(line.delegates_total))])
    } else {
        words.say(word, &[])
    }
}
