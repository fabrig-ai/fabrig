//! Direct Core mechanics only. Real journal/store release, crash recovery and
//! the durable readback oracle belong to the reference-root integration.
use jig_core_world::world;
use jig_domain_accounts as accounts;
use jig_domain_core::{
    self as core,
    start::{Action, Failure, LiveSet, Phase, SweepInput},
};
use jig_domain_people as people;
use skein_lib::{Env, Queue, Time, Wall};
use skein_world::domain::heap::Meter;

struct Harness {
    core: core::Core,
    env: Env<core::Limits>,
    out: Queue<core::Request>,
}

impl Harness {
    fn new(seed: u64) -> Self {
        let limits = world::limits().core;
        Self {
            core: core::Core::new(world::config(seed).core, &limits).expect("configured direct Core"),
            env: Env { now: Time::ZERO, wall: Wall::from_nanos(10), limits },
            out: Queue::uncapped(),
        }
    }

    fn select(&mut self, pending: bool) {
        phase(&self.core.start_begin(pending, self.env.wall), Phase::LoadCore);
    }

    fn link(&mut self) -> Action {
        phase(&self.core.start_done(&self.env, Phase::LoadCore, &mut self.out), Phase::LoadInbox);
        let mut previous = Phase::LoadInbox;
        for connector in self.core.connectors.clone() {
            let next = Phase::RestoreConnector { connector };
            phase(&self.core.start_done(&self.env, previous, &mut self.out), next);
            previous = next;
        }
        phase(&self.core.start_done(&self.env, previous, &mut self.out), Phase::Link);
        self.core.start_done(&self.env, Phase::Link, &mut self.out)
    }

    fn record(&mut self, record: accounts::Stored) {
        let core::Requests::Out(out) =
            core::step(&mut self.core, &self.env, core::Event::Account(accounts::Event::Restore { record }));
        assert!(out.len() <= 1);
    }
}

fn phase(action: &Action, expected: Phase) {
    assert!(matches!(action, Action::Step(found) if *found == expected));
}

fn account(number: u32) -> accounts::Stored {
    accounts::Stored::Account {
        number,
        provider: 1,
        holder: accounts::Holder::Deployment,
        place: number,
        uses: None,
        subject: Box::new([]),
        display: Box::new([]),
    }
}

#[test]
fn stored_account_bound_is_typed_for_active_and_pending_starts() {
    for pending in [false, true] {
        let mut h = Harness::new(201);
        h.select(pending);
        let count = h.env.limits.accounts.accounts.checked_add(1).expect("small fixture account bound");
        for number in 1..=count {
            h.record(account(number));
        }
        let expected = Failure::AccountsPastLimit {
            failure: accounts::RestoreLimit {
                key: accounts::Key::Account { number: count },
                set: accounts::RestoreSet::Accounts,
                limit: h.env.limits.accounts.accounts,
            },
        };
        assert_eq!(h.core.start_failure(), Some(expected.clone()));
        assert!(
            matches!(h.core.start_done(&h.env, Phase::LoadCore, &mut h.out), Action::Refused(found) if found == expected)
        );
        assert!(h.out.is_empty());
    }
}

#[test]
fn stored_people_bytes_keep_the_outer_key_and_bound() {
    let mut h = Harness::new(202);
    h.select(false);
    let limit = h.env.limits.people.identity_bytes;
    let held = limit.checked_add(1).expect("small fixture identity bound");
    let record = people::Stored::Person {
        number: 41,
        identity: people::Identity {
            key: people::IdentityKey {
                provider: 0,
                subject: vec![b'x'; usize::try_from(held).expect("identity fixture fits usize")].into_boxed_slice(),
            },
            login: Box::new([]),
            name: Box::new([]),
        },
    };
    let _out = core::step(&mut h.core, &h.env, core::Event::People(people::Event::Restore { record }));
    assert_eq!(
        h.core.start_failure(),
        Some(Failure::LiveSet {
            set: LiveSet::PeopleBound(people::RestoreBound::IdentityBytes),
            key: core::Key::People(people::Key::Person(41)),
            held: u64::from(held),
            limit: u64::from(limit),
        })
    );
}

#[test]
fn cleanup_acknowledgement_blocks_sweep_and_held_account_outputs_are_metered() {
    let mut h = Harness::new(203);
    h.select(false);
    let meter = Meter::new();
    meter.start();
    h.record(account(1));
    h.record(accounts::Stored::Source {
        account: 1,
        number: 2,
        place: 0,
        kind: accounts::Kind::Key,
        name: Box::new([]),
        held: None,
    });
    let Action::Commit(cleanup) = h.link() else { panic!("counter cleanup requires a commit") };
    let captured = meter.end();
    meter.check(captured, 18_000, &h.env.limits);
    assert!(captured.held() > 0);
    println!("selected account loading and held outputs: peak={} held={}", captured.peak(), captured.held());
    let mut saved_counter = false;
    for _ in 0..h.out.len() {
        let core::Request::Write(core::Write::Save(core::Record::Accounts(accounts::Stored::Next { number }))) =
            h.out.pop().expect("cleanup output")
        else {
            panic!("only cleanup writes before durable Wires")
        };
        assert_eq!(number, 3);
        saved_counter = true;
    }
    assert!(saved_counter);
    let rejected = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        h.core.start_sweep(&h.env, SweepInput::Finished, &mut h.out)
    }));
    assert!(rejected.is_err());
    assert!(h.out.is_empty());
    phase(&h.core.start_resume(&h.env, cleanup, &mut h.out), Phase::Sweep);
    phase(&h.core.start_sweep(&h.env, SweepInput::Finished, &mut h.out), Phase::Switch);
    phase(&h.core.start_done(&h.env, Phase::Switch, &mut h.out), Phase::Wires);
    let Action::Commit(wires) = h.core.start_done(&h.env, Phase::Wires, &mut h.out) else {
        panic!("Wires requires release")
    };
    assert!(h.out.is_empty());
    assert!(h.core.tasks.checked());
    meter.start();
    phase(&h.core.start_resume(&h.env, wires, &mut h.out), Phase::AdoptRuns);
    let released = meter.end();
    meter.check(released, 18_000, &h.env.limits);
    println!("selected account release including owned outputs: peak={} held={}", released.peak(), released.held());
    assert!(!h.core.tasks.checked());
    assert!(!h.out.is_empty());
}
