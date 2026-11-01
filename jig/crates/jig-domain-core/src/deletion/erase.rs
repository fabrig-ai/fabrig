//! Bounded whole-task erasure (protocol/store.md, 8.5 and 11.1;
//! temper/docs/plan/playground-reset-r6/notes/contracts.md, J4 and J6).
//! The ended owner survives every page until the final task decision. A start
//! discards this transient scan and visits that owner again from the beginning.
//!
//! | State | Answer | Next | Writes |
//! | --- | --- | --- | --- |
//! | absent | start or Erasing saved | project | none |
//! | project | ended owner | task | none |
//! | project | decided proposal | proposal | none |
//! | project | filtered page | project or absent | Releasing at end |
//! | task | continued page | task | rows and reply indexes |
//! | task | final page | project or absent | rows, owner, result, people, transcript |
//! | proposal | joined decision | project or absent | proposal, decision and update |
//! | any load | busy | same phase, new load | none |
//! | any | stale answer | same | none |
//! | any load | missing or malformed range | fail-stop | none |
use crate::reads::{Loaded, Position, Range, Scope};
use crate::{Core, CoreKey, CoreRecord, Held, Key, Limits, Record, Request, Requests, Write};
use alloc::boxed::Box;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, Token};

// At most 64 erases per task page: its rows and one reply index per activity.
// Including the final task, Releasing and deployment writes, one decision takes
// at most 74 + limits.tasks.placed_holds + 2 * limits.people.people writes.
const TASK_ROWS: u32 = 32;

#[derive(Debug)]
pub(crate) struct Scan {
    project: u32,
    load: Token,
    project_next: Option<Position>,
    phase: Phase,
}

#[derive(Debug)]
enum Phase {
    Project { after: Option<Position> },
    Task { record: Box<tasks::TaskRecord>, after: Option<Position> },
    Proposal { record: Box<tasks::PersonProposal> },
}

/// Slab slots never have index `u32::MAX`; these tokens belong only to this scan.
pub(crate) fn owns(load: Token) -> bool {
    load.raw() >> 32 == u64::from(u32::MAX)
}

pub(crate) fn begin(core: &mut Core, _env: &Env<Limits>, out: &mut Queue<Request>) {
    if core.erasing.is_some() || !core.people.ready() {
        return;
    }
    let mut project = None;
    for (&number, &stage) in core.people.deletions() {
        if stage == people::Stage::Erasing {
            project = Some(number);
            break;
        }
    }
    let Some(project) = project else { return };
    let mut scan = Scan { project, load: Token::new(0), project_next: None, phase: Phase::Project { after: None } };
    ask(core, &mut scan, out);
    core.erasing = Some(scan);
}

fn ask(core: &mut Core, scan: &mut Scan, out: &mut Queue<Request>) {
    core.erase_load = core.erase_load.checked_add(1).expect("transient erasure load space");
    scan.load = Token::new((u64::from(u32::MAX) << 32) | u64::from(core.erase_load));
    let (party, scope, after, rows) = match &scan.phase {
        Phase::Project { after } => (0, Scope::ProjectRows { project: scan.project }, after.clone(), 1),
        Phase::Task { record, after } => (0, Scope::TaskRows { task: record.number }, after.clone(), TASK_ROWS),
        // Accepted proposals keep their decision time in the joined archive,
        // not in PersonProposal. Load that evidence before erasing its update.
        Phase::Proposal { record } => (
            record.proposer,
            Scope::Proposal { proposer: people::Proposer::Party(record.proposer), proposal: record.number },
            None,
            1,
        ),
    };
    // load_bytes bounds authenticated reads. Erasure must visit every stored
    // row even when that budget cannot carry one. The row count and the store's
    // admitted record bounds limit this core-owned page and its proposal join.
    let range = Range { party, scope, after, backwards: false, rows, bytes: u64::MAX };
    out.push(Request::Held(Box::new(Held::ReadLoad { load: scan.load, range: Box::new(range) })));
}

pub(crate) fn loaded(core: &mut Core, env: &Env<Limits>, load: Token, result: Loaded) -> Requests {
    let mut out = Queue::uncapped();
    out.push(Request::Decided);
    let Some(mut scan) = core.erasing.take() else { return Requests::Out(out) };
    if scan.load != load {
        core.erasing = Some(scan);
        return Requests::Out(out);
    }
    let (rows, next) = match result {
        Loaded::Page { rows, next } => (rows, next),
        Loaded::Busy => {
            ask(core, &mut scan, &mut out);
            core.erasing = Some(scan);
            return Requests::Out(out);
        }
        Loaded::Gone | Loaded::TooLinked { .. } | Loaded::TurnProblem { .. } => {
            unreachable!("erasure ranges retain raw rows and proposal decision evidence")
        }
    };
    let phase = core::mem::replace(&mut scan.phase, Phase::Project { after: None });
    match phase {
        Phase::Project { .. } => {
            assert!(rows.len() <= 1, "one project owner per page");
            scan.project_next = next;
            if let Some(row) = rows.into_iter().next() {
                match row.value {
                    Record::Tasks(tasks::Stored::Ended(record)) => {
                        assert!(record.project == scan.project, "project scan owner");
                        scan.phase = Phase::Task { record, after: None };
                        ask(core, &mut scan, &mut out);
                        core.erasing = Some(scan);
                        return Requests::Out(out);
                    }
                    Record::Tasks(tasks::Stored::DecidedProposal(record)) => {
                        assert!(record.project == scan.project, "project proposal owner");
                        scan.phase = Phase::Proposal { record };
                        ask(core, &mut scan, &mut out);
                        core.erasing = Some(scan);
                        return Requests::Out(out);
                    }
                    Record::Tasks(_) | Record::Core(_) | Record::People(_) | Record::Notes(_) | Record::Accounts(_) => {
                        unreachable!("project scan returns ended tasks and decided person proposals")
                    }
                }
            }
        }
        Phase::Task { record, .. } => {
            assert!(rows.len() <= usize::try_from(TASK_ROWS).expect("page bound fits"), "bounded task page");
            for row in rows {
                erase_row(&mut out, row.value, record.requester);
            }
            if let Some(after) = next {
                scan.phase = Phase::Task { record, after: Some(after) };
                ask(core, &mut scan, &mut out);
                core.erasing = Some(scan);
                return Requests::Out(out);
            }
            finish(core, env, record, &mut out);
        }
        Phase::Proposal { record } => {
            assert!(rows.len() == 1 && next.is_none(), "decided proposal remains until its erasure");
            for row in rows {
                let Some(Record::Core(CoreRecord::ProposalDecision(decision))) = row.related else {
                    unreachable!("decided person proposal has its durable decision time")
                };
                let update = crate::indexes::proposal_update(&record, decision.at);
                erase(&mut out, Key::Tasks(update.key()));
                erase(
                    &mut out,
                    Key::Core(CoreKey::ProposalDecision {
                        proposer: people::Proposer::Party(record.proposer),
                        proposal: record.number,
                    }),
                );
                erase(&mut out, Key::Tasks(tasks::Key::DecidedProposal(record.number)));
            }
        }
    }
    match scan.project_next.take() {
        Some(after) => {
            scan.phase = Phase::Project { after: Some(after) };
            ask(core, &mut scan, &mut out);
            core.erasing = Some(scan);
        }
        None => {
            crate::routing::children::append_people(
                core,
                env,
                people::Event::Deletion { project: scan.project, stage: Some(people::Stage::Releasing) },
                &mut out,
            );
            begin(core, env, &mut out);
        }
    }
    Requests::Out(out)
}

fn erase(out: &mut Queue<Request>, key: Key) {
    out.push(Request::Write(Write::Erase(key)));
}

fn finish(core: &mut Core, env: &Env<Limits>, record: Box<tasks::TaskRecord>, out: &mut Queue<Request>) {
    let task = record.number;
    let project = record.project;
    if let Some(update) = crate::indexes::result_update(&record) {
        erase(out, Key::Tasks(update.key()));
        erase(out, Key::Tasks(tasks::Key::Result { party: update.party, position: record.result_position }));
    }
    drop(core.archived_chats.remove(&task));
    crate::routing::children::append_tasks(core, env, tasks::Event::Erase { record }, out);
    out.push(Request::Write(Write::DropTranscript { task }));
    crate::routing::children::append_people(core, env, people::Event::EraseTask { task, project }, out);
}

fn erase_row(out: &mut Queue<Request>, row: Record, requester: tasks::Party) {
    let update = match &row {
        Record::Tasks(record) => match record {
            tasks::Stored::Activity(line) => crate::indexes::activity_update(line, Some(requester)),
            tasks::Stored::Live(_)
            | tasks::Stored::Ended(_)
            | tasks::Stored::Stub(_)
            | tasks::Stored::Ledger(_)
            | tasks::Stored::Writer(_)
            | tasks::Stored::Pool(_)
            | tasks::Stored::PersonProposal(_)
            | tasks::Stored::DecidedProposal(_)
            | tasks::Stored::History(_)
            | tasks::Stored::Milestone(_)
            | tasks::Stored::Update(_)
            | tasks::Stored::Result(_)
            | tasks::Stored::EndedByParty(_)
            | tasks::Stored::EndedByProject(_)
            | tasks::Stored::EndedDelegate(_)
            | tasks::Stored::Delegate(_)
            | tasks::Stored::PlacedHold(_)
            | tasks::Stored::ClosedLedger(_) => None,
        },
        Record::Core(_) | Record::People(_) | Record::Notes(_) | Record::Accounts(_) => None,
    };
    if let Some(update) = update {
        erase(out, Key::Tasks(update.key()));
    }
    erase(out, row.key());
}
