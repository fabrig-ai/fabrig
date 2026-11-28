//! Restart through the engine ports, with a paged store and a journal release boundary
//! (`jig/docs/design/domain/engine/vocabulary.md`, 2.6).

use super::{Engine, In, Out, SEED, config, engine, environment, fresh_engine};
use crate::{
    Core, CoreRecord, Counters, Event, Key, Limits, Record, Request,
    documents::StoreCensus,
    effects::resources::Key as ResourceKey,
    engine::{Peer, Resume, application, store},
    start::{ApplicationWhy, Failure, Linked, Phase, RestoreRange},
    tests::{empty_declarations, finish_start, fixtures, make_task, people, tasks},
};
use jig_domain_contract::{Ask, Connector, Up, lifecycle, resources, settings};
use skein_lib::{Env, List, Map, Queue, Token, Wall};

const FIRST: Connector = Connector(1);
const SECOND: Connector = Connector(2);
const COMMITS: u64 = 18;
const STARTED_AT: Wall = Wall::from_nanos(12);

struct Store {
    rows: Map<Key, Record>,
    bound: u32,
}

impl Store {
    fn new(rows: Box<[Record]>, bound: u32) -> Self {
        let mut ordered = Map::with_capacity(u32::try_from(rows.len()).expect("fixture rows"));
        for row in rows {
            assert!(ordered.insert(row.key(), row).expect("fixture store room").is_none());
        }
        Self { rows: ordered, bound }
    }

    fn load(&self, load: Token, range: store::Range) -> In {
        let store::Range::Start { ranges, after } = range else { panic!("only start pages") };
        // The fake's cursor is the number of rows read in the full walk, including across ranges.
        let mut skip = match after {
            Some(after) => {
                assert_eq!(after.after.as_ref(), b"store cursor");
                after.interval
            }
            None => 0,
        };
        let start = skip;
        let mut items = List::with_capacity(self.bound);
        let mut next = None;
        'walk: for range in ranges {
            for (key, row) in &self.rows {
                if !range.admits(key) {
                    continue;
                }
                if let Some(before) = skip.checked_sub(1) {
                    skip = before;
                    continue;
                }
                if items.len() == self.bound {
                    next = Some(cursor(start.checked_add(items.len()).expect("fixture row count")));
                    break 'walk;
                }
                items.push(store::Item::Record(row.clone())).expect("room for one page");
            }
        }
        In::Store(store::In::Loaded {
            load,
            result: store::Answer::Rows(store::Rows::Start { items: items.into_boxed(), next }),
        })
    }
}

struct World {
    engine: Engine,
    env: Env<Limits>,
    out: Queue<Out>,
    store: Store,
}

impl World {
    fn new(rows: Box<[Record]>, bound: u32) -> Self {
        let mut env = environment();
        env.wall = STARTED_AT;
        Self { engine: fresh_engine(&env.limits), env, out: Queue::uncapped(), store: Store::new(rows, bound) }
    }

    fn step(&mut self, input: In) {
        assert!(self.out.is_empty(), "inspect each decision before giving its answer: {:?}", self.out);
        engine::step(&mut self.engine, &self.env, input, &mut self.out);
    }

    fn open(&mut self) {
        self.step(In::Store(store::In::Opened { pending: false }));
    }

    fn phase(&mut self, expected: Phase) {
        for _ in 0..self.out.len() {
            let Some(Out::Core(Request::Ask {
                ask: crate::Ask::Workers(_) | crate::Ask::Providers(_) | crate::Ask::Catalogue { .. },
                ..
            })) = self.out.iter().next()
            else {
                break;
            };
            drop(self.out.pop());
        }
        let output = self.out.pop();
        let Some(Out::Now(Peer::Application(application::Out::Start { phase: Some(phase) }))) = output else {
            panic!("expected {expected:?}, got {output:?}");
        };
        assert_eq!(phase, expected);
    }

    fn load(&mut self) -> (Token, store::Range) {
        let Some(Out::Held(Peer::Store(store::Out::Load { load, range }))) = self.out.pop() else {
            panic!("expected a held store load: {:?}", self.out);
        };
        (load, range)
    }

    fn page(&mut self) {
        let (load, range) = self.load();
        self.step(self.store.load(load, range));
    }

    fn restore(&mut self, expected: Connector) {
        self.phase(Phase::RestoreConnector { connector: expected.0 });
        let Some(Out::Ask(number, Ask::Lifecycle(lifecycle::Ask::Restore))) = self.out.pop() else {
            panic!("unexpected start output")
        };
        assert_eq!(number, expected);
        assert!(self.out.is_empty());
    }

    fn continuation(&mut self) -> Resume {
        let mut resume = None;
        while let Some(output) = self.out.pop() {
            match output {
                Out::Write(_) | Out::Core(Request::Ask { .. }) => {}
                Out::Held(Peer::Resume(value)) => {
                    assert!(resume.replace(value).is_none());
                    assert!(self.out.is_empty(), "continuation closes the decision");
                }
                other @ (Out::Core(
                    Request::Held(_) | Request::Now(_) | Request::ProjectMade { .. } | Request::Write(_),
                )
                | Out::Now(_)
                | Out::Ask(..)
                | Out::Held(Peer::Application(_) | Peer::Connector(..) | Peer::Store(_))) => {
                    panic!("not a start commit: {other:?}");
                }
            }
        }
        resume.expect("the commit holds the start continuation")
    }

    fn release(&mut self) {
        let resume = self.continuation();
        self.step(In::Resume(resume));
    }

    /// Arrive at the first connector's restore, from an empty store.
    fn reach_restore(&mut self) {
        self.open();
        self.phase(Phase::LoadCore);
        self.page();
        self.release(); // First-start writes before the inbox.
        self.phase(Phase::LoadInbox);
        self.page();
        self.restore(FIRST);
    }

    fn reach_sweep(&mut self) {
        self.step(restored(FIRST));
        self.restore(SECOND);
        self.step(restored(SECOND));
        self.phase(Phase::Link);
        let mut held = false;
        for output in &self.out {
            if let Out::Held(Peer::Resume(_)) = output {
                held = true;
            }
        }
        if held {
            self.release();
        }
        self.phase(Phase::Sweep);
    }

    fn switch(&mut self) {
        self.phase(Phase::Switch);
        let Some(Out::Held(Peer::Store(store::Out::Switch))) = self.out.pop() else {
            panic!("switch is held even with no pending generation")
        };
        assert!(!self.engine.core().start_opened());
        self.step(In::Store(store::In::Switched));
        self.phase(Phase::Wires);
        let Some(Out::Write(store::Write::Wires)) = self.out.pop() else { panic!("Wires commits its mark") };
        assert!(!self.engine.core().start_opened());
    }

    fn opened(&mut self) {
        self.phase(Phase::AdoptRuns);
        while let Some(Out::Core(_) | Out::Write(_)) = self.out.iter().next() {
            drop(self.out.pop());
        }
        self.phase(Phase::Open);
        for expected in [FIRST, SECOND] {
            let Some(Out::Ask(number, Ask::Lifecycle(lifecycle::Ask::CatchUp))) = self.out.pop() else {
                panic!("unexpected start output")
            };
            assert_eq!(number, expected);
        }
        let Some(Out::Now(Peer::Application(application::Out::Start { phase: None }))) = self.out.pop() else {
            panic!("unexpected start output")
        };
        assert!(self.out.is_empty() && self.engine.core().start_opened());
    }
}

fn restored(connector: Connector) -> In {
    In::Connector(connector, Up::Lifecycle(lifecycle::Up::Done { stage: lifecycle::RestartStage::Restored }))
}

fn cursor(rows: u32) -> store::Cursor {
    store::Cursor { interval: rows, after: b"store cursor".as_slice().into() }
}

fn header() -> Record {
    let mut header = Counters::bootstrap([3; 16]).deployment;
    header.commits = COMMITS;
    header.projects = 1;
    header.tasks = 1;
    header.people = 1;
    header.sign_ins = 11;
    Record::Core(CoreRecord::Deployment(header))
}

fn live_rows() -> Box<[Record]> {
    let env = environment();
    let fixture = config(SEED);
    let declarations = empty_declarations(&fixture);
    let mut core = Core::new(fixture, &declarations, &env.limits).expect("fixture configuration");
    let mut first = finish_start(&mut core, &env);
    let mut policy = None;
    for _ in 0..first.len() {
        if let Some(Request::Write(store::Write::Save(record @ Record::People(people::Stored::Policy { .. })))) =
            first.pop()
        {
            policy = Some(record);
        }
    }
    make_task(&mut core, &env.limits, 1);
    let task = core.tasks.task(1).expect("saved task").clone();
    let ledger = *core.tasks.funding(task.funder).expect("saved funding");
    Box::new([
        Record::Tasks(tasks::Stored::Live(Box::new(task))),
        policy.expect("first start saves the seeded project"),
        Record::People(people::Stored::Person { number: 1, identity: fixtures::newcomer() }),
        header(),
        Record::Tasks(tasks::Stored::Ledger(ledger)),
    ])
}

fn census(commit: u64) -> In {
    In::Store(store::In::Census(Box::new(StoreCensus { commit, since: 1, keyspaces: List::with_capacity(0) })))
}

#[test]
fn empty_store_opens_through_every_phase() {
    let mut world = World::new(Box::new([]), 2);
    world.reach_restore();
    world.reach_sweep();
    world.page();
    world.switch();
    let resume = world.continuation();
    world.step(In::Resume(Resume(Token::new(resume.0.raw().checked_add(1).expect("next token")))));
    assert!(world.out.is_empty() && !world.engine.core().start_opened(), "an unknown continuation opens nothing");
    world.step(In::Resume(resume));
    world.opened();
    world.step(In::Resume(resume));
    assert!(world.out.is_empty(), "a released continuation is not taken twice");
}

#[test]
fn header_reports_numbering_in_its_page_decision() {
    let mut world = World::new(live_rows(), 2);
    world.open();
    world.phase(Phase::LoadCore);
    world.page();
    let Some(Out::Now(Peer::Application(application::Out::Numbered { commits: COMMITS }))) = world.out.pop() else {
        panic!("unexpected start output")
    };
    let Some(Out::Held(Peer::Store(store::Out::Load { range: store::Range::Start { after: Some(_), .. }, .. }))) =
        world.out.pop()
    else {
        panic!("unexpected start output")
    };
    assert!(world.out.is_empty());
}

#[test]
fn live_pages_echo_the_cursor_and_restore_the_project_and_task() {
    let mut world = World::new(live_rows(), 2);
    world.open();
    world.phase(Phase::LoadCore);
    let mut before = None;
    for after in [None, Some(cursor(2)), Some(cursor(4))] {
        let (load, range) = world.load();
        assert_eq!(range, store::Range::Start { ranges: Box::new(RestoreRange::LIVE), after });
        assert_ne!(before, Some(load));
        let input = world.store.load(load, range);
        world.step(input);
        if before.is_none() {
            let Some(Out::Now(Peer::Application(application::Out::Numbered { .. }))) = world.out.pop() else {
                panic!("unexpected start output")
            };
        }
        before = Some(load);
    }
    world.phase(Phase::LoadInbox);
    world.page();
    world.restore(FIRST);
    world.reach_sweep();
    world.page();
    world.switch();
    world.release();
    world.opened();
    assert!(world.engine.core().projects.to_boxed().contains(&1));
    assert_eq!(world.engine.core().tasks.task(1).expect("restored task").project, 1);
}

#[test]
fn sweep_waits_for_each_erase_and_keeps_the_open_wall_time() {
    let mut rows = List::with_capacity(8);
    for row in live_rows() {
        rows.push(row).expect("room for live rows");
    }
    for number in [9, 10, 11] {
        rows.push(Record::People(people::Stored::SignIn { number, person: 1, expires: Wall::from_nanos(number) }))
            .expect("room for expired sign-ins");
    }
    let mut world = World::new(rows.into_boxed(), 2);
    world.open();
    world.phase(Phase::LoadCore);
    world.page();
    let Some(Out::Now(Peer::Application(application::Out::Numbered { .. }))) = world.out.pop() else {
        panic!("unexpected start output")
    };
    for _ in 0_u8..3 {
        world.page();
    }
    world.phase(Phase::LoadInbox);
    world.page();
    world.restore(FIRST);
    world.reach_sweep();
    world.env.wall = Wall::EPOCH;
    // Consume each page until the first expiry. Each erase is the only write of its decision,
    // and comes before its held release, never alongside a further load or phase.
    let mut pages = 0_u8;
    for number in [9, 10, 11] {
        while let Some(Out::Held(Peer::Store(store::Out::Load { .. }))) = world.out.iter().next() {
            world.page();
            pages = pages.checked_add(1).expect("fixture pages");
        }
        let Some(Out::Write(store::Write::Erase(Key::People(people::Key::SignIn(found))))) = world.out.pop() else {
            panic!("unexpected start output")
        };
        assert_eq!(found, number);
        let Some(Out::Held(Peer::Resume(resume))) = world.out.pop() else { panic!("each erase holds its release") };
        assert!(world.out.is_empty(), "nothing follows the erase until release");
        world.step(In::Resume(resume));
    }
    assert_eq!(pages, 2, "three expiries span two pages");
    let (load, range) = world.load();
    assert_eq!(range, store::Range::Start { ranges: Box::new([RestoreRange::People]), after: Some(cursor(4)) });
    world.step(world.store.load(load, range));
    world.switch();
    world.release();
    world.opened();
}

#[test]
fn only_the_current_connector_finishes_restore_and_roles_decide_nothing() {
    let mut world = World::new(Box::new([]), 2);
    world.reach_restore();
    world.step(restored(SECOND));
    assert!(world.out.is_empty(), "the second connector has not been asked");
    let name = resources::Name { connector: FIRST.0, path: Box::new([b"resource".as_slice().into()]) };
    let report = || {
        Up::Resources(resources::Up::Resource {
            project: 1,
            name: name.clone(),
            role: resources::ResourceRole::Context,
            hold: resources::HoldKind::Exclusive { wait: true },
        })
    };
    let key = ResourceKey { project: 1, name: name.clone() };
    world.step(In::Connector(SECOND, report()));
    assert!(world.engine.core().resource_roles.get(&key).is_none());
    world.step(In::Connector(FIRST, report()));
    assert!(world.out.is_empty(), "restored roles do not decide the reported hold");
    assert_eq!(world.engine.core().resource_roles.get(&key), Some(&resources::ResourceRole::Context));
    world.step(restored(FIRST));
    world.restore(SECOND);
    world.step(restored(FIRST));
    assert!(world.out.is_empty(), "a stale report cannot complete the next connector");
}

#[test]
fn unfit_refuses_once_in_the_owning_connector_phase() {
    let mut world = World::new(Box::new([]), 2);
    world.reach_restore();
    let unfit =
        |connector| In::Connector(connector, Up::Settings(settings::Up::Unfit { kind: 3, scope: 5, number: 7 }));
    world.step(unfit(SECOND));
    assert!(world.out.is_empty());
    world.step(unfit(FIRST));
    let Some(Out::Now(Peer::Application(application::Out::Refused {
        phase: Phase::RestoreConnector { connector: 1 },
        failure: Failure::Application { kind: 3, scope: 5, number: 7, why: ApplicationWhy::Shape },
    }))) = world.out.pop()
    else {
        panic!("unexpected start output")
    };
    world.step(unfit(FIRST));
    world.step(restored(FIRST));
    assert!(world.out.is_empty(), "a refused start neither speaks again nor advances");
}

#[test]
fn missing_owner_refuses_at_link_once() {
    let mut world = World::new(Box::new([]), 2);
    world.open();
    world.phase(Phase::LoadCore);
    world.page();
    world.release();
    world.phase(Phase::LoadInbox);
    let (load, _) = world.load();
    let record = Key::People(people::Key::EndedRead { task: 1, party: 1 });
    let target = Key::Tasks(tasks::Key::Ended(1));
    world.step(In::Store(store::In::Loaded {
        load,
        result: store::Answer::Rows(store::Rows::Start {
            items: Box::new([store::Item::Missing { record: record.clone(), target: target.clone() }]),
            next: None,
        }),
    }));
    world.restore(FIRST);
    world.step(restored(FIRST));
    world.restore(SECOND);
    world.step(restored(SECOND));
    world.phase(Phase::Link);
    let Some(Out::Now(Peer::Application(application::Out::Refused {
        phase: Phase::Link,
        failure: Failure::Link { record: found, linked: Linked::Record(linked) },
    }))) = world.out.pop()
    else {
        panic!("unexpected start output")
    };
    assert_eq!(found, record);
    assert_eq!(linked, target);
    world.step(restored(SECOND));
    assert!(world.out.is_empty());
}

#[test]
fn failed_or_wrong_start_page_keeps_its_phase() {
    for result in [
        store::Answer::Failed(store::Failure::Failed),
        store::Answer::Rows(store::Rows::Records { records: Box::new([]), more: false }),
    ] {
        let mut world = World::new(Box::new([]), 2);
        world.open();
        world.phase(Phase::LoadCore);
        let (load, range) = world.load();
        world.step(In::Store(store::In::Loaded { load, result }));
        assert!(world.out.is_empty(), "a failed page does not ask or refuse a new phase");
        world.step(world.store.load(load, range));
        assert!(world.out.is_empty());
        assert_eq!(world.engine.core().start_phase(), Some(Phase::LoadCore));
    }
}

#[test]
fn store_reports_wait_for_adoption_and_then_apply_immediately() {
    let mut world = World::new(Box::new([]), 2);
    world.open();
    world.phase(Phase::LoadCore);
    let (load, range) = world.load();
    for seen in [5, 6] {
        world.step(In::Store(store::In::Fill { bytes: seen }));
        world.step(census(seen));
    }
    assert_eq!(world.engine.core().fill, 0);
    assert!(world.engine.core().census.is_none());
    world.step(world.store.load(load, range));
    world.release();
    world.phase(Phase::LoadInbox);
    world.page();
    world.restore(FIRST);
    world.reach_sweep();
    world.page();
    world.switch();
    assert!(world.engine.core().census.is_none(), "reports stay buffered through the Wires write");
    world.release();
    world.opened();
    assert_eq!(world.engine.core().census.as_ref().expect("latest census applied").commit, 6);
    assert_eq!(world.engine.core().fill, 6);
    world.step(census(7));
    assert_eq!(world.engine.core().census.as_ref().expect("latest census applied").commit, 7);
    world.step(In::Store(store::In::Fill { bytes: 7 }));
    assert_eq!(world.engine.core().fill, 7);
}

#[test]
fn stopping_drops_an_outstanding_start_page() {
    let mut world = World::new(live_rows(), 2);
    world.open();
    world.phase(Phase::LoadCore);
    let (load, range) = world.load();
    world.step(In::Core(Event::Stopping));
    let Some(Out::Core(Request::Held(held))) = world.out.pop() else { panic!("unexpected start output") };
    let crate::Held::Drained = *held else { panic!("stopping drains") };
    world.step(world.store.load(load, range));
    assert!(world.out.is_empty(), "a page delivered after stopping cannot number or load again");
    assert_eq!(world.engine.core().start_phase(), Some(Phase::LoadCore));
}

#[test]
fn every_store_input_and_resume_bypass_the_watermark() {
    for input in [
        In::Store(store::In::Opened { pending: false }),
        In::Store(store::In::Loaded { load: Token::new(1), result: store::Answer::Failed(store::Failure::Busy) }),
        In::Store(store::In::Switched),
        In::Store(store::In::Fill { bytes: 0 }),
        census(1),
        In::Resume(Resume(Token::new(1))),
    ] {
        assert!(!input.new_work(), "{input:?} must be taken when new work waits");
    }
}
