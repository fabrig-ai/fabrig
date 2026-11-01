//! A project's ordered erasure keeps spending and removes owned rows (draft 5).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "unit fixtures use ordinary Rust (programming-model.md, 10.2)"
)]
use super::*;
use crate::project_rows::erase_rows;
use crate::reads::Sweep;
use alloc::collections::BTreeSet;

const PROJECT: u32 = 1;
const PARTY: u64 = 1;

fn fixture() -> (Core, Env<Limits>) {
    let (mut core, limits) = core();
    send_task(&mut core, &limits, tasks::Event::Check);
    send_task(&mut core, &limits, tasks::Event::Restored);
    (core, Env { now: Time::ZERO, wall: Wall::from_nanos(1000), limits })
}

fn begin(core: &mut Core, env: &Env<Limits>) -> (Token, Queue<Request>) {
    let mut work = Queue::uncapped();
    let mut out = Queue::uncapped();
    erase_rows(core, env, &mut work, &mut out, PROJECT);
    while let Some(event) = work.pop() {
        let Requests::Out(mut child) = step(core, env, event);
        while let Some(request) = child.pop() {
            out.push(request);
        }
    }
    (sweep(&out).0, out)
}

fn sweep(out: &Queue<Request>) -> (Token, Sweep) {
    for request in out {
        if let Request::Held(held) = request
            && let Held::Sweep { owner, sweep } = held.as_ref()
        {
            return (*owner, *sweep);
        }
    }
    panic!("held row sweep")
}

fn erased(out: &Queue<Request>) -> BTreeSet<Key> {
    let mut keys = BTreeSet::new();
    for request in out {
        if let Request::Write(Write::Erase(key)) = request {
            keys.insert(key.clone());
        }
    }
    keys
}

fn page(core: &mut Core, env: &Env<Limits>, owner: Token, rows: Box<[Record]>, more: bool) -> Queue<Request> {
    let Requests::Out(out) = step(core, env, Event::Swept { owner, rows, more });
    out
}

fn proposals(core: &mut Core, env: &Env<Limits>) -> Token {
    let (owner, _) = begin(core, env);
    let out = page(core, env, owner, Box::new([]), false);
    assert_eq!(sweep(&out), (owner, Sweep::Recurring { project: PROJECT }));
    let out = page(core, env, owner, Box::new([]), false);
    assert_eq!(sweep(&out), (owner, Sweep::Proposals { after: None }));
    owner
}

fn proposal(number: u64, project: u32, state: tasks::PersonProposalState) -> Record {
    Record::Tasks(tasks::Stored::DecidedProposal(Box::new(tasks::PersonProposal {
        number,
        project,
        proposer: PARTY,
        action: tasks::PersonAction::Amend { task: 8, revision: 1, amendment: Box::new(fixtures::unchanged(b"")) },
        reason: Box::new([]),
        state,
    })))
}

fn decision_keys(number: u64, at: Option<Wall>) -> BTreeSet<Key> {
    let mut keys = BTreeSet::from([
        Key::Tasks(tasks::Key::DecidedProposal(number)),
        Key::Core(CoreKey::ProposalDecision { proposer: Proposer::Party(PARTY), proposal: number }),
    ]);
    if let Some(at) = at {
        let update = tasks::Update {
            party: PARTY,
            at,
            task: 8,
            object: tasks::UpdateObject::Proposal { proposal: tasks::Newest::of(number) },
        };
        keys.insert(Key::Tasks(update.key()));
    }
    keys
}

#[test]
fn erasing_a_projects_rows_closes_its_ledgers_and_sweeps_its_notes() {
    let (mut core, env) = fixture();
    core.permission_roles.insert(PROJECT, Box::new([])).unwrap();
    core.template_overrides.insert(PROJECT, Box::new([])).unwrap();
    let period = tasks::Funder::Period { project: PROJECT, period: 1 };
    let _out = step(
        &mut core,
        &env,
        Event::Tasks(tasks::Event::OpenPeriod {
            reply_to: ReplyTo::new(Token::new(90)),
            project: PROJECT,
            period: 1,
            budget: 100,
        }),
    );
    assert!(core.tasks.funding(period).is_some());
    let (owner, out) = begin(&mut core, &env);
    assert_eq!(sweep(&out), (owner, Sweep::Notes { project: PROJECT }));
    assert_eq!(core.row_sweeps.capacity(), env.limits.authority.projects);
    assert!(core.tasks.funding(period).is_none());
    assert!(core.permission_roles.is_empty());
    assert!(core.template_overrides.is_empty());
    assert!(erased(&out).contains(&Key::Tasks(tasks::Key::Ledger(period))));
    assert!(out.iter().any(|request| {
        if let Request::Write(Write::Save(Record::Tasks(tasks::Stored::ClosedLedger(row)))) = request {
            row.funder == period && row.closed
        } else {
            false
        }
    }));
}

#[test]
fn a_notes_page_erases_each_line_and_its_entry() {
    let (mut core, env) = fixture();
    let (owner, _) = begin(&mut core, &env);
    let scopes = [
        notes::Scope::Project { project: PROJECT },
        notes::Scope::Goal { project: PROJECT, goal: 8 },
        notes::Scope::Resources {
            project: PROJECT,
            connector: 1,
            pattern: notes::Pattern { segments: Box::new([]), last: notes::Last::Open(Box::new([])) },
        },
    ];
    let mut expected = BTreeSet::new();
    let rows = scopes
        .into_iter()
        .enumerate()
        .map(|(index, scope)| {
            let name = u64::try_from(index + 1).unwrap();
            expected.insert(Key::Notes(notes::Key::Line { scope: scope.clone(), name }));
            expected.insert(Key::Notes(notes::Key::Entry { name }));
            Record::Notes(notes::Record::Line(notes::Line { scope, name, description: Box::new([]), revision: 1 }))
        })
        .collect();
    let out = page(&mut core, &env, owner, rows, true);
    assert_eq!(erased(&out), expected);
    assert_eq!(sweep(&out), (owner, Sweep::Notes { project: PROJECT }));
}

#[test]
fn the_last_notes_page_sweeps_the_closed_recurring_ledgers() {
    let (mut core, env) = fixture();
    let (owner, _) = begin(&mut core, &env);
    let out = page(&mut core, &env, owner, Box::new([]), false);
    assert_eq!(sweep(&out), (owner, Sweep::Recurring { project: PROJECT }));
    let funder = tasks::Funder::Recurring { project: PROJECT, task: 8, period: 1 };
    let row = Record::Tasks(tasks::Stored::ClosedLedger(tasks::FundingRecord {
        funder,
        parent: Some(tasks::Funder::Period { project: PROJECT, period: 1 }),
        period: None,
        numbers: tasks::Numbers { budget: 10, spent: 2, spent_below: 3, reserved: 0 },
        made: 1,
        closed: true,
    }));
    let out = page(&mut core, &env, owner, Box::new([row]), true);
    assert_eq!(erased(&out), BTreeSet::from([Key::Tasks(tasks::Key::ClosedLedger(funder))]));
    assert_eq!(sweep(&out), (owner, Sweep::Recurring { project: PROJECT }));
    let out = page(&mut core, &env, owner, Box::new([]), false);
    assert_eq!(sweep(&out), (owner, Sweep::Proposals { after: None }));
    let out = page(&mut core, &env, owner, Box::new([]), false);
    assert_eq!(out.len(), 1);
    let Some(Request::Decided) = out.iter().next() else { panic!("only the completion marker remains") };
    assert!(core.row_sweeps.is_empty());
}

#[test]
fn a_proposals_page_erases_only_the_projects_decisions_and_their_updates() {
    let (mut core, env) = fixture();
    let owner = proposals(&mut core, &env);
    let at = Wall::from_nanos(20);
    let rows = Box::new([
        proposal(
            1,
            PROJECT,
            tasks::PersonProposalState::Rejected {
                by: tasks::Party::Person(PARTY),
                reason: Box::new([]),
                message: 8,
                at,
            },
        ),
        proposal(2, PROJECT, tasks::PersonProposalState::Withdrawn { at }),
        proposal(3, 2, tasks::PersonProposalState::Withdrawn { at }),
    ]);
    let out = page(&mut core, &env, owner, rows, false);
    let expected = decision_keys(1, Some(at)).union(&decision_keys(2, Some(at))).cloned().collect();
    assert_eq!(erased(&out), expected);
    assert!(core.row_sweeps.is_empty());
}

#[test]
fn a_proposals_page_of_other_projects_rows_sweeps_on_past_them() {
    let (mut core, env) = fixture();
    let owner = proposals(&mut core, &env);
    let at = Wall::from_nanos(20);
    // Two retained rows fill a page; the next request must advance past both.
    let out = page(
        &mut core,
        &env,
        owner,
        Box::new([
            proposal(1, 2, tasks::PersonProposalState::Withdrawn { at }),
            proposal(2, 2, tasks::PersonProposalState::Withdrawn { at }),
        ]),
        true,
    );
    assert!(erased(&out).is_empty());
    assert_eq!(sweep(&out), (owner, Sweep::Proposals { after: Some(2) }));
    let out = page(
        &mut core,
        &env,
        owner,
        Box::new([proposal(3, PROJECT, tasks::PersonProposalState::Withdrawn { at })]),
        false,
    );
    assert_eq!(erased(&out), decision_keys(3, Some(at)));
    assert!(core.row_sweeps.is_empty());
}

fn loaded(
    core: &mut Core,
    env: &Env<Limits>,
    owner: Token,
    row: Option<ProposalDecisionRecord>,
    busy: bool,
) -> Queue<Request> {
    let Requests::Out(out) = step(
        core,
        env,
        Event::HistoricalProposal {
            request: owner,
            person: PARTY,
            project: PROJECT,
            proposer: Proposer::Party(PARTY),
            proposal: 1,
            row,
            busy,
        },
    );
    out
}

fn load(out: &Queue<Request>) -> (Token, u64) {
    let mut loads = Vec::new();
    for request in out {
        if let Request::Now(now) = request
            && let Now::HistoricalProposal { request, person, project, proposer, proposal } = now.as_ref()
        {
            assert_eq!((*person, *project, *proposer), (PARTY, PROJECT, Proposer::Party(PARTY)));
            loads.push((*request, *proposal));
        }
    }
    assert_eq!(loads.len(), 1);
    loads[0]
}

#[test]
fn an_accepted_proposals_update_goes_at_its_loaded_decision_time() {
    let (mut core, env) = fixture();
    let owner = proposals(&mut core, &env);
    let at = Wall::from_nanos(20);
    let out = page(
        &mut core,
        &env,
        owner,
        Box::new([
            proposal(1, PROJECT, tasks::PersonProposalState::Accepted { by: tasks::Party::Person(PARTY) }),
            proposal(
                2,
                PROJECT,
                tasks::PersonProposalState::Rejected {
                    by: tasks::Party::Person(PARTY),
                    reason: Box::new([]),
                    message: 8,
                    at,
                },
            ),
            proposal(3, 2, tasks::PersonProposalState::Withdrawn { at }),
        ]),
        false,
    );
    assert_eq!(erased(&out), decision_keys(2, Some(at)));
    assert_eq!(load(&out), (owner, 1));
    let retry = loaded(&mut core, &env, owner, None, true);
    assert!(erased(&retry).is_empty());
    assert_eq!(load(&retry), (owner, 1));
    let decided_at = Wall::from_nanos(77);
    let row = ProposalDecisionRecord {
        project: PROJECT,
        proposer: Proposer::Party(PARTY),
        proposal: 1,
        kind: tasks::ProposalKind::Amend,
        by: PARTY,
        at: decided_at,
        choice: people::ProposalChoice::Accepted,
        created: None,
    };
    let out = loaded(&mut core, &env, owner, Some(row), false);
    assert_eq!(erased(&out), decision_keys(1, Some(decided_at)));
    assert!(core.row_sweeps.is_empty());
}

#[test]
fn missing_decisions_erase_their_keys_and_finish_before_a_stale_answer() {
    let (mut core, env) = fixture();
    let owner = proposals(&mut core, &env);
    let out = page(
        &mut core,
        &env,
        owner,
        Box::new([proposal(
            1,
            PROJECT,
            tasks::PersonProposalState::SettingsAccepted { by: tasks::Party::Person(PARTY), revision: 2 },
        )]),
        true,
    );
    assert_eq!(load(&out), (owner, 1));
    let out = loaded(&mut core, &env, owner, None, false);
    assert_eq!(erased(&out), decision_keys(1, None));
    assert_eq!(sweep(&out), (owner, Sweep::Proposals { after: Some(1) }));
    let out = page(&mut core, &env, owner, Box::new([]), false);
    assert_eq!(out.len(), 1);
    let Some(Request::Decided) = out.iter().next() else { panic!("only the completion marker remains") };
    assert!(core.row_sweeps.is_empty());
    let Requests::Out(out) =
        crate::project_rows::decision_loaded(&mut core, &env, &mut Queue::uncapped(), owner, None, true);
    assert!(out.is_empty());
}

#[test]
fn accepted_decisions_load_one_at_a_time_before_the_next_bounded_page() {
    let (mut core, env) = fixture();
    let owner = proposals(&mut core, &env);
    let out = page(
        &mut core,
        &env,
        owner,
        Box::new([
            proposal(1, PROJECT, tasks::PersonProposalState::Accepted { by: tasks::Party::Person(PARTY) }),
            proposal(2, PROJECT, tasks::PersonProposalState::Accepted { by: tasks::Party::Person(PARTY) }),
            proposal(3, 2, tasks::PersonProposalState::Withdrawn { at: Wall::EPOCH }),
        ]),
        true,
    );
    assert!(erased(&out).is_empty());
    assert_eq!(load(&out), (owner, 2));
    let out = loaded(&mut core, &env, owner, None, false);
    assert_eq!(erased(&out), decision_keys(2, None));
    assert_eq!(load(&out), (owner, 1));
    let out = loaded(&mut core, &env, owner, None, false);
    assert_eq!(erased(&out), decision_keys(1, None));
    assert_eq!(sweep(&out), (owner, Sweep::Proposals { after: Some(3) }));
    let _out = page(&mut core, &env, owner, Box::new([]), false);
    assert!(core.row_sweeps.is_empty());
    let Requests::Out(out) =
        crate::project_rows::swept(&mut core, &env, &mut Queue::uncapped(), owner, Box::new([]), true);
    assert!(out.is_empty());
}
