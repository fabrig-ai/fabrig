//! Start's written conversion from the previous store layouts (temper/docs/design/domain/landing-members.md, 4.2).
#![expect(clippy::disallowed_types, reason = "ordinary Rust store fixtures")]
use super::{dev_layouts::Dev, fixtures};
use crate::{keys, records};
use alloc::collections::BTreeMap;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, Time, Wall};
use temper_engine_domain_forge as forge;
use temper_engine_domain_forge_change::{Ask, Handed, Judgement, State};
use temper_engine_domain_forge_landing as landing;

const STALE_CHANGE: u64 = 5;
const STALE_LANDING: u64 = 20;
const BOTH_CHANGE: u64 = 6;
const BOTH_LANDING: u64 = 21;
const SINGLE_CHANGE: u64 = 7;
const SINGLE_LANDING: u64 = 22;
const FAILED_CHANGE: u64 = 8;
const FAILED_LANDING: u64 = 23;
const LONE_CHANGE: u64 = 9;
const NAMED_LANDING: u64 = 25;
const INDEX_LANDING: u64 = 26;
const ROUND: u16 = 3;
const BASE: [u8; 32] = [1; 32];
const HEAD: [u8; 32] = [2; 32];
const PREFIX: [u8; 32] = [3; 32];
const OTHER_HEAD: [u8; 32] = [4; 32];
const READY_SINCE: Wall = Wall::from_nanos(7);
// The overflow story restores seventeen historical landing rows for one change.
const QUEUE_BOUND: u32 = 16;
const LANDING_ROWS: u32 = QUEUE_BOUND + 1;
const ENV: Env<forge::Limits> =
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: forge::Limits { landings: LANDING_ROWS, ..fixtures::LIMITS } };

fn forge() -> forge::Domain {
    let configured = forge::Configured {
        admin_connect: false,
        bot: b"bot".as_slice().into(),
        owned: None,
        git: b"https://forge.example".as_slice().into(),
        web: None,
        read_only: false,
        number: fixtures::REPO.forge,
        kind: forge::Kind::Forgejo,
        mode: forge::Mode::Fake,
        asked: false,
    };
    let mut domain = forge::Domain::new(
        &ENV.limits,
        91,
        forge::Config { namespace: Box::new([7; 16]), writers: Box::new([]), push: None },
        &[],
        &[configured],
        false,
    )
    .unwrap();
    domain.restore_begin(ENV.wall);
    domain
}

fn task_store(live: &[u64]) -> tasks::Domain {
    let mut tasks = jig_tasks_kit::domain(&jig_tasks_kit::LIMITS, 71);
    if !live.is_empty() {
        let mut world = jig_tasks_kit::World::new(71, jig_tasks_kit::LIMITS);
        let mut created = Vec::new();
        for number in live {
            created.push(jig_tasks_kit::task(*number, &[]));
        }
        assert_eq!(world.make(jig_tasks_kit::PERSON_PARTY, created), jig_tasks_kit::Reply::Made(Vec::from(live)));
        for task in live {
            tasks::step(
                &mut tasks,
                &world.env,
                tasks::Event::Restore { record: tasks::Stored::Live(Box::new(world.record(*task).clone())) },
                &mut Queue::uncapped(),
            );
        }
    }
    tasks
}

fn restore(domain: &mut forge::Domain, record: forge::Stored) {
    forge::step(domain, &ENV, forge::Event::Restore { record }, &mut Queue::uncapped());
}

struct Store {
    dev: Dev,
    handover: Option<Dev>,
    domain: forge::Domain,
    restored: BTreeMap<forge::Key, forge::Stored>,
}
impl Store {
    fn new() -> Self {
        let mut store = Self { dev: Dev::new(), handover: None, domain: forge(), restored: BTreeMap::new() };
        store.restore(fixtures::adopted(forge::Protection::Absent), "repository");
        store
    }
    fn restore(&mut self, record: forge::Stored, variant: &str) {
        let row = self.dev.row(&record, variant);
        let record = self.dev.read(&row, variant);
        self.restore_record(record);
    }
    fn restore_record(&mut self, record: forge::Stored) {
        self.restored.insert(forge::stored_key(&record), record.clone());
        restore(&mut self.domain, record);
    }
    fn change(&mut self, task: u64, state: State, named: Option<u64>, ask: Option<Ask>) {
        let mut change = fixtures::change_row(state);
        change.task = task;
        change.change.task = task;
        change.change.last_head = Some(HEAD);
        let handed_over = ask.is_some();
        change.previous =
            if named.is_some() || handed_over { Some(forge::PreviousLink { landing: named, ask }) } else { None };
        let (cold, hot) = change.split();
        self.restore(forge::Stored::ChangeCold(Box::new(cold)), "change_cold");
        let hot = forge::Stored::ChangeHot(Box::new(hot));
        if handed_over {
            // Dev lacks Change.landing_ask. Use the store-v25 shape with the handed-over
            // slot kept and without the new queue/count on ChangeHot.
            if self.handover.is_none() {
                let layout = fixtures::layouts::BINARY.replace(" asks: list 16 LandingHanded = [] handed: u32 = 0", "");
                assert_ne!(layout, fixtures::layouts::BINARY);
                self.handover = Some(Dev::with_layout(&layout));
            }
            let old = self.handover.as_mut().expect("layout with landing_ask");
            let row = old.row(&hot, "change_hot");
            let record = old.read(&row, "change_hot");
            self.restore_record(record);
        } else {
            self.restore(hot, "change_hot");
        }
    }
    fn landing(&mut self, row: forge::LandingRow) {
        self.restore(forge::Stored::Landing(row), "landing");
    }
    fn link(&mut self, live: &[u64]) -> Vec<forge::Request> {
        link(&mut self.domain, live)
    }
    fn assert_queue(&self, task: u64, handed: u32, asks: &[Handed]) {
        let row = self.domain.change(task).expect("converted change");
        assert_eq!(row.asks.as_ref(), asks);
        assert_eq!(row.handed, handed);
    }
}

fn link(domain: &mut forge::Domain, live: &[u64]) -> Vec<forge::Request> {
    let mut out = Queue::uncapped();
    domain.start_links(&task_store(live), &ENV, &mut out).expect("link previous store");
    let mut requests = Vec::new();
    while let Some(request) = out.pop() {
        requests.push(request);
    }
    requests
}

fn ready() -> State {
    State::Ready { head: HEAD, ready_since: READY_SINCE }
}
fn judgement(landing: u64, round: u16) -> Judgement {
    Judgement {
        landing,
        round,
        pull: 1,
        branch: b"temper/round".as_slice().into(),
        base: b"main".as_slice().into(),
        prefix: HEAD,
    }
}
fn row(task: u64, change: u64, phase: landing::Phase, ask: Option<Ask>) -> forge::LandingRow {
    let forge::Stored::Landing(mut row) = super::test_ids::landing() else { panic!("landing fixture") };
    row.landing.task = task;
    row.landing.preparation = ROUND;
    row.landing.phase = phase;
    row.landing.members = Box::new([landing::Member {
        round: Some(ROUND),
        change,
        repository: landing::Repository { forge: fixtures::REPO.forge, repository: fixtures::REPO.repository },
        pull: 16,
        head: Some(HEAD),
        base: Some(BASE),
        candidate: None,
        gates: Box::new([]),
        asked: None,
        landed: None,
        left: None,
        unit: 0,
        branch: b"temper/change".as_slice().into(),
        combined: None,
        recorded: false,
        told: false,
        judging: false,
        leave_told: false,
    }]);
    row.previous_asks = Box::new([ask]);
    row
}
fn cancelled(task: u64, change: u64, ask: Ask) -> forge::LandingRow {
    row(task, change, landing::Phase::Cancelled, Some(ask))
}
fn single(task: u64, change: u64, prefix: [u8; 32], ask: Option<Ask>) -> forge::LandingRow {
    let mut row = row(task, change, landing::Phase::Landed, ask);
    row.landing.round.branch = Box::new([]);
    row.landing.members[0].landed = Some(prefix);
    row
}
fn failed(task: u64, change: u64) -> forge::LandingRow {
    let mut row = row(task, change, landing::Phase::Proving, None);
    let member = &mut row.landing.members[0];
    member.candidate = Some(HEAD);
    member.gates = Box::new([landing::GateProof {
        proof: landing::Proof::Failed { key: [5; 32] },
        posted: landing::Posted::Verdict,
        faults: 0,
    }]);
    row
}
fn ask(number: u32, landing: u64, body: Ask) -> Handed {
    Handed { number, landing, round: ROUND, ask: body }
}
fn resolve() -> Ask {
    Ask::Resolve { base: BASE }
}
fn repair(gate: u16, candidate: [u8; 32]) -> Ask {
    Ask::Repair { gate, candidate, below: None }
}
fn warnings(output: &[forge::Request]) -> Vec<&forge::Request> {
    let mut warnings = Vec::new();
    for request in output {
        if let forge::Request::LinkDisagreed { .. } | forge::Request::ConversionAskDropped { .. } = request {
            warnings.push(request);
        }
    }
    warnings
}

#[test]
fn a_cancelled_landings_resolve_survives_its_stale_link() {
    let mut store = Store::new();
    store.change(STALE_CHANGE, ready(), Some(STALE_LANDING), None);
    store.landing(cancelled(STALE_LANDING, STALE_CHANGE, resolve()));
    let output = store.link(&[]);
    store.assert_queue(STALE_CHANGE, 1, &[ask(1, STALE_LANDING, resolve())]);
    assert_eq!(store.domain.landing_of(STALE_CHANGE), None);
    assert_eq!(
        warnings(&output),
        [&forge::Request::LinkDisagreed { change: STALE_CHANGE, named: STALE_LANDING, landing: None }]
    );
}

#[test]
fn a_row_repair_wins_over_the_handed_over_repair() {
    let mut store = Store::new();
    store.change(BOTH_CHANGE, ready(), None, Some(repair(0, OTHER_HEAD)));
    store.landing(row(STALE_LANDING, BOTH_CHANGE, landing::Phase::Cancelled, None));
    store.landing(cancelled(BOTH_LANDING, BOTH_CHANGE, repair(1, HEAD)));
    store.link(&[]);
    store.assert_queue(BOTH_CHANGE, 1, &[ask(1, BOTH_LANDING, repair(1, HEAD))]);
}

#[test]
fn a_single_landings_recorded_prefix_precedes_the_resolve() {
    let mut store = Store::new();
    store.change(SINGLE_CHANGE, ready(), None, Some(resolve()));
    store.landing(single(SINGLE_LANDING, SINGLE_CHANGE, PREFIX, None));
    store.link(&[]);
    store.assert_queue(
        SINGLE_CHANGE,
        2,
        &[ask(1, SINGLE_LANDING, Ask::Landed { prefix: PREFIX }), ask(2, SINGLE_LANDING, resolve())],
    );
}

#[test]
fn an_empty_slot_recovers_a_failed_proofs_repair() {
    let mut store = Store::new();
    store.change(FAILED_CHANGE, ready(), None, None);
    store.landing(failed(FAILED_LANDING, FAILED_CHANGE));
    store.link(&[FAILED_LANDING]);
    store.assert_queue(FAILED_CHANGE, 1, &[ask(1, FAILED_LANDING, repair(0, HEAD))]);
}

#[test]
fn a_handed_over_judgement_keeps_its_own_target() {
    const TARGET_ROUND: u16 = 2;
    let mut store = Store::new();
    let judge = Ask::Judge(judgement(NAMED_LANDING, TARGET_ROUND));
    store.change(BOTH_CHANGE, ready(), None, Some(judge.clone()));
    store.landing(row(INDEX_LANDING, BOTH_CHANGE, landing::Phase::Cancelled, None));
    store.link(&[]);
    store.assert_queue(
        BOTH_CHANGE,
        1,
        &[Handed { number: 1, landing: NAMED_LANDING, round: TARGET_ROUND, ask: judge }],
    );
}

#[test]
fn a_handed_over_resolve_without_a_row_is_unattributed() {
    let mut store = Store::new();
    store.change(LONE_CHANGE, ready(), None, Some(resolve()));
    store.link(&[]);
    store.assert_queue(LONE_CHANGE, 1, &[Handed { number: 1, landing: 0, round: 0, ask: resolve() }]);
}

#[test]
fn only_a_link_disagreeing_with_the_live_index_warns() {
    for (named, expected) in [
        (INDEX_LANDING, None),
        (
            NAMED_LANDING,
            Some(forge::Request::LinkDisagreed {
                change: LONE_CHANGE,
                named: NAMED_LANDING,
                landing: Some(INDEX_LANDING),
            }),
        ),
    ] {
        let mut store = Store::new();
        store.change(LONE_CHANGE, ready(), Some(named), None);
        store.landing(row(INDEX_LANDING, LONE_CHANGE, landing::Phase::Waiting, None));
        let output = store.link(&[INDEX_LANDING]);
        assert_eq!(warnings(&output), Vec::from_iter(&expected));
    }
}

#[test]
fn landed_asks_precede_ordinary_asks_in_landing_order() {
    let mut store = Store::new();
    store.change(SINGLE_CHANGE, ready(), None, Some(Ask::Landed { prefix: OTHER_HEAD }));
    store.landing(single(SINGLE_LANDING, SINGLE_CHANGE, PREFIX, Some(resolve())));
    store.landing(single(INDEX_LANDING, SINGLE_CHANGE, HEAD, Some(repair(1, HEAD))));
    store.link(&[]);
    store.assert_queue(
        SINGLE_CHANGE,
        4,
        &[
            ask(1, SINGLE_LANDING, Ask::Landed { prefix: PREFIX }),
            ask(2, INDEX_LANDING, Ask::Landed { prefix: HEAD }),
            ask(3, SINGLE_LANDING, resolve()),
            ask(4, INDEX_LANDING, repair(1, HEAD)),
        ],
    );
}

#[test]
fn overflow_drops_and_warns_for_oldest_non_landed_asks() {
    const FIRST: u64 = 30;
    const AFTER_DROPPED: u64 = FIRST + 2;
    let last = FIRST + u64::from(QUEUE_BOUND);
    let mut store = Store::new();
    store.change(SINGLE_CHANGE, ready(), None, None);
    store.landing(single(FIRST, SINGLE_CHANGE, PREFIX, Some(resolve())));
    for task in FIRST + 1..=last {
        store.landing(cancelled(task, SINGLE_CHANGE, resolve()));
    }
    let output = store.link(&[]);
    let mut kept = Vec::from([ask(1, FIRST, Ask::Landed { prefix: PREFIX })]);
    for (number, task) in (2..=QUEUE_BOUND).zip(AFTER_DROPPED..=last) {
        kept.push(ask(number, task, resolve()));
    }
    store.assert_queue(SINGLE_CHANGE, QUEUE_BOUND, &kept);
    assert_eq!(
        warnings(&output),
        [
            &forge::Request::ConversionAskDropped { change: SINGLE_CHANGE, landing: FIRST, kind: "resolve" },
            &forge::Request::ConversionAskDropped { change: SINGLE_CHANGE, landing: FIRST + 1, kind: "resolve" },
        ]
    );
}

#[test]
fn the_written_rows_do_not_convert_again() {
    let mut store = Store::new();
    store.change(SINGLE_CHANGE, ready(), Some(SINGLE_LANDING), Some(resolve()));
    store.change(FAILED_CHANGE, ready(), Some(FAILED_LANDING), None);
    store.landing(single(SINGLE_LANDING, SINGLE_CHANGE, PREFIX, None));
    store.landing(failed(FAILED_LANDING, FAILED_CHANGE));
    for request in store.link(&[FAILED_LANDING]) {
        if let forge::Request::Save { record, .. } = request {
            if let forge::Stored::ChangeHot(row) = &record {
                assert_eq!(row.previous, None);
            }
            if let forge::Stored::Landing(row) = &record {
                assert!(row.previous_asks.is_empty());
            }
            store.restored.insert(forge::stored_key(&record), record);
        }
    }
    let mut again = forge();
    let bounds = crate::bounds(&ENV.limits, ENV.limits.client.op_bytes).unwrap();
    for (key, record) in store.restored {
        let key = keys::encode(&key, u32::MAX).unwrap();
        restore(&mut again, records::row(&key, &records::encode(&record), &bounds).unwrap());
    }
    assert!(link(&mut again, &[FAILED_LANDING]).is_empty(), "no writes or warnings on the second start");
    for task in [SINGLE_CHANGE, FAILED_CHANGE] {
        let row = again.change(task).expect("restored conversion");
        let first = store.domain.change(task).expect("first start's change");
        assert_eq!((&row.asks, row.handed), (&first.asks, first.handed));
    }
}

#[test]
fn unmade_cleanup_keeps_the_repair_converted_in_link() {
    let mut store = Store::new();
    store.change(
        FAILED_CHANGE,
        State::Judging {
            head: HEAD,
            ready_since: READY_SINCE,
            target: judgement(FAILED_LANDING, ROUND),
            asked: Box::new([]),
        },
        Some(FAILED_LANDING),
        None,
    );
    store.landing(failed(FAILED_LANDING, FAILED_CHANGE));
    let output = store.link(&[]);
    assert!(warnings(&output).is_empty(), "the link agrees before the unmade pass");
    store.assert_queue(FAILED_CHANGE, 1, &[ask(1, FAILED_LANDING, repair(0, HEAD))]);
    let mut converted = None;
    let mut erased = None;
    let mut saved_ready = false;
    for (index, request) in output.iter().enumerate() {
        if let forge::Request::Save { record: forge::Stored::ChangeHot(hot), .. } = request
            && hot.task == FAILED_CHANGE
            && hot.previous.is_none()
            && hot.asks.as_ref() == [ask(1, FAILED_LANDING, repair(0, HEAD))]
        {
            if let State::Judging { .. } = hot.change.state {
                converted = Some(index);
            }
            saved_ready |= hot.change.state == ready();
        }
        if *request == (forge::Request::Erase { key: forge::Key::Landing(FAILED_LANDING) }) {
            erased = Some(index);
        }
    }
    assert!(converted.expect("conversion save in Link") < erased.expect("unmade erase in Link"));
    assert!(saved_ready, "Link also saves the Ready change with its converted queue");
    assert_eq!(store.domain.landing_of(FAILED_CHANGE), None);
    assert_eq!(store.domain.change(FAILED_CHANGE).unwrap().change.state, ready());
}
