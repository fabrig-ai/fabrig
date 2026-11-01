//! A deleted project's own rows (jig/docs/design/domain/core.md, 11).
//! Period and pool spending outlives the project; recurring spending belongs to its period.
use crate::{
    Core, CoreKey, Event, Held, Key, Limits, Now, ProposalDecisionRecord, Record, Request, Requests, Write,
    reads::Sweep,
};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_notes as notes;
use jig_domain_people::Proposer;
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, Token, Wall};

/// One project's ordered row sweeps; volatile, restarted by the deletion.
#[derive(Debug)]
pub(crate) struct RowSweep {
    // Proposal keys carry no project, so retain it across the final stage.
    project: u32,
    sweep: Sweep,
    waiting: List<Waiting>,
    more: bool,
}

#[derive(Clone, Copy, Debug)]
struct Waiting {
    proposer: u64,
    proposal: u64,
    first: u64,
}

/// Erase a deleted project's own rows; completes in `rows_done`.
pub(crate) fn erase_rows(
    core: &mut Core,
    _env: &Env<Limits>,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    project: u32,
) {
    work.push(Event::Tasks(tasks::Event::ProjectGone { project }));
    work.push(Event::Account(accounts::Event::ProjectGone { project }));
    work.push(Event::Notes(notes::Event::ProjectGone { project }));
    drop(core.permission_roles.remove(&project));
    drop(core.template_overrides.remove(&project));
    let owner = Token::new(core.next_result_request);
    core.next_result_request = core.next_result_request.checked_add(1).expect("transient sweep owner space");
    let sweep = Sweep::Notes { project };
    let state = RowSweep { project, sweep, waiting: List::with_capacity(0), more: false };
    assert!(core.row_sweeps.insert(owner, state).is_ok(), "one row sweep per deleted project");
    request_sweep(out, owner, sweep);
}

/// A page of the project's notes, archived recurring ledgers or decided proposals.
pub(crate) fn swept(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    owner: Token,
    rows: Box<[Record]>,
    more: bool,
) -> Requests {
    let mut out = Queue::uncapped();
    let Some(mut state) = core.row_sweeps.remove(&owner) else {
        return crate::routing::removal::swept(core, env, work, owner, rows, more);
    };
    assert!(state.waiting.is_empty(), "a page follows its decision loads");
    let proposals_page = match state.sweep {
        Sweep::Proposals { .. } => true,
        Sweep::Notes { .. } | Sweep::Recurring { .. } => false,
        Sweep::Ended { .. } | Sweep::Task(_) => unreachable!("project row sweep"),
    };
    state.more = more;
    let mut greatest = None;
    state.waiting = List::with_capacity(u32::try_from(rows.len()).expect("bounded sweep page"));
    for row in rows {
        let key = row.key();
        assert!(state.sweep.admits(&key), "rows belong to the requested sweep");
        match state.sweep {
            Sweep::Notes { .. } => {
                let Key::Notes(notes::Key::Line { name, .. }) = &key else { unreachable!("notes sweep line") };
                erase(&mut out, Key::Notes(notes::Key::Entry { name: *name }));
                erase(&mut out, key);
            }
            Sweep::Recurring { .. } => erase(&mut out, key),
            Sweep::Proposals { .. } => {
                let Record::Tasks(tasks::Stored::DecidedProposal(proposal)) = row else {
                    unreachable!("decided proposal sweep")
                };
                greatest = Some(match greatest {
                    Some(previous) => proposal.number.max(previous),
                    None => proposal.number,
                });
                if proposal.project == state.project {
                    let waiting =
                        Waiting { proposer: proposal.proposer, proposal: proposal.number, first: proposal.first() };
                    match proposal.state {
                        tasks::PersonProposalState::Rejected { at, .. }
                        | tasks::PersonProposalState::Withdrawn { at } => {
                            erase_proposal(&mut out, &waiting, Some(at));
                        }
                        tasks::PersonProposalState::Accepted { .. }
                        | tasks::PersonProposalState::SettingsAccepted { .. } => {
                            state.waiting.push(waiting).expect("at most one load per page row");
                        }
                        tasks::PersonProposalState::Pending { .. } => unreachable!("decided proposal is terminal"),
                    }
                }
            }
            Sweep::Ended { .. } | Sweep::Task(_) => unreachable!("project row sweep"),
        }
    }
    if let Some(greatest) = greatest {
        state.sweep = Sweep::Proposals { after: Some(greatest) };
    }
    match state.sweep {
        Sweep::Notes { .. } if !more => state.sweep = Sweep::Recurring { project: state.project },
        Sweep::Recurring { .. } if !more => state.sweep = Sweep::Proposals { after: None },
        Sweep::Notes { .. } | Sweep::Recurring { .. } | Sweep::Proposals { .. } => {}
        Sweep::Ended { .. } | Sweep::Task(_) => unreachable!("project row sweep"),
    }
    // A finished notes/recurring stage always requests the next stage's first page.
    if proposals_page {
        continue_proposals(core, env, work, &mut out, owner, state);
    } else {
        request_sweep(&mut out, owner, state.sweep);
        core.row_sweeps.insert(owner, state).expect("removed sweep owner");
    }
    Requests::Out(out)
}

/// An accepted proposal's archived decision, loaded for its `Update`'s time.
pub(crate) fn decision_loaded(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    owner: Token,
    row: Option<ProposalDecisionRecord>,
    busy: bool,
) -> Requests {
    let mut out = Queue::uncapped();
    let Some(mut state) = core.row_sweeps.remove(&owner) else { return Requests::Out(out) };
    if busy {
        load_decision(&mut out, owner, &state);
        core.row_sweeps.insert(owner, state).expect("removed sweep owner");
        return Requests::Out(out);
    }
    let Some(&waiting) = state.waiting.last() else {
        core.row_sweeps.insert(owner, state).expect("removed sweep owner");
        return Requests::Out(out);
    };
    state.waiting.truncate(state.waiting.len().checked_sub(1).expect("waiting proposal"));
    let at = match row {
        Some(row) => {
            assert!(
                row.project == state.project
                    && row.proposal == waiting.proposal
                    && row.proposer == Proposer::Party(waiting.proposer),
                "loaded decision names the waiting proposal"
            );
            Some(row.at)
        }
        None => None,
    };
    erase_proposal(&mut out, &waiting, at);
    continue_proposals(core, env, work, &mut out, owner, state);
    Requests::Out(out)
}

fn continue_proposals(
    core: &mut Core,
    env: &Env<Limits>,
    work: &mut Queue<Event>,
    out: &mut Queue<Request>,
    owner: Token,
    state: RowSweep,
) {
    if !state.waiting.is_empty() {
        load_decision(out, owner, &state);
    } else if state.more {
        request_sweep(out, owner, state.sweep);
    } else {
        rows_done(core, env, work, out, state.project);
        return;
    }
    core.row_sweeps.insert(owner, state).expect("removed sweep owner");
}

fn load_decision(out: &mut Queue<Request>, owner: Token, state: &RowSweep) {
    let waiting = state.waiting.last().expect("one accepted proposal waiting");
    out.push(Request::Now(Box::new(Now::HistoricalProposal {
        request: owner,
        person: waiting.proposer,
        project: state.project,
        proposer: Proposer::Party(waiting.proposer),
        proposal: waiting.proposal,
    })));
}

fn erase_proposal(out: &mut Queue<Request>, waiting: &Waiting, at: Option<Wall>) {
    if let Some(at) = at {
        let update = tasks::Update {
            party: waiting.proposer,
            at,
            task: waiting.first,
            object: tasks::UpdateObject::Proposal { proposal: tasks::Newest::of(waiting.proposal) },
        };
        erase(out, Key::Tasks(update.key()));
    }
    erase(
        out,
        Key::Core(CoreKey::ProposalDecision {
            proposer: Proposer::Party(waiting.proposer),
            proposal: waiting.proposal,
        }),
    );
    erase(out, Key::Tasks(tasks::Key::DecidedProposal(waiting.proposal)));
}

fn erase(out: &mut Queue<Request>, key: Key) {
    out.push(Request::Write(Write::Erase(key)));
}

fn request_sweep(out: &mut Queue<Request>, owner: Token, sweep: Sweep) {
    out.push(Request::Held(Box::new(Held::Sweep { owner, sweep })));
}

/// `pd-delete` continues the deletion here.
fn rows_done(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Event>, out: &mut Queue<Request>, project: u32) {
    crate::routing::removal::rows_erased(core, env, work, out, project);
}
