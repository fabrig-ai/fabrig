//! Point-load continuations (jig/docs/design/domain/engine/decisions.md, 1.3).
use crate::{
    CallKey, Core, CoreKey, CoreRecord, Event, Key, Limits, Record, Request,
    engine::store,
    routing::{
        Work, ended_marks,
        party::{
            dispatch::{Party, Routed},
            ended, inputs,
        },
        run::{delegation, results},
    },
};
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, ReplyTo, Token};

#[derive(Hash, Debug)]
pub(crate) enum Point {
    Result { request: Token, task: u64 },
    SeenConversation { request: Token, task: u64 },
    EndedRead { request: Token, task: u64, party: u64 },
    PersonTask { party: Party, task: u64 },
    PartyInput { request: Token, task: u64 },
    DelegateInput(Box<delegation::DelegateInput>),
    ProposalDecision { party: Party, proposer: people::Proposer, proposal: u64 },
    EscalationDecision { party: Party, task: u64, revision: u64 },
    Proposal { to: ReplyTo, key: CallKey, proposal: u64 },
}

impl Point {
    pub(super) fn is_result(&self, token: Token) -> bool {
        if let Self::Result { request, .. } = self { *request == token } else { false }
    }

    pub(super) fn range(&self) -> store::Range {
        let key = match self {
            Self::Result { task, .. }
            | Self::SeenConversation { task, .. }
            | Self::PersonTask { task, .. }
            | Self::PartyInput { task, .. } => Key::Tasks(tasks::Key::Ended(*task)),
            Self::DelegateInput(read) => Key::Tasks(tasks::Key::Ended(*read.ids.get(read.at).expect("pending input"))),
            Self::EndedRead { task, party, .. } => Key::People(people::Key::EndedRead { task: *task, party: *party }),
            Self::ProposalDecision { proposer, proposal, .. } => {
                Key::Core(CoreKey::ProposalDecision { proposer: *proposer, proposal: *proposal })
            }
            Self::EscalationDecision { task, revision, .. } => {
                Key::Core(CoreKey::EscalationDecision { task: *task, revision: *revision })
            }
            Self::Proposal { key, proposal, .. } => {
                return store::Range::Proposal { task: key.task, proposal: *proposal };
            }
        };
        store::Range::Record(key)
    }
}

pub(crate) fn party(core: &mut Core, limits: &Limits, routed: &mut Routed, request: Token, point: Point) {
    if !super::try_point(core, limits, &mut routed.out, point) {
        crate::routing::refusals::pool_again(&mut routed.work, request, people::Again::Busy);
    }
}

pub(super) fn loaded(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Work>,
    point: Point,
    answer: store::Answer,
) -> Queue<Request> {
    let (row, busy) = match answer {
        store::Answer::Rows(store::Rows::Records { records, more: false }) if records.len() <= 1 => {
            (checked_row(&point.range(), records.into_iter().next()), false)
        }
        store::Answer::Failed(_) | store::Answer::Cut { .. } | store::Answer::Rows(_) => (None, true),
    };
    match point {
        Point::Result { request, .. } => results::loaded(core, env, work, request, ended_task(row), busy),
        Point::SeenConversation { request, .. } => {
            crate::inbox::seen_conversation_loaded(core, env, work, request, ended_task(row), busy)
        }
        Point::EndedRead { request, task, party } => {
            let through =
                if let Some(Record::People(people::Stored::EndedRead { through, .. })) = row { through } else { 0 };
            ended_marks::loaded(core, env, work, ended_marks::Answer { request, task, party, through, busy })
        }
        Point::PersonTask { party, task } => {
            let outcome = core.person_task_ended(party, task, ended_task(row), busy);
            work.push(Work::Event(Event::People(people::Event::Decided { request: party.request, outcome })));
            Queue::uncapped()
        }
        Point::PartyInput { request, .. } => {
            let mut routed = Routed::new(work);
            inputs::loaded(core, env, &mut routed, inputs::Loaded { request, row: ended_task(row), busy });
            routed.finish(work)
        }
        Point::DelegateInput(read) => delegation::input_loaded(core, env, work, *read, ended_task(row)),
        Point::ProposalDecision { party, proposer, proposal } => {
            let row = if let Some(Record::Core(CoreRecord::ProposalDecision(row))) = row { Some(row) } else { None };
            ended::route_ended_proposal(core, work, party, proposer, proposal, row, busy)
        }
        Point::EscalationDecision { party, task, revision } => {
            let row = if let Some(Record::Core(CoreRecord::EscalationDecision(row))) = row { Some(row) } else { None };
            ended::route_ended_escalation(core, env, work, party, ended::Escalation { task, revision, row, busy })
        }
        Point::Proposal { to, key, proposal } => {
            let row = if let Some(Record::Tasks(tasks::Stored::History(row))) = row {
                match row.proposal {
                    Some(proposal) => Some(*proposal),
                    None => None,
                }
            } else {
                None
            };
            ended::decided_proposal(core, to, key, proposal, row, busy)
        }
    }
}

fn checked_row(range: &store::Range, row: Option<Record>) -> Option<Record> {
    let row = row?;
    match range {
        store::Range::Record(key) => (&row.key() == key).then_some(row),
        store::Range::Proposal { task, .. } => {
            if let Record::Tasks(tasks::Stored::History(history)) = &row
                && history.task == *task
            {
                return Some(row);
            }
            None
        }
        store::Range::Read(_) | store::Range::Window { .. } | store::Range::Notes(_) => {
            unreachable!("point load range")
        }
    }
}

fn ended_task(row: Option<Record>) -> Option<Box<tasks::TaskRecord>> {
    if let Some(Record::Tasks(tasks::Stored::Ended(row))) = row { Some(row) } else { None }
}
