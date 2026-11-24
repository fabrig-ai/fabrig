//! The store/io composition uses the shared world's seeded power-loss model.
use crate::store_world::{Disk, Owner as StoreOwner, Policy};
use skein_fake_machine::Machine as Files;
use skein_io::{file, file_layer};
use skein_lib::{Duration, Queue, Time};
use skein_world::{Cut, Inherited, Memory, Referee, StartupRoot, World};
use smith_local_domain as local;
use smith_local_protocol::{Action, History, Store, StoreRequest};

type Owner = StoreOwner<Reservation>;

struct Reservation {
    reserved: Option<u64>,
    next: Option<local::ChatState>,
    submitted: u32,
    fresh: bool,
    refusal: Option<local::StoreFailure>,
    expects_old: bool,
}

fn make(inherited: &Inherited) -> Owner {
    let file_limits =
        file_layer::Limits { files: 2, read: 4096, entries: 8, whole: 4096, timeout: Duration::from_secs(30) };
    let mut owner = StoreOwner::new(
        inherited.roots[0].1,
        smith_local_protocol_fixtures::STORE,
        file_limits,
        smith_local_process_world::process::lower_configuration().channel_endpoints,
        Some(inherited.signal),
        Reservation {
            reserved: None,
            next: None,
            submitted: 0,
            fresh: inherited.roots[0].0.as_ref() == b"fresh",
            refusal: None,
            expects_old: inherited.roots[0].0.as_ref() == b"older",
        },
    );
    owner.request(StoreRequest::Load, Time::ZERO);
    owner
}

impl Policy for Reservation {
    fn io(&mut self, request: &file::Request) {
        if self.expects_old {
            assert!(
                matches!(
                    request,
                    file::Request::Load { .. } | file::Request::Scan { .. } | file::Request::Close { .. }
                ),
                "older root requested mutation: {request:?}"
            );
        }
    }

    fn action(&mut self, action: Action, now: Time, store: &mut Store, actions: &mut Queue<Action>) {
        match action {
            Action::Domain(local::Event::Loaded { state, .. }) => {
                assert!(!self.expects_old, "an older root must never load");
                let mut state = state.unwrap_or(local::ChatState {
                    identity: smith_domain::run::Identity::new([42; 16]),
                    activation: 0,
                    next_message: 1,
                    read: None,
                });
                state.activation += 1;
                self.next = Some(state);
                store.request(
                    StoreRequest::SaveState { state, history: if self.fresh { History::Fresh } else { History::Kept } },
                    now,
                    actions,
                );
            }
            Action::Domain(local::Event::StateSaved) => {
                self.reserved = Some(self.next.unwrap().activation);
                store.close(now, actions);
            }
            Action::Domain(local::Event::StoreFailed { reason, .. }) => {
                assert!(self.expects_old, "current stores must not refuse: {reason:?}");
                self.refusal = Some(reason);
                store.close(now, actions);
            }
            Action::Fault(_) if self.expects_old => {}
            other => panic!("store reservation terminal: {other:?}"),
        }
    }

    fn submitted(&mut self, count: u32) {
        self.submitted += count;
    }
}

#[derive(Default)]
struct Judge {
    activations: Vec<u64>,
    reserved: bool,
    refused: bool,
}
impl Referee<Owner> for Judge {
    fn watch(&mut self, _: Time, owners: &[Owner]) -> Result<(), String> {
        let owner = owners.first().ok_or("store owner")?;
        self.refused |= owner.policy.refusal.is_some();
        if let Some(activation) = owner.policy.reserved {
            if !self.reserved {
                if !self.activations.last().is_none_or(|prior| *prior < activation) {
                    return Err(String::from("self.activations.last().is_none_or(|prior| *prior < activation)"));
                }
                self.activations.push(activation);
                self.reserved = true;
            }
        } else {
            self.reserved = false;
        }
        Ok(())
    }
    fn passed(&self) -> bool {
        !self.activations.is_empty() || self.refused
    }
}
fn world(seed: u64) -> World<Owner, Judge, Disk> {
    configured(seed, false)
}

fn configured(seed: u64, fresh: bool) -> World<Owner, Judge, Disk> {
    configured_mode(seed, fresh, Initial::Current, skein_sim::Config::calm())
}

#[derive(Clone, Copy, Debug)]
enum Initial {
    Current,
    Window,
    Empty,
    Unrecorded,
    StoreLayout,
    TranscriptLayout,
}

fn initial_files(initial: Initial) -> Vec<skein_fake_machine::Item> {
    use skein_fake_machine::Item;
    let state = local::ChatState {
        identity: smith_domain::run::Identity::new([42; 16]),
        activation: 1,
        next_message: 1,
        read: None,
    };
    match initial {
        Initial::Empty => Vec::new(),
        Initial::Window => {
            let mut items = initial_files(Initial::Current);
            let file = initial_window();
            items.push(Item::file(&file.name, &file.bytes));
            items
        }
        Initial::Current => vec![
            Item::file(b"format", &smith_local_protocol::format::file(&smith_local_protocol::format::layouts()).bytes),
            Item::file(b"state", &smith_local_protocol::save_state(state).bytes),
            Item::file(&delivery().name, &delivery().bytes),
        ],
        Initial::Unrecorded | Initial::StoreLayout | Initial::TranscriptLayout => {
            let mut items = vec![
                Item::file(b"state", &smith_local_protocol::save_state(state).bytes),
                Item::file(b"fresh", &smith_local_protocol::save_state(state).bytes),
                Item::file(b"0000000001.turn", b"old payload"),
                Item::file(&delivery().name, &delivery().bytes),
            ];
            if !matches!(initial, Initial::Unrecorded) {
                let mut layouts = smith_local_protocol::format::layouts();
                match initial {
                    Initial::StoreLayout => layouts.store[0] ^= 1,
                    Initial::TranscriptLayout => layouts.transcript[0] ^= 1,
                    Initial::Current | Initial::Window | Initial::Empty | Initial::Unrecorded => unreachable!(),
                }
                let marker = smith_local_protocol::format::file(&layouts);
                items.push(Item::file(b"format", &marker.bytes));
            }
            items
        }
    }
}

fn delivery() -> smith_local_protocol::File {
    smith_local_protocol::save_delivery(
        &local::DeliveryRecord {
            name: smith_domain::run::CallName { activation: 1, completion: 1, position: 0 },
            state: local::DeliveryState::Intent(local::DeliveryIntent { directories: Box::new([]) }),
            landed: Box::new([]),
        },
        4096,
    )
    .unwrap()
}

fn configured_mode(seed: u64, fresh: bool, initial: Initial, config: skein_sim::Config) -> World<Owner, Judge, Disk> {
    drop(smith_local_process_world::process::lower_configuration());
    let disk = Disk::laid(&initial_files(initial));
    // Peak across power-loss recovery, fresh resets, first marker creation, older-layout
    // refusals, generated whole-turn loading and the extra-seed 713 store sweep.
    // Measured 48_507 B on 2026-11-19, including first-use fingerprint parsing;
    // rounded up to the next 1_000 B.
    const STORE_SCENARIOS_CEILING: u64 = 49_000;
    let mut world =
        World::new(seed, config, Judge::default(), Memory::Within(STORE_SCENARIOS_CEILING)).with_machine(disk);
    world.spawn_restartable(
        vec![StartupRoot {
            name: if matches!(initial, Initial::Unrecorded | Initial::StoreLayout | Initial::TranscriptLayout) {
                b"older".as_slice().into()
            } else if fresh {
                b"fresh".as_slice().into()
            } else {
                b"chat".as_slice().into()
            },
            path: b"chat".as_slice().into(),
        }],
        make,
    );
    world
}

fn run(world: World<Owner, Judge, Disk>) -> skein_world::Outcome<Owner, Disk> {
    let outcome = world.run();
    println!("store seed {} peaks: {:?}", outcome.seed, outcome.peaks());
    outcome
}

mod sweep;

#[test]
fn a_power_loss_at_any_store_operation_resumes_without_reusing_an_activation() {
    let baseline = run(world(70));
    let operations = baseline.procs[0].policy.submitted;
    assert!(operations >= 12);
    for before in 0..operations {
        let mut world = world(70 + u64::from(before));
        world.cut(0, before, Cut::PowerLoss);
        let outcome = run(world);
        assert!(outcome.procs[0].policy.reserved.is_some(), "cut {before}");
        assert_eq!(outcome.machine.files().open_handles(), 0);
    }
}

#[test]
fn a_fresh_journal_resumes_its_reset_after_a_power_loss() {
    let baseline = run(configured(80, true));
    for before in 0..baseline.procs[0].policy.submitted {
        let mut world = configured(80 + u64::from(before), true);
        world.cut(0, before, Cut::PowerLoss);
        let outcome = run(world);
        let root = outcome.machine.files().reopen_root(0);
        let names = outcome.machine.files().list(root, 6, 4096).unwrap();
        assert_eq!(
            names.iter().filter(|(_, name)| name.len() == 42 || name.as_ref() == b"fresh").count(),
            0,
            "cut {before}"
        );
        outcome.machine.files().close(root);
    }
}

#[test]
fn interrupted_initial_marker_creation_resumes_without_adopting_old_records() {
    sweep::cuts(7000, false, Initial::Empty);
}

fn disk_snapshot(files: &mut Files) -> Vec<(Box<[u8]>, skein_fake_machine::Facts, Vec<u8>)> {
    let root = files.reopen_root(0);
    let names = files.list(root, 8, 4096).unwrap();
    let mut snapshot = Vec::new();
    for (_, name) in names {
        let file = files.open(root, &name, skein_fake_machine::How::ReadNoFollow).unwrap();
        snapshot.push((name, files.stat(file), files.read(file, 0, 4096).unwrap()));
        files.close(file);
    }
    files.close(root);
    snapshot
}

#[test]
fn older_refusal_is_immutable_under_power_loss_and_injected_read_errors() {
    for initial in [Initial::Unrecorded, Initial::StoreLayout, Initial::TranscriptLayout] {
        let mut expected = Files::new();
        let root = expected.lay(&initial_files(initial));
        expected.close(root);
        let expected = disk_snapshot(&mut expected);
        for io_error in [0, 300, 1000] {
            let mut config = skein_sim::Config::calm();
            config.faults.io_error = io_error;
            let baseline = run(configured_mode(8000, false, initial, config));
            for before in 0..baseline.procs[0].policy.submitted {
                let seed = 8000 + u64::from(before) + u64::from(io_error);
                let mut world = configured_mode(seed, false, initial, config);
                world.cut(0, before, Cut::PowerLoss);
                let outcome = run(world);
                assert!(outcome.procs[0].policy.refusal.is_some(), "{initial:?} seed {seed}");
                assert_eq!(expected, disk_snapshot(&mut outcome.machine.files()), "{initial:?} seed {seed}");
                assert_eq!(outcome.machine.files().open_handles(), 0);
            }
        }
    }
}

#[test]
#[ignore = "explicit 713 extra-seed QA sweep, outside the normal gate"]
fn extra_seed_713_store_power_loss_proof() {
    const KEPT_BASE: u64 = 713_000;
    const FRESH_BASE: u64 = 714_000;
    const FIRST_CREATE_BASE: u64 = 715_000;
    const SECOND_CREATE_BASE: u64 = 716_000;
    for (base, fresh, initial) in [
        (KEPT_BASE, false, Initial::Current),
        (FRESH_BASE, true, Initial::Current),
        (FIRST_CREATE_BASE, false, Initial::Empty),
        (SECOND_CREATE_BASE, true, Initial::Empty),
    ] {
        let operations = sweep::cuts(base, fresh, initial);
        println!("713 extra sweep: {initial:?}, fresh={fresh}, seed base {base}, cuts 0..{operations}");
    }
}

#[test]
fn generated_whole_turn_loading_and_recovery_fit_the_shared_metered_store_bound() {
    sweep::cuts(9000, false, Initial::Window);
}

fn initial_window() -> smith_local_protocol::File {
    let endpoints = smith_local_process_world::process::lower_configuration().channel_endpoints;
    let turn = smith_domain::Turn {
        dialect: 3,
        ..smith_domain_session_fixtures::turn(
            b"model".as_slice().into(),
            smith_domain::session::record::Kind::First(Box::new(smith_domain::session::record::Opener {
                system: None,
                affinity: smith_domain::llm::Affinity { key: [0; 16], thread: 0 },
                account: None,
                backend: smith_domain::session::record::Backend::Direct,
            })),
            Box::new([smith_domain::session::llm::Message {
                role: smith_domain::session::llm::Role::Assistant,
                content: Box::new([smith_domain::session::llm::Block::Text {
                    text: Box::new([b'x'; 3000]),
                    replay: None,
                }]),
            }]),
        )
    };
    let file = smith_local_protocol::save_turn(1, None, true, &turn, &smith_transcript::CEILINGS, &endpoints).unwrap();
    assert!(file.bytes.len() > 3000 && file.bytes.len() < 4096);
    file
}
