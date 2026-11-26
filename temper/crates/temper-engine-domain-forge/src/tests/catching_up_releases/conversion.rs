//! Start stories for previous branch rows (temper/docs/design/domain/forge/writes.md, 7).
#![expect(clippy::disallowed_types, reason = "ordinary Rust task fixtures")]
use super::*;
use crate::writes::{Actor, Operation, Seen};
use skein_forge_client as client;

const ATTEMPT: u64 = 4;
const PUSHED: Commit = [6; 32];
const WAITING: Commit = [8; 32];
const DESIGNER: Sender = Sender::Attempt { task: GOAL, attempt: ATTEMPT };

fn link(d: &mut Domain, live: &[u64]) -> Result<Queue<Request>, StartLink> {
    let mut world = jig_tasks_kit::World::new(71, jig_tasks_kit::LIMITS);
    let mut tasks = Vec::new();
    for task in live {
        tasks.push(jig_tasks_kit::task(*task, &[]));
    }
    assert_eq!(world.make(jig_tasks_kit::PERSON_PARTY, tasks), jig_tasks_kit::Reply::Made(live.into()));
    let mut store = jig_tasks_kit::domain(&world.env.limits, 71);
    for task in live {
        jig_domain_tasks::step(
            &mut store,
            &world.env,
            jig_domain_tasks::Event::Restore {
                record: jig_domain_tasks::Stored::Live(Box::new(world.record(*task).clone())),
            },
            &mut Queue::uncapped(),
        );
    }
    let mut out = Queue::uncapped();
    d.start_links(&store, &env(), &mut out)?;
    Ok(out)
}

fn live_branch(branch: &[u8], pushed: Commit) -> Stored {
    Stored::Client(client::Stored::Live(client::LiveRecord {
        watch: client::Watch { resource: resource(branch), participating: false },
        position: client::Position { at: Time::ZERO, comment: 0, review: 0, review_page: 0 },
        listed: Time::ZERO,
        comment_after: 0,
        comment_page: 1,
        comment_scanning: false,
        comment_repair: false,
        review_page: 1,
        comments: Box::new([]),
        reviews: Box::new([]),
        cached: Box::new(client::Cached { item: None, pull: None, tip: Some(fixtures::HEAD) }),
        pushed: Some(pushed),
        echoes: Box::new([]),
    }))
}

fn with_writer(rereading: bool) -> Hold {
    let mut held = hold(G, GOAL);
    held.design_writers = Box::new([DesignWriter { task: GOAL, attempt: ATTEMPT, rereading }]);
    held
}

fn persisted_watch() -> Stored {
    Stored::Client(client::Stored::Repository(client::RepositoryRecord {
        repository: REPO,
        mark: None,
        clock: Time::ZERO,
    }))
}

fn link_goal(held: Hold) -> Domain {
    let mut d = restored(G, Box::new([Stored::Hold(held), persisted_watch(), live_branch(G, fixtures::HEAD)]));
    link(&mut d, &[GOAL]).unwrap();
    d
}

#[test]
fn link_converts_expected_heads_and_preserves_holds_once() {
    let mut held = with_writer(false);
    held.beside = Some(HeadBeside { head: fixtures::HEAD, waiting: None });
    held.writer = Some(Writer::Entry(ENTRY));
    held.drift =
        Some(Drift { at: env().wall, change: DriftChange::Branch { expected: fixtures::HEAD, observed: Some(PUSH) } });
    let fallback = BranchHead { name: name(C), commit: fixtures::HEAD, owned: true };
    let mut d =
        restored(G, Box::new([Stored::Hold(held.clone()), Stored::BranchHead(fallback), live_branch(G, PUSHED)]));
    let first = link(&mut d, &[GOAL]).unwrap();
    let cleared = Hold { beside: None, ..held };
    let record = Writes {
        name: name(G),
        head: Some(PUSHED),
        next: 2,
        entries: Box::new([Written {
            order: 1,
            operation: Operation::Writer { before: Some(PUSHED) },
            sender: DESIGNER,
            state: WriteState::Sent,
        }]),
    };
    let base = Writes { name: name(C), head: Some(fixtures::HEAD), next: 1, entries: Box::new([]) };
    assert_eq!(d.writes.get(&name(G)), Some(&record));
    assert_eq!(d.writes.get(&name(C)), Some(&base));
    assert_eq!(d.hold(&name(G)), Some(&cleared));
    assert_eq!(first.len(), 3, "one decision saves both write records and the cleared hold");
    for row in [Stored::Hold(cleared.clone()), Stored::Writes(record.clone()), Stored::Writes(base.clone())] {
        let saved = Request::Save { read_afresh: None, release: None, record: row };
        let mut found = false;
        for request in &first {
            found |= *request == saved;
        }
        assert!(found, "missing {saved:?}");
    }
    let mut next = restored(G, Box::new([Stored::Hold(cleared), Stored::Writes(record), Stored::Writes(base)]));
    assert!(link(&mut next, &[GOAL]).unwrap().is_empty());
}

fn many_writers(count: usize) -> (Domain, Hold, Env<Limits>) {
    let mut held = hold(G, GOAL);
    let mut writers = Vec::new();
    for task in 1..=u64::try_from(count).unwrap() {
        writers.push(DesignWriter { task, attempt: ATTEMPT, rereading: false });
    }
    held.design_writers = writers.into_boxed_slice();
    let mut env = env();
    env.limits.holds = u32::try_from(count).unwrap();
    env.limits.client.resources = env.limits.holds;
    let mut d = domain_with_limits(&env.limits);
    d.restore_begin(env.wall);
    for record in [Stored::Repository(adopted()), Stored::Hold(held.clone())] {
        step(&mut d, &env, Event::Restore { record }, &mut Queue::uncapped());
    }
    (d, held, env)
}

#[test]
fn a_branch_can_convert_thirty_two_writers() {
    const WRITERS: usize = WRITES;
    let (d, _, env) = many_writers(WRITERS);
    let rows = crate::conversion::branches::prepare(&d, &env.limits).unwrap().into_boxed();
    assert_eq!(rows.first().unwrap().entries.len(), WRITERS);
}

#[test]
fn a_thirty_third_writer_refuses_conversion_without_any_save() {
    const WRITERS: usize = 33;
    let (mut d, held, env) = many_writers(WRITERS);
    let mut out = Queue::uncapped();
    let tasks = jig_tasks_kit::domain(&jig_tasks_kit::LIMITS, 71);
    assert_eq!(d.start_links(&tasks, &env, &mut out), Err(StartLink::BranchWrites(name(G), WRITERS)));
    assert!(out.is_empty());
    assert!(d.writes.is_empty());
    assert_eq!(d.hold(&name(G)), Some(&held));
}

fn branch_call(d: &mut Domain, branch: &[u8]) -> Token {
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Branch { branch: branch.into() }));
    call
}

fn no_drift(requests: &[Request]) {
    for request in requests {
        if let Request::Drift { .. } = request {
            panic!("conversion must not hold the branch: {request:?}");
        }
    }
}

fn contains(d: &mut Domain, tip: Commit, base: Commit, holds: bool) -> Box<[Request]> {
    let (call, op) = fixtures::next_call(d);
    assert_eq!(op, Op::Read(Read::Commits { before: tip, after: base, page: 1 }));
    let commits: Box<[_]> = if holds {
        Box::new([])
    } else {
        Box::new([skein_forge::api::CommitMessage { id: base, parents: Box::new([]), message: Box::new([]) }])
    };
    fixtures::answer(d, call, Ok(Answer::Commits { commits, more: false }))
}

#[test]
fn a_live_previous_design_push_advances_g_without_holding_it() {
    let mut d = link_goal(with_writer(false));
    begin(&mut d);
    no_drift(&finish_reads(&mut d, Some(fixtures::HEAD)));
    outputs(
        &mut d,
        Event::Hint {
            hint: skein_forge::api::Hint {
                repository: REPO,
                change: skein_forge::api::Change::Branch(G.into()),
                key: None,
            },
        },
    );
    let mut later = env();
    later.now = later.now.saturating_add(LIMITS.client.hinted);
    fire(&mut d, &later, &mut Queue::uncapped());
    let call = branch_call(&mut d, G);
    no_drift(&fixtures::answer(&mut d, call, Ok(Answer::Commit(PUSH))));
    no_drift(&contains(&mut d, PUSH, fixtures::HEAD, true));
    assert_eq!(d.branch_head(&name(G)), Some(PUSH));
    assert_eq!(d.hold(&name(G)).unwrap().drift, None);
    assert_eq!(d.nearest_write(&name(G)).unwrap().state, WriteState::Sent, "a live attempt can push again");
}

#[test]
fn a_converted_live_attempt_reads_back_at_its_answer_after_restart() {
    let d = link_goal(with_writer(false));
    let mut restarted = restored(
        G,
        Box::new([
            Stored::Hold(with_writer(false)),
            Stored::Writes(d.writes.get(&name(G)).unwrap().clone()),
            persisted_watch(),
            live_branch(G, fixtures::HEAD),
        ]),
    );
    link(&mut restarted, &[GOAL]).unwrap();
    begin(&mut restarted);
    finish_reads(&mut restarted, Some(fixtures::HEAD));
    outputs(&mut restarted, Event::Finished { task: GOAL, attempt: ATTEMPT, lost: false });
    let call = branch_call(&mut restarted, G);
    no_drift(&fixtures::answer(&mut restarted, call, Ok(Answer::Commit(PUSH))));
    no_drift(&contains(&mut restarted, PUSH, fixtures::HEAD, true));
    assert_eq!(restarted.branch_head(&name(G)), Some(PUSH));
    assert_eq!(restarted.nearest_write(&name(G)).unwrap().state, WriteState::Confirmed);
}

#[test]
fn a_waiting_head_is_ordered_once_and_remembers_the_older_head() {
    for (holds_waiting, newer, older) in [(true, fixtures::HEAD, WAITING), (false, WAITING, fixtures::HEAD)] {
        let mut held = hold(G, GOAL);
        held.beside = Some(HeadBeside { head: fixtures::HEAD, waiting: Some(WAITING) });
        let mut d = link_goal(held);
        let entry = d.nearest_write(&name(G)).unwrap();
        assert_eq!(entry.operation, Operation::Compare { after: WAITING });
        assert_eq!(entry.sender, Sender::Previous);
        assert_eq!(entry.state, WriteState::Uncertain);
        let mut out = Queue::uncapped();
        begin(&mut d);
        assert_eq!(d.observe(&name(G), Some(PUSH), Actor::Other, &mut out), Seen::Owed, "even drift waits");
        no_drift(&finish_reads(&mut d, Some(older)));
        no_drift(&contains(&mut d, fixtures::HEAD, WAITING, holds_waiting));
        assert_eq!(d.branch_head(&name(G)), Some(newer));
        let mut restarted = restored(G, Box::new([Stored::Writes(d.writes.get(&name(G)).unwrap().clone())]));
        assert_eq!(restarted.observe(&name(G), Some(older), Actor::Unknown, &mut out), Seen::Behind { order: 1 });
    }
}

#[test]
fn a_changes_lost_slot_is_read_before_its_branch_is_judged() {
    let mut held = hold(C, fixtures::CHANGE);
    held.writer = Some(Writer::Lost { task: GOAL, attempt: ATTEMPT });
    let mut d = restored(C, Box::new([Stored::Hold(held), Stored::Change(Box::new(fixtures::opening_row()))]));
    link(&mut d, &[GOAL, fixtures::CHANGE]).unwrap();
    begin(&mut d);
    let mut out = Queue::uncapped();
    assert_eq!(d.observe(&name(C), Some(fixtures::HEAD), Actor::Unknown, &mut out), Seen::Owed);
    finish_reads(&mut d, Some(fixtures::HEAD));
    let call = branch_call(&mut d, C);
    let asked = outputs(&mut d, fixtures::change_step());
    no_change_decision(&asked);
    assert_eq!(d.observe(&name(C), Some(fixtures::HEAD), Actor::Unknown, &mut out), Seen::Owed);
    fixtures::answer(&mut d, call, Ok(Answer::Commit(PUSH)));
    no_drift(&contains(&mut d, PUSH, fixtures::HEAD, true));
    assert_eq!(d.observe(&name(C), Some(PUSH), Actor::Unknown, &mut out), Seen::Current);
    assert_eq!(d.nearest_write(&name(C)).unwrap().state, WriteState::Confirmed);
}

fn no_change_decision(requests: &[Request]) {
    for request in requests {
        if let Request::ChangeDecision { decision, .. } = request {
            assert!(matches!(decision, change::Decision::Wait { .. }), "not a decision on the head: {request:?}");
        }
    }
}

#[test]
fn a_previous_design_claim_marked_rereading_reads_at_start() {
    let mut d = link_goal(with_writer(true));
    begin(&mut d);
    no_drift(&finish_reads(&mut d, Some(fixtures::HEAD)));
    let call = branch_call(&mut d, G);
    no_drift(&fixtures::answer(&mut d, call, Ok(Answer::Commit(PUSH))));
    no_drift(&contains(&mut d, PUSH, fixtures::HEAD, true));
    assert_eq!(d.branch_head(&name(G)), Some(PUSH));
    assert!(d.hold(&name(G)).unwrap().design_writers.is_empty());
    assert_eq!(d.nearest_write(&name(G)).unwrap().state, WriteState::Confirmed);
}
