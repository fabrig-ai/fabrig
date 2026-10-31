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
        let mut header = self.core.counters.deployment();
        header.commits = 1;
        assert!(matches!(
            self.core.restore_core(core::CoreRecord::Deployment(header), &self.env.limits),
            core::Restored::Deployment { commits: 1 }
        ));
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
fn stored_account_bound_keeps_its_detail_for_active_and_pending_starts() {
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

#[test]
fn an_empty_selected_store_seeds_before_opening() {
    let configured = world::config(204).core;
    let owners = configured.owners.clone();
    let mut h = Harness::new(204);
    phase(&h.core.start_begin(false, h.env.wall), Phase::LoadCore);
    let Action::Commit(seed) = h.core.start_done(&h.env, Phase::LoadCore, &mut h.out) else {
        panic!("empty seed requires a commit")
    };
    assert!(!h.core.people.ready());
    let mut policy = false;
    let mut deployment_roles = false;
    let mut project_roles = false;
    let mut settings = false;
    for _ in 0..h.out.len() {
        let core::Request::Write(core::Write::Save(record)) = h.out.pop().expect("seed write") else {
            panic!("seed emits only saved rows")
        };
        #[expect(
            clippy::wildcard_enum_match_arm,
            reason = "seed commits contain only configured people and settings rows"
        )]
        match record {
            core::Record::People(people::Stored::Policy { .. }) => policy = true,
            core::Record::People(people::Stored::Roles { project, .. }) => {
                if project == people::DEPLOYMENT {
                    deployment_roles = true;
                } else {
                    project_roles = true;
                }
            }
            core::Record::Core(core::CoreRecord::Settings(_)) => settings = true,
            core::Record::People(people::Stored::Person { .. }) => {}
            _ => panic!("unexpected seed family"),
        }
    }
    assert!(policy && deployment_roles && project_roles && settings);
    let mut action = h.core.start_resume(&h.env, seed, &mut h.out);
    for _ in 0..32 {
        action = match action {
            Action::Step(Phase::Sweep) => h.core.start_sweep(&h.env, SweepInput::Finished, &mut h.out),
            Action::Step(done) => h.core.start_done(&h.env, done, &mut h.out),
            Action::Commit(token) => h.core.start_resume(&h.env, token, &mut h.out),
            Action::Idle => break,
            Action::Refused(failure) => panic!("seed refused: {failure:?}"),
        };
        while h.out.pop().is_some() {}
    }
    assert_eq!(h.core.start_phase(), None);
    assert!(h.core.people.ready());
    for owner in owners {
        let (person, role) = h.core.people.role_for_identity(owner.identity, owner.project).expect("seed owner");
        assert_eq!(role, people::Role::Owner);
        assert_eq!(h.core.people.role(person, people::DEPLOYMENT), Some(people::Role::Owner));
        assert!(h.core.authority.policy(owner.project).is_some());
    }
    assert!(h.core.charter(1).is_some());
}
