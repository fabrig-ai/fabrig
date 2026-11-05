//! A goal's header and report (temper/docs/design/client/README.md, 5.8, 5.12).
use crate::{
    frame::Bindings,
    look,
    words::{Shell, TimeForm, Times, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::{List, bytes::copy_of};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{
        Act, delegation,
        goal::{self, Counts, Fallback},
        goals::State,
        report::Report,
    },
};

#[cfg_attr(not(test), expect(dead_code, reason = "goal-route hands over"))]
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    words: &Words,
    bindings: &mut Bindings,
    _rows: u32,
    times: &mut Times,
) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::GOAL);
    if let Some(page) = root.pages().task() {
        match page.document(root.client()) {
            Some(task) => {
                if let Some(goal) = &page.goal {
                    let ended = match &page.ended {
                        Some(ended) => ended.page.as_deref(),
                        None => None,
                    };
                    let needs = match root.pages().inbox() {
                        Some(inbox) => inbox.needs().next().is_some(),
                        None => false,
                    };
                    let shown = Page {
                        task,
                        report: goal.report.as_ref(),
                        counts: goal.counts(task, ended),
                        state: goal.state(task, ended, needs),
                    };
                    shown.header(b, words, bindings, times);
                    b.open(Role::Slot);
                    b.style(look::GOAL_TWO);
                    shown.report(b, words, times);
                    b.close();
                }
            }
            None => super::task::reading(b, page, words),
        }
    }
    b.close();
}

struct Page<'a> {
    task: &'a c::Task,
    report: Option<&'a Report>,
    counts: Counts,
    state: State,
}
impl Page<'_> {
    fn header(&self, b: &mut Builder, words: &Words, bindings: &mut Bindings, times: &mut Times) {
        b.open(Role::Banner);
        b.style(look::GOAL_HEAD);
        self.meta(b, words, times);
        let headline = match self.report {
            Some(report) => report.headline.clone(),
            None => fallback(words, goal::fallback(self.state, self.task, self.counts)),
        };
        text(b, Role::Heading { level: 1 }, &headline);
        byline(b, self.report, Word::GoalWrittenAt, words, times);
        b.open(Role::Slot);
        b.style(look::GOAL_ACTS);
        let (draft, chat) = goal::draft_of(self.task.spec.specification.words.bytes.bytes());
        if draft.is_some()
            && let Some(task) = chat
        {
            chat_link(b, task, &words.say(Word::GoalDraft, &[]), bindings);
        }
        if let c::Requester::Task(task) = self.task.requester {
            chat_link(b, task, &words.say(Word::GoalChat, &[Hole::Number(task)]), bindings);
        }
        b.close();
        b.close();
    }

    fn meta(&self, b: &mut Builder, words: &Words, times: &mut Times) {
        let (state_word, token) = super::goals::state_word(self.state);
        let mut items = List::with_capacity(5);
        items.push(copy_of(self.task.chip.title.bytes())).expect("five meta items");
        items.push(words.say(state_word, &[])).expect("five meta items");
        if self.counts.total != 0 {
            items
                .push(words.say(
                    Word::GoalMetaCount,
                    &[Hole::Number(u64::from(self.counts.done)), Hole::Number(u64::from(self.counts.total))],
                ))
                .expect("five meta items");
        }
        let numbers = self.task.budget.numbers;
        items
            .push(words.say(Word::GoalSpent, &[Hole::Number(numbers.spent), Hole::Number(numbers.budget)]))
            .expect("five meta items");
        items
            .push(words.say(Word::GoalSince, &[Hole::Name(times.say(words, TimeForm::Clock, self.task.made))]))
            .expect("five meta items");
        b.open(Role::Paragraph);
        b.style(look::GOAL_META);
        match words.shell {
            Shell::Browser => {
                for (index, item) in items.iter().enumerate() {
                    if index != 0 {
                        text(b, Role::Text, &words.say(Word::GoalSeparator, &[]));
                    }
                    b.open(Role::Text);
                    if index == 1 {
                        b.style(token);
                    }
                    b.run(if index == 0 { Run::Strong } else { Run::Plain }, item);
                    b.close();
                }
            }
            Shell::Terminal => {
                b.open(Role::Text);
                for (index, item) in items.iter().enumerate() {
                    if index != 0 {
                        b.run(Run::Plain, &words.say(Word::ToolSeparator, &[]));
                    }
                    b.run(Run::Plain, item);
                }
                b.close();
            }
        }
        b.close();
    }

    fn report(&self, b: &mut Builder, words: &Words, times: &mut Times) {
        b.open(Role::Region);
        b.name(&words.say(Word::GoalReportTitle, &[]));
        b.style(look::GOAL_REPORT);
        b.open(Role::Heading { level: 2 });
        b.style(look::GOAL_TITLE);
        b.name(&words.say(Word::GoalReportTitle, &[]));
        b.text(Run::Plain, &words.say(Word::GoalReportTitle, &[]));
        b.close();
        if let Some(report) = self.report {
            for paragraph in &report.paragraphs {
                text(b, Role::Paragraph, paragraph);
            }
        }
        match self.state {
            State::Planning => {
                let (draft, _) = goal::draft_of(self.task.spec.specification.words.bytes.bytes());
                let line = match draft {
                    Some(path) => words.say(Word::GoalReadingDraft, &[Hole::Name(copy_of(goal::file_of(&path)))]),
                    None => words.say(Word::GoalReadingAsked, &[]),
                };
                text(b, Role::Paragraph, &line);
                b.open(Role::Figure);
                b.name(&words.say(Word::GoalPlanningTitle, &[]));
                b.style(look::GOAL_TYPING);
                for _ in 0..3_u8 {
                    b.open(Role::Text);
                    if words.shell == Shell::Terminal {
                        b.run(Run::Plain, &words.say(Word::GoalsWayEnd, &[]));
                    }
                    b.close();
                }
                b.close();
            }
            State::Needs { .. } | State::Paused | State::Running | State::Landed | State::Stopped | State::Failed => {
                if let Some(report) = self.report {
                    for (word, value) in [(Word::GoalLeft, &report.left), (Word::GoalRisk, &report.risk)] {
                        if let Some(value) = value {
                            b.open(Role::Paragraph);
                            let run = if words.shell == Shell::Browser { Run::Strong } else { Run::Plain };
                            b.open(Role::Text);
                            b.run(run, &words.say(word, &[]));
                            b.run(Run::Plain, value);
                            b.close();
                            b.close();
                        }
                    }
                } else {
                    match self.state {
                        State::Landed | State::Stopped | State::Failed => match delegation::facts(self.task).said {
                            Some(delegation::Said::Words(value) | delegation::Said::Merge(value)) => {
                                skein_ui::markdown::read(value, 3, b);
                            }
                            None => {}
                        },
                        State::Needs { .. } | State::Paused | State::Planning | State::Running => {}
                    }
                }
            }
        }
        byline(b, self.report, Word::GoalByAt, words, times);
        b.close();
    }
}

fn fallback(words: &Words, value: Fallback) -> Box<[u8]> {
    match value {
        Fallback::Planning { draft: Some(draft) } => words.say(Word::GoalPlanningDraft, &[Hole::Name(draft)]),
        Fallback::Planning { draft: None } => words.say(Word::GoalPlanningAsked, &[]),
        Fallback::Stopped => words.say(Word::GoalStoppedHeading, &[]),
        Fallback::Landed { first_line: Some(line) } => line,
        Fallback::Landed { first_line: None } => words.say(Word::GoalLandedLine, &[]),
        Fallback::Failed { reason } => words.say(Word::GoalFailedReason, &[Hole::Name(reason)]),
        Fallback::Tasks(counts) => words
            .say(Word::GoalTasksLanded, &[Hole::Number(u64::from(counts.done)), Hole::Number(u64::from(counts.total))]),
    }
}
fn text(b: &mut Builder, role: Role, value: &[u8]) {
    b.open(role);
    b.name(value);
    if role == Role::Text {
        b.run(Run::Plain, value);
    } else {
        b.text(Run::Plain, value);
    }
    b.close();
}
fn byline(b: &mut Builder, report: Option<&Report>, word: Word, words: &Words, times: &mut Times) {
    if let Some(report) = report
        && let Some(at) = report.at
    {
        b.open(Role::Paragraph);
        b.style(look::GOAL_BYLINE);
        b.text(Run::Plain, &words.say(word, &[Hole::Name(times.say(words, TimeForm::Clock, at))]));
        b.close();
    }
}
fn chat_link(b: &mut Builder, task: u64, label: &[u8], bindings: &mut Bindings) {
    crate::link(b, label, bindings.insert(Act::Go { address: Address::Chat { task } }));
    b.style(look::GOAL_BTN);
    b.style(look::GOAL_SMALL);
    b.text(Run::Plain, label);
    b.close();
}
