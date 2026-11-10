//! Private turn and live-input parts belong to their task's requester
//! (`jig/docs/design/domain/engine/executors.md`, 3, and `jig/docs/design/domain/engine/briefs.md`, 2.1).
//! Durable read owners take precedence over live tasks. Every turn read by
//! a run as its person is withheld before it can join the run's transcript.
//! Live streams and opening snapshots project copies for each follower.

use crate::{
    Core, TurnRecord,
    reads::{Answer, EndedValue, PageRow, PageTurn},
};
use alloc::boxed::Box;
use jig_domain_tasks::{Party, TaskRecord};
use jig_domain_views::{Call, Writing, WritingCall};

fn withheld(core: &Core, party: u64, owner: Option<&TaskRecord>, task: u64) -> bool {
    let owner = match owner {
        Some(owner) => Some(owner),
        None => core.tasks.task(task),
    };
    match owner {
        Some(owner) => match owner.requester {
            Party::Person(requester) => requester != party,
            Party::Task(_) | Party::Deployment { .. } => false,
        },
        None => false,
    }
}

pub(crate) fn turn(core: &Core, party: u64, owner: Option<&TaskRecord>, record: TurnRecord) -> EndedValue {
    if withheld(core, party, owner, record.task) { EndedValue::WithheldTurn(record) } else { EndedValue::Turn(record) }
}

pub(crate) fn page_turn(core: &Core, party: u64, owner: Option<&TaskRecord>, record: TurnRecord) -> PageTurn {
    if withheld(core, party, owner, record.task) { PageTurn::Withheld(record) } else { PageTurn::Carried(record) }
}

pub(crate) fn person_answer(answer: Answer) -> Answer {
    match answer {
        Answer::Ended(mut ended) => {
            ended.value = match ended.value {
                EndedValue::Turn(record) => EndedValue::WithheldTurn(record),
                value @ (EndedValue::WithheldTurn(_)
                | EndedValue::Task { .. }
                | EndedValue::Proposal(_)
                | EndedValue::PersonProposal { .. }
                | EndedValue::Escalation { .. }
                | EndedValue::Question { .. }
                | EndedValue::PersonTask { .. }
                | EndedValue::Reply(_)) => value,
            };
            Answer::Ended(ended)
        }
        Answer::Page(mut page) => {
            for row in page.rows.iter_mut() {
                match row {
                    PageRow::Activity { turn, .. } => {
                        if let Some(value) = turn.take() {
                            *turn = Some(match value {
                                PageTurn::Carried(record) => PageTurn::Withheld(record),
                                value @ (PageTurn::Withheld(_) | PageTurn::TooLarge { .. } | PageTurn::Unread) => value,
                            });
                        }
                    }
                    PageRow::Line(_)
                    | PageRow::EndedDelegate(_)
                    | PageRow::History(_)
                    | PageRow::Milestone(_)
                    | PageRow::Inbox(_)
                    | PageRow::Note(_)
                    | PageRow::Spend(_) => {}
                }
            }
            Answer::Page(page)
        }
        answer @ (Answer::TaskText(_)
        | Answer::Setting(_)
        | Answer::Revision(_)
        | Answer::Revisions(_)
        | Answer::Tasks(_)
        | Answer::Document(_)
        | Answer::None(_)) => answer,
    }
}

fn requester(core: &Core, party: u64, task: u64) -> bool {
    match core.tasks.task(task) {
        Some(task) => task.requester == Party::Person(party),
        None => false,
    }
}

fn private_tool(core: &Core, tool: &[u8]) -> bool {
    for name in &core.settings.private_tools {
        if name.as_ref() == tool {
            return true;
        }
    }
    false
}

fn call_input(core: &Core, call: &mut Option<Call>) {
    if let Some(call) = call
        && private_tool(core, &call.tool)
    {
        call.input = Box::new([]);
    }
}

fn writing_input(core: &Core, writing: &mut Option<WritingCall>) {
    if let Some(writing) = writing
        && private_tool(core, &writing.tool)
    {
        writing.input = Box::new([]);
    }
}

pub(crate) fn live(core: &Core, party: u64, task: u64, call: &mut Option<Call>, writing: &mut Option<WritingCall>) {
    if !requester(core, party, task) {
        call_input(core, call);
        writing_input(core, writing);
    }
}

/// The opening snapshot has both current calls and calls inside its writing.
pub(crate) fn snapshot(core: &Core, party: u64, task: u64, call: &mut Option<Call>, writing: &mut Option<Writing>) {
    if !requester(core, party, task) {
        call_input(core, call);
        if let Some(writing) = writing {
            call_input(core, &mut writing.call);
            writing_input(core, &mut writing.writing);
        }
    }
}
