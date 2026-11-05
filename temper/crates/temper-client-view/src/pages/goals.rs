//! The project goals' page (temper/docs/design/client/README.md, 3.5).
use crate::{
    frame::Bindings,
    look,
    words::{Shell, TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::List;
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain, PlanFilter,
    pages::{Act, goals::State, report::Report},
};

pub(crate) fn main(
    b: &mut Builder,
    _root: &Domain,
    words: &Words,
    _bindings: &mut Bindings,
    _project: u32,
    _times: &mut Times,
) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.open(Role::Heading { level: 1 });
    b.name(&words.say(Word::Goals, &[]));
    words.text(b, Word::Goals, &[]);
    b.close();
    b.close();
}

/// A project's named way of carrying out a goal (client/README.md, 3.5).
#[derive(Debug)]
pub struct Way {
    pub name: Box<[u8]>,
    pub line: Option<Box<[u8]>>,
}

/// Tracked templates in their configured number order (client/README.md, 3.5).
#[must_use]
pub fn ways(me: &c::Me, project: u32, words: &Words) -> List<Way> {
    let mut result = List::uncapped();
    for member in &me.projects {
        if member.project != project {
            continue;
        }
        let mut order = List::uncapped();
        for (index, template) in member.templates.iter().enumerate() {
            if template.tracked.is_some() {
                order
                    .push((template.number, u32::try_from(index).expect("template index fits")))
                    .expect("ways grow with templates");
            }
        }
        let mut order = order.into_boxed();
        order.sort_unstable();
        for (_, index) in &order {
            let template = member.templates.get(*index).expect("ordered template");
            let way = match template.name.bytes() {
                b"goal" => Way {
                    name: words.say(Word::GoalTaskByTask, &[]),
                    line: Some(words.say(Word::GoalTaskByTaskLine, &[])),
                },
                b"goal for a draft" => Way {
                    name: words.say(Word::GoalLandWhole, &[]),
                    line: Some(words.say(Word::GoalLandWholeLine, &[])),
                },
                name => Way { name: name.into(), line: None },
            };
            result.push(way).expect("ways grow with templates");
        }
    }
    result
}

/// One goal links its title, state and report to its task (client/README.md, 3.5).
#[cfg_attr(not(test), expect(dead_code, reason = "shared goal row awaits composition by the Goals and Home pages"))]
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
    let (word, token) = match state {
        State::Needs { funds: true } => (Word::GoalNeedsFunds, look::GOALS_STATE_NEEDS),
        State::Needs { funds: false } => (Word::GoalNeedsYou, look::GOALS_STATE_NEEDS),
        State::Paused => (Word::GoalPaused, look::GOALS_STATE_PAUSED),
        State::Planning => (Word::GoalPlanning, look::GOALS_STATE_PLANNING),
        State::Running => (Word::Running, look::GOALS_STATE_RUNNING),
        State::Landed => (Word::GoalLandedAlone, look::GOALS_STATE_LANDED),
        State::Stopped => (Word::GoalStopped, look::GOALS_STATE_STOPPED),
        State::Failed => (Word::GoalFailed, look::GOALS_STATE_STOPPED),
    };
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
    crate::link(
        b,
        &name,
        bindings.insert(Act::Go { address: Address::Task { task: line.chip.task, filter: PlanFilter::All } }),
    );
    b.style(look::GOALS_ROW);
    match state {
        State::Landed | State::Stopped | State::Failed => b.style(look::GOALS_ROW_ENDED),
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

fn headline(words: &Words, line: &c::TaskLine, state: State, report: Option<&Report>) -> Box<[u8]> {
    if let Some(report) = report {
        return report.headline.clone();
    }
    let word = match state {
        State::Planning => Word::GoalPlanningLine,
        State::Needs { .. } => {
            if line.delegates_total == 0 {
                Word::GoalPlanningLine
            } else {
                Word::GoalTasksLanded
            }
        }
        State::Paused | State::Running => Word::GoalTasksLanded,
        State::Landed => Word::GoalLandedLine,
        State::Stopped => Word::GoalStoppedLine,
        State::Failed => Word::GoalFailedLine,
    };
    if word == Word::GoalTasksLanded {
        words.say(word, &[Hole::Number(u64::from(line.delegates_done)), Hole::Number(u64::from(line.delegates_total))])
    } else {
        words.say(word, &[])
    }
}
