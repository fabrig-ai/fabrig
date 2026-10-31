//! Output close handoffs and private-store lifecycle (shell.md, sections 4.2 and 9).

use super::{Service, local};
use skein_fake_machine::{How, Is, Item, Machine, Opened};
use skein_io::kernel;
use skein_lib::{List, Queue, Time};
use smith_local_protocol::{History, StoreRequest};

const SEEDS: [u64; 2] = [8, 19];
const PRIVATE: &[u8] = b"private";
const FOREIGN: &[u8] = b"kept foreign content";
// The private fixture writes state, one turn and one delivery record.
const OWNED_FILES: u32 = 3;
// One foreign entry plus the three files written by the fixture.
const LIST_ENTRIES: u32 = OWNED_FILES + 1;
// Each listed entry fits a name of at most 255 bytes and its directory record.
const LIST_BYTES: u32 = LIST_ENTRIES * 512;
const DELIVERY: smith_domain::run::CallName = smith_domain::run::CallName { activation: 1, completion: 1, position: 0 };

struct PrivateWorld {
    service: Service,
    sim: skein_sim::Sim,
    pid: skein_sim::Pid,
    machine: Machine,
    root: Opened,
    events: Queue<local::Event>,
}

impl PrivateWorld {
    #[expect(clippy::disallowed_macros, reason = "ordinary Rust fixture data (programming-model.md, section 10.2)")]
    fn new(seed: u64, extra: &[Item]) -> Self {
        let mut items = vec![Item::directory(PRIVATE).mode(0o700)];
        items.extend_from_slice(extra);
        let mut machine = Machine::new();
        let root = machine.lay(&items);
        let store = machine.open(root, PRIVATE, How::Directory).unwrap();
        let parent = machine.open(root, b".", How::Directory).unwrap();
        let mut sim = skein_sim::Sim::new(seed, skein_sim::Config::calm());
        let pid = sim.spawn_process();
        let store = sim.root(pid, skein_sim::Handle::new(store.raw()));
        let parent = sim.root(pid, skein_sim::Handle::new(parent.raw()));
        let mut config = crate::tests::config();
        let mut endpoints = List::with_capacity(1);
        endpoints
            .push(smith_protocol_channel::Endpoint { name: b"main".as_slice().into(), number: 4, dialect: 1 })
            .unwrap();
        config.endpoints = smith_protocol_channel::Endpoints::new(endpoints);
        let mut service = Service::new(config, seed).unwrap();
        service.adopt_store(store).unwrap();
        service.ephemeral(parent, PRIVATE.into()).unwrap();
        Self { service, sim, pid, machine, root, events: Queue::uncapped() }
    }

    fn settle(&mut self) {
        crate::tests::settle_file_io(
            &mut self.service.io,
            &mut self.sim,
            &mut self.machine,
            self.pid,
            &mut self.events,
        );
    }

    fn save_known_files(&mut self) {
        let state = local::ChatState {
            identity: smith_domain::run::Identity::new([42; 16]),
            activation: 1,
            next_message: 1,
            read: None,
        };
        for request in [
            StoreRequest::SaveState { state, history: History::Fresh },
            StoreRequest::SaveTurn { number: 1, read: None, opens_window: true, turn: turn() },
            StoreRequest::SaveDelivery {
                record: Box::new(local::DeliveryRecord {
                    name: DELIVERY,
                    state: local::DeliveryState::Intent(local::DeliveryIntent { directories: Box::new([]) }),
                    landed: Box::new([]),
                }),
            },
        ] {
            self.service.io.store_request(request);
            self.settle();
        }
    }

    fn end(&mut self, status: local::ExitStatus) {
        self.service.route_local(self.sim.now(), local::Request::Exit { status });
        self.settle();
        self.service.advance_end();
    }

    fn finish(&mut self) -> local::ExitStatus {
        let status = finish_output(&mut self.service);
        assert!(self.service.io.closed());
        self.machine.close(self.root);
        assert_eq!(self.machine.open_handles(), 0, "store, cleanup parent and observer close");
        status
    }

    fn assert_saved(&mut self) {
        let Some(local::Event::StateSaved) = self.events.pop() else { panic!("state written") };
        let Some(local::Event::TurnSaved { number: 1 }) = self.events.pop() else { panic!("turn written") };
        let Some(local::Event::DeliverySaved { name: DELIVERY }) = self.events.pop() else {
            panic!("delivery written")
        };
        assert!(self.events.is_empty());
    }
}

fn turn() -> smith_domain::Turn {
    smith_local_domain::fixture::turn()
}

fn assert_removed_after_exit(status: local::ExitStatus) {
    for seed in SEEDS {
        let mut world = PrivateWorld::new(seed, &[]);
        world.save_known_files();
        world.end(status);
        world.assert_saved();
        assert_eq!(world.machine.list(world.root, LIST_ENTRIES, LIST_BYTES).unwrap(), []);
        assert_eq!(world.service.cleanup_warning(), None);
        assert!(world.service.shell.is_empty(), "the shell still owns its last output");
        assert_eq!(world.finish(), status);
    }
}

/// (top tiers test wiring only).
#[test]
fn acceptance_removes_private_records_and_their_directory() {
    assert_removed_after_exit(local::ExitStatus::Accepted);
}

/// (top tiers test wiring only).
#[test]
fn failure_removes_private_records_and_their_directory() {
    assert_removed_after_exit(local::ExitStatus::RunFailed);
}

/// (top tiers test wiring only); warning text belongs to the shell.
#[test]
fn a_foreign_file_survives_and_warns_before_exit() {
    for seed in SEEDS {
        let mut world = PrivateWorld::new(seed, &[Item::file(b"private/foreign.txt", FOREIGN)]);
        world.save_known_files();
        world.end(local::ExitStatus::Accepted);
        world.assert_saved();
        let directory = world.machine.open(world.root, PRIVATE, How::Directory).unwrap();
        assert_eq!(world.machine.stat(directory).mode, 0o700);
        assert_eq!(
            world.machine.list(directory, LIST_ENTRIES, LIST_BYTES).unwrap(),
            [(Is::File, b"foreign.txt".as_slice().into())]
        );
        let foreign = world.machine.open(directory, b"foreign.txt", How::Read).unwrap();
        assert_eq!(world.machine.read(foreign, 0, u32::try_from(FOREIGN.len()).unwrap()).unwrap(), FOREIGN);
        world.machine.close(foreign);
        world.machine.close(directory);
        assert!(world.service.shell.is_empty(), "the final exit waits for the shell's warning output");
        assert_eq!(world.service.cleanup_warning(), Some(kernel::Error::NotEmpty));
        assert_eq!(world.service.cleanup_warning(), None, "the warning is handed off once");
        assert_eq!(world.finish(), local::ExitStatus::Accepted);
    }
}

fn closing(status: local::ExitStatus) -> Service {
    let mut service = Service::new(crate::tests::config(), 7).unwrap();
    service.route_local(Time::ZERO, local::Request::Exit { status });
    service
}

fn finish_output(service: &mut Service) -> local::ExitStatus {
    // Components, the stream and the signal owner each report once.
    const CLOSE_PHASES: u32 = 3;
    service.shell_closed();
    for _ in 0..CLOSE_PHASES {
        service.advance_end();
    }
    let Some(local::Request::Exit { status }) = service.shell.pop() else { panic!("final shell exit") };
    assert!(service.shell.is_empty(), "one exit after the shell closes");
    status
}

/// (top tiers test wiring only); the shell owns the file operation.
#[test]
fn result_failure_changes_acceptance_before_the_final_exit() {
    let mut service = closing(local::ExitStatus::Accepted);
    let changed = service.result_file_failed();
    service.advance_end();
    assert!(changed);
    assert!(service.shell.is_empty(), "no last line before result-file output settles");
    assert_eq!(finish_output(&mut service), local::ExitStatus::RunFailed);
}

/// (top tiers test wiring only); warning text belongs to the shell.
#[test]
fn result_failure_keeps_the_parked_exit_behind_shell_output() {
    let mut service = closing(local::ExitStatus::NeedsInput);
    let changed = service.result_file_failed();
    service.advance_end();
    assert!(!changed);
    assert!(service.shell.is_empty(), "the shell's warning precedes its last line");
    assert_eq!(finish_output(&mut service), local::ExitStatus::NeedsInput);
}
