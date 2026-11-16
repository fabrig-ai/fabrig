//! The upgrade phase on a simulated disk (protocol/store/opening.md 2 and 4).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::wildcard_enum_match_arm,
    clippy::match_like_matches_macro,
    reason = "disk step tests use ordinary Rust (programming-model.md 10.2)"
)]
use super::fixtures::{
    self,
    disk::{Disk, Seen, file_limits},
};
use super::owners::{REGISTERED as BASE, SETS, label, shelf};
use crate::directory::{OWNER, Opening};
use crate::patching::phase::Patching;
use crate::{Arrived, Event, Get, Io, OpenRefusal, Phase, Request, Root, Row, Store, Write};
use skein_fake_machine::{How, Is, Item, Machine, Opened};
use skein_io::{
    file,
    file_layer::{FileIo, Limits as FileLimits, step},
    kernel::{Complete, Error, Kind, Submit},
};
use skein_lib::{Queue, Token, length::len_u64};
use skein_sim::{Answer, Ask, Config, Handle, Pid, Sim};

mod charter_versions;
mod checked;
mod floor;
mod jobs;
mod registry;
mod store_floor;
mod task_logs;
mod value_types;

const CHUNK: u32 = 4096;
const KEY: &[u8] = b"row";
const VALUE: &[u8] = b"committed";
fn assert_opened(events: &[Event], expected: u64) {
    let [Event::Opened { owner, last, .. }, Event::Census { .. }] = events else { panic!("one opened: {events:?}") };
    assert_eq!(*owner, Token::new(1));
    assert_eq!(*last, expected);
}

fn bridge_open(
    store: &mut Store,
    env: &skein_lib::Env<crate::Limits>,
    event: &Event,
    switching: &mut bool,
    above: &mut Queue<Event>,
    below: &mut crate::Below,
) -> bool {
    if let Event::Opened { pending: true, owner, .. } = event
        && !*switching
    {
        *switching = true;
        // This store-only driver has no engine root to ask for the switch.
        crate::down(store, env, Request::Switch { owner: *owner }, above, below);
    }
    if let Event::Switched { .. } = event { true } else { false }
}
fn baseline() -> Vec<Item> {
    baseline_with_proof(false)
}
fn baseline_with_proof(proof: bool) -> Vec<Item> {
    baseline_with_sets(proof, upgrade_sets(0))
}
fn baseline_with_sets(proof: bool, sets: crate::PatchSets) -> Vec<Item> {
    let (mut disk, _) = source_with_sets(proof, sets);
    disk.items()
}
fn source_with_sets(proof: bool, sets: crate::PatchSets) -> (Disk, Store) {
    let mut disk = Disk::new(&[]);
    let mut store = disk.store_with_chunk(CHUNK);
    store.patch_sets(sets);
    let events = disk.open(&mut store);
    assert!({ if let [Event::Opened { last: 0, .. }, Event::Census { .. }] = events.as_ref() { true } else { false } });
    let mut writes =
        vec![fixtures::header(1), Write::Put { key: Box::from(KEY), value: Box::from(VALUE) }, Write::Wires];
    if proof {
        writes.push(fixtures::save(fixtures::run_proof(1)));
    }
    let events = disk.request(&mut store, fixtures::commit(1, writes.into_boxed_slice()));
    assert!({ if let [Event::Committed { number: 1, .. }] = events.as_ref() { true } else { false } });
    (disk, store)
}
fn read_row(disk: &mut Disk, store: &mut Store) {
    let events = disk.request(store, Request::Get { owner: Token::new(2), key: Get::Application(Box::from(KEY)) });
    let [Event::Got { row: Some(Row::Application { key, value }), .. }] = events.as_ref() else {
        panic!("committed application row: {events:?}")
    };
    assert_eq!(key.as_ref(), KEY);
    assert_eq!(value.as_ref(), VALUE);
}
fn upgrade(disk: &mut Disk, generation: usize) -> Store {
    let mut store = disk.store_with_chunk(CHUNK);
    store.patch_sets(upgrade_sets(generation));
    let events = disk.open(&mut store);
    let events = finish_empty_jobs(disk, &mut store, events);
    assert_opened(&events, store.stores.store.last());
    let events = disk.request(&mut store, Request::Switch { owner: Token::new(1) });
    assert!({ if let [Event::Switched { owner }] = events.as_ref() { *owner == Token::new(1) } else { false } });
    store
}
#[test]
fn an_upgrade_opens_the_copy_with_the_committed_row_and_keeps_the_previous_whole() {
    let mut disk = Disk::new(&baseline());
    let mut store = upgrade(&mut disk, 1);
    assert_eq!(disk.bytes(b"active").as_ref(), b"2\n");
    assert_manifest(
        &disk.bytes(b"generations/2/manifest"),
        2,
        crate::levels(&upgrade_sets(0)),
        crate::levels(&upgrade_sets(1)),
        3,
    );
    let contents = disk.contents();
    // The maps are copied; each generation has its own manifest.
    for (path, kind, _, bytes) in contents
        .iter()
        .filter(|(p, ..)| p.starts_with(b"generations/1/store") || p.starts_with(b"generations/1/secrets"))
    {
        let target = [b"generations/2/", path.strip_prefix(b"generations/1/").expect("prefix")].concat();
        let (_, copied_kind, _, copied_bytes) =
            contents.iter().find(|(p, ..)| p.as_ref() == target).expect("whole copy");
        assert_eq!(copied_kind, kind);
        if path.starts_with(b"generations/1/secrets/") {
            assert_eq!(copied_bytes, bytes);
        }
    }
    read_row(&mut disk, &mut store);
    disk.crash(17);
    let mut store = disk.store_with_chunk(CHUNK);
    store.patch_sets(upgrade_sets(1));
    assert_opened(&disk.open(&mut store), 3);
    assert_eq!(store.directory.active.expect("active read").number, 2);
    assert!(
        !disk
            .requests()
            .iter()
            .any(|(driver, r)| *driver == 2 && { if let Seen::Remove { .. } = r { true } else { false } })
    );
    read_row(&mut disk, &mut store);
}
#[test]
fn the_next_job_drops_one_keeps_two_and_opens_three() {
    let mut first = Disk::new(&baseline());
    upgrade(&mut first, 1);
    let mut disk = Disk::new(&first.items());
    let mut store = upgrade(&mut disk, 2);
    assert_eq!(disk.bytes(b"active").as_ref(), b"3\n");
    assert_manifest(
        &disk.bytes(b"generations/3/manifest"),
        3,
        crate::levels(&upgrade_sets(1)),
        crate::levels(&upgrade_sets(2)),
        5,
    );
    let tree = disk.tree();
    assert!(!tree.iter().any(|(p, ..)| p.as_ref() == b"generations/1"));
    for generation in [2_u64, 3] {
        for map in ["store", "secrets"] {
            let path = format!("generations/{generation}/{map}");
            assert!(
                tree.iter()
                    .any(|(p, kind, mode)| p.as_ref() == path.as_bytes() && *kind == Kind::Directory && *mode == 0o700)
            );
        }
    }
    read_row(&mut disk, &mut store);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Other,
    Closing,
    Dropping,
    Copying,
    Opening,
    Jobs,
    Sealing,
    Checking,
    Manifest,
    Switching,
}
fn stage(store: &Store) -> Stage {
    match &store.directory.opening {
        Opening::Patching(phase) => match **phase {
            Patching::Closing { .. } => Stage::Closing,
            Patching::Dropping(_) => Stage::Dropping,
            Patching::Copying(_) => Stage::Copying,
            Patching::Mapping { .. } | Patching::Recovering { .. } => Stage::Opening,
            Patching::Walking(_) | Patching::Jobs(_) => Stage::Jobs,
            Patching::Sealing { .. } => Stage::Sealing,
            Patching::Checking(_) => Stage::Checking,
        },
        Opening::Pending(pending) => match pending.stage {
            crate::patching::switch::Stage::Manifest { .. } => Stage::Manifest,
            crate::patching::switch::Stage::Switching { .. } => Stage::Switching,
            _ => Stage::Other,
        },
        _ => Stage::Other,
    }
}
#[derive(Debug, PartialEq, Eq)]
enum Trace {
    KvClose(usize),
    MapClose(usize),
    MapOpen(usize, Box<[u8]>),
    KvOpen(usize),
    Store(Box<[u8]>),
    Load(Box<[u8]>),
    Cut,
    Remove(Box<[u8]>),
    Other,
}
fn trace(driver: usize, request: &file::Request) -> Trace {
    match request {
        file::Request::Close { owner, .. } if *owner == OWNER => Trace::MapClose(driver),
        file::Request::Close { .. } => Trace::KvClose(driver),
        file::Request::OpenDirectory { name, .. } => Trace::MapOpen(driver, name.clone()),
        file::Request::OpenPrivate { path, .. } => Trace::MapOpen(driver, path.clone()),
        file::Request::List { .. } => Trace::KvOpen(driver),
        file::Request::Store { path, .. } => Trace::Store(path.clone()),
        file::Request::Load { path, .. } => Trace::Load(path.clone()),
        file::Request::Cut { .. } => Trace::Cut,
        file::Request::Remove { name, .. } | file::Request::RemoveIn { name, .. } => Trace::Remove(name.clone()),
        _ => Trace::Other,
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Change {
    Create,
    Write,
    Sync,
    Rename,
    Remove,
    Cut,
    Make,
    Other,
}
fn change(ask: &Ask) -> Change {
    match ask {
        Ask::Open {
            how: skein_io::kernel::OpenHow::Create { .. } | skein_io::kernel::OpenHow::CreateNoFollow { .. },
            ..
        } => Change::Create,
        Ask::Write { .. } | Ask::Append { .. } => Change::Write,
        Ask::Remove { .. } => Change::Remove,
        Ask::Truncate { .. } => Change::Cut,
        Ask::MakeDirectory { .. } => Change::Make,
        Ask::Sync { .. } => Change::Sync,
        Ask::Rename { .. } => Change::Rename,
        _ => Change::Other,
    }
}
/// Same three real file drivers as `fixtures::disk`, with a seam inside Store's
/// durable replace. Cuts and faults act on the simulator's machine calls.
struct Harness {
    machine: Machine,
    handle: Opened,
    sim: Sim,
    pid: Pid,
    root: Root,
    io: [FileIo; 3],
    file_limits: FileLimits,
    seeded: crate::Seeded,
    trace: Vec<(Stage, Trace)>,
    changes: Vec<(Stage, Change)>,
    cut: Option<(usize, bool)>,
    fault_write: bool,
    fault_at: Option<Stage>,
    halted: bool,
    source: Option<Vec<Item>>,
    check_entries: Vec<Item>,
    wall: skein_lib::Wall,
}
impl Harness {
    fn new(items: &[Item]) -> Self {
        let mut machine = Machine::for_user(0);
        let handle = machine.lay(items);
        Self::over(machine, handle)
    }
    fn over(machine: Machine, handle: Opened) -> Self {
        let mut sim = Sim::new(1, Config::calm());
        let pid = sim.spawn_process();
        let fd = sim.root(pid, Handle::new(handle.raw()));
        let mut io = core::array::from_fn(|_| {
            let mut io = FileIo::with_limits(&file_limits()).with_effective_user(0);
            assert_eq!(io.adopt_root(fd), Some(Token::new(1)));
            io
        });
        let [public, private, transcripts] = &mut io;
        let seeded = crate::seed([public, private, transcripts], 1);
        Self {
            machine,
            handle,
            sim,
            pid,
            root: Root { token: Token::new(1), fd },
            io,
            file_limits: file_limits(),
            seeded,
            trace: Vec::new(),
            changes: Vec::new(),
            cut: None,
            fault_write: false,
            fault_at: None,
            halted: false,
            source: None,
            check_entries: Vec::new(),
            wall: skein_lib::Wall::EPOCH,
        }
    }
    fn store(&self, patch: bool) -> Store {
        let mut limits = fixtures::store_limits();
        limits.store.chunk = CHUNK;
        limits.store.value =
            CHUNK.checked_sub(limits.store.key).expect("key fits").checked_sub(20).expect("header fits");
        limits.secrets.chunk = CHUNK;
        limits.secrets.value =
            CHUNK.checked_sub(limits.secrets.key).expect("key fits").checked_sub(20).expect("header fits");
        let mut store = Store::on_directory(&limits, fixtures::APPLICATION, &[], self.seeded);
        store.patch_sets(upgrade_sets(0));
        if patch {
            store.patch_sets(upgrade_sets(1));
        }
        store
    }
    fn open(&mut self, store: &mut Store) -> Box<[Event]> {
        let mut env = fixtures::env();
        env.wall = self.wall;
        let mut above = Queue::with_capacity(128);
        let mut below = fixtures::below();
        crate::down(
            store,
            &env,
            Request::Open { owner: Token::new(1), root: self.root, installation: fixtures::INSTALLATION },
            &mut above,
            &mut below,
        );
        let mut switching = false;
        for _ in 0_u32..10_000 {
            let mut retained = Vec::new();
            while let Some(event) = above.pop() {
                if bridge_open(store, &env, &event, &mut switching, &mut above, &mut below) {
                    continue;
                }
                if let Event::Job { owner, page, .. } = event {
                    assert!(page.end && page.rows.is_empty());
                    crate::down(
                        store,
                        &env,
                        Request::Job {
                            owner,
                            answer: Ok(crate::JobAnswer { writes: Box::new([]), through: None, counts: Box::new([]) }),
                        },
                        &mut above,
                        &mut below,
                    );
                } else if if let Event::Ran { .. } = event { false } else { true } {
                    retained.push(event);
                }
            }
            for event in retained {
                above.push(event);
            }
            crate::fire(store, &env, &mut above, &mut below);

            let next = below
                .store
                .pop()
                .map(|r| (0, r))
                .or_else(|| below.secrets.pop().map(|r| (1, r)))
                .or_else(|| below.transcripts.pop().map(|r| (2, r)));
            let Some((driver, request)) = next else {
                if stage(store) == Stage::Jobs {
                    continue;
                }
                return fixtures::events(&mut above);
            };
            let stage = stage(store);
            if stage == Stage::Checking && !self.check_entries.is_empty() {
                let items = core::mem::take(&mut self.check_entries);
                self.lay_private(&items);
            }
            if stage == Stage::Closing && self.source.is_none() {
                self.source = Some(self.source_items(store.directory.active.expect("active read").number));
            }
            self.trace.push((stage, trace(driver, &request)));
            let Some(arrived) = self.file(driver, request, stage) else {
                return fixtures::events(&mut above);
            };
            let io = match driver {
                0 => Io::Store(arrived),
                1 => Io::Secrets(arrived),
                2 => Io::Transcripts(arrived),
                _ => unreachable!(),
            };
            crate::up(store, &env, io, &mut above, &mut below);
        }
        panic!("phase did not settle");
    }
    fn file(&mut self, driver: usize, request: file::Request, stage: Stage) -> Option<Arrived> {
        let mut events = Queue::with_capacity(128);
        let mut subs: Queue<Submit> = Queue::with_capacity(128);
        let mut completions: Queue<Complete> = Queue::with_capacity(128);
        let env = fixtures::env().with(self.file_limits);
        step::down(&mut self.io[driver], &env, request, &mut events, &mut subs);
        for _ in 0_u32..100 {
            self.sim.submit(self.pid, &mut subs);
            let mut calls = Queue::with_capacity(128);
            let mut answers = Queue::with_capacity(128);
            self.sim.calls(&mut calls);
            while let Some(call) = calls.pop() {
                let changing = change(&call.ask);
                let targeted = (stage == Stage::Manifest
                    || stage == Stage::Opening
                    || stage == Stage::Jobs
                    || stage == Stage::Sealing
                    || stage == Stage::Checking)
                    && changing != Change::Other;
                let ordinal = self.changes.len();
                if targeted {
                    self.changes.push((stage, changing));
                    if self.cut == Some((ordinal, false)) {
                        self.halted = true;
                        return None;
                    }
                }
                if (changing == Change::Write || stage == Stage::Checking)
                    && ((self.fault_write && stage == Stage::Manifest) || self.fault_at == Some(stage))
                {
                    self.fault_write = false;
                    self.fault_at = None;
                    answers.push(Answer { ticket: call.ticket, result: Err(Error::NoSpace) });
                } else {
                    skein_fake_machine::step(&mut self.machine, call, &mut answers);
                }
                self.sim.answer(&mut answers);
                if targeted && self.cut == Some((ordinal, true)) {
                    self.halted = true;
                    return None;
                }
            }
            self.sim.reap(self.pid, &mut completions);
            while let Some(complete) = completions.pop() {
                step::up(&mut self.io[driver], &env, complete, &mut events, &mut subs);
            }
            if self.io[driver].takes() {
                let event = events.pop().expect("file terminal");
                let fd = if let file::Event::Opened { file, .. } = &event {
                    self.io[driver].descriptor(*file)
                } else {
                    None
                };
                return Some(Arrived { event, fd });
            }
        }
        panic!("file did not settle");
    }
    fn bytes(&mut self, path: &[u8]) -> Box<[u8]> {
        let file = self.machine.open(self.handle, path, How::ReadNoFollow).expect("file stands");
        let bytes = self.machine.read(file, 0, u32::MAX).expect("file read");
        self.machine.close(file);
        bytes.into_boxed_slice()
    }
    fn items(&mut self) -> Vec<Item> {
        let mut items = Vec::new();
        self.walk(b"", &mut items);
        items
    }
    fn walk(&mut self, prefix: &[u8], items: &mut Vec<Item>) {
        let directory = self
            .machine
            .open(self.handle, if prefix.is_empty() { b"." } else { prefix }, How::Directory)
            .expect("directory");
        loop {
            let entries = self.machine.list(directory, 32, 8192).expect("list");
            if entries.is_empty() {
                break;
            }
            for (kind, name) in entries {
                let path = if prefix.is_empty() { name.to_vec() } else { [prefix, b"/", &name].concat() };
                if kind == Is::Directory {
                    items.push(Item::directory(&path).mode(0o700));
                    self.walk(&path, items);
                } else {
                    items.push(Item::file(&path, &self.bytes(&path)).mode(0o600));
                }
            }
        }
        self.machine.close(directory);
    }
    // Lay a file, or a segment and its second link, through the fake's Item
    // vocabulary. Move them beneath the live copy without changing its handles.
    fn lay_private(&mut self, laid: &[Item]) {
        use skein_fake_machine::Made;
        let mut items = Vec::new();
        for item in laid {
            if let Made::HardLink(target) = &item.made {
                let path = [b"generations/2/secrets/", target.as_slice()].concat();
                items.push(Item::file(target, &self.bytes(&path)).mode(0o600));
            }
        }
        items.extend_from_slice(laid);
        let staged = self.machine.lay(&items);
        let private = self
            .machine
            .open(self.handle, b"generations/2/secrets", How::DirectoryNoFollow)
            .expect("sealed copy's private directory");
        for item in &items {
            self.machine.rename(staged, &item.path, private, &item.path, false).expect("lay check entry");
        }
        self.machine.sync(private);
        self.machine.close(private);
        self.machine.close(staged);
    }
    fn source_items(&mut self, number: u64) -> Vec<Item> {
        let prefix = format!("generations/{number}/");
        self.items().into_iter().filter(|item| item.path.starts_with(prefix.as_bytes())).collect()
    }
    fn assert_source_unchanged(&mut self, number: u64) {
        assert_eq!(self.source_items(number), *self.source.as_ref().expect("captured before closing A"));
    }
    fn crash(self, seed: u64) -> Self {
        let mut machine = self.machine;
        machine.crash(seed);
        let handle = machine.reopen_root(0);
        Self::over(machine, handle)
    }
}
#[test]
fn the_phase_opens_pending_then_manifests_and_switches_without_reopening() {
    let mut first = Disk::new(&baseline());
    upgrade(&mut first, 1);
    let mut disk = Harness::new(&first.items());
    let mut store = disk.store(true);
    store.patch_sets(upgrade_sets(2));
    assert_opened(&disk.open(&mut store), 5);
    assert!(disk.changes.iter().all(|(stage, _)| *stage != Stage::Checking), "the check writes nothing");
    let stages: Vec<_> = disk.trace.iter().map(|(stage, _)| *stage).filter(|s| *s != Stage::Other).fold(
        Vec::new(),
        |mut stages, stage| {
            if stages.last() != Some(&stage) {
                stages.push(stage);
            }
            stages
        },
    );
    assert_eq!(
        stages,
        [
            Stage::Closing,
            Stage::Dropping,
            Stage::Copying,
            Stage::Opening,
            Stage::Jobs,
            Stage::Sealing,
            Stage::Checking,
            Stage::Manifest,
            Stage::Switching
        ]
    );
    {
        let stage = Stage::Closing;
        let closes: Vec<_> = disk.trace.iter().filter(|(s, _)| *s == stage).map(|(_, request)| request).collect();
        // Both KV log closes must finish before either map directory closes.
        assert_eq!(closes, [&Trace::KvClose(0), &Trace::KvClose(1), &Trace::MapClose(0), &Trace::MapClose(1)]);
    }
    let opens: Vec<_> = disk
        .trace
        .iter()
        .filter(|(s, _)| *s == Stage::Opening)
        .map(|(_, r)| r)
        .filter(|r| **r != Trace::Other)
        .collect();
    assert_eq!(opens.first(), Some(&&Trace::MapOpen(0, Box::from(b"generations/3/store".as_slice()))));
    assert_eq!(opens.get(1), Some(&&Trace::MapOpen(1, Box::from(b"generations/3/secrets".as_slice()))));
    assert!(opens.contains(&&Trace::KvOpen(0)) && opens.contains(&&Trace::KvOpen(1)));
    let switch = disk
        .trace
        .iter()
        .position(|(s, r)| *s == Stage::Switching && *r == Trace::Store(Box::from(b"active".as_slice())))
        .expect("switch");
    assert_eq!(switch + 1, disk.trace.len(), "the switch needs no reopen");
    assert_eq!(
        disk.trace
            .iter()
            .filter(|(_, r)| { if let Trace::Load(p) = r { p.as_ref() == b"active" } else { false } })
            .count(),
        1
    );
}
#[test]
fn cuts_at_the_manifests_write_syncs_and_rename_and_copy_recovery_keep_a_whole_generation() {
    let items = baseline();
    let mut completed = Harness::new(&items);
    let mut store = completed.store(true);
    assert_opened(&completed.open(&mut store), 3);
    let changes = completed.changes.clone();
    for change in [Change::Write, Change::Sync, Change::Rename] {
        assert!(changes.contains(&(Stage::Manifest, change)), "manifest {change:?}");
    }
    assert!(changes.contains(&(Stage::Opening, Change::Sync)), "copy recovery reaches the simulated disk");
    for (cut, target) in changes.iter().enumerate() {
        for after in [false, true] {
            let mut disk = Harness::new(&items);
            disk.cut = Some((cut, after));
            let mut store = disk.store(true);
            let events = disk.open(&mut store);
            assert_eq!(
                events.len(),
                usize::from(target.0 == Stage::Manifest).checked_mul(2).expect("opening and census"),
                "cut {cut} {target:?}"
            );
            assert!(events.iter().all(|event| {
                if let Event::Opened { pending: true, .. } | Event::Census { .. } = event { true } else { false }
            }));
            assert!(disk.halted, "cut reached");
            disk.assert_source_unchanged(1);
            let mut disk = disk.crash(u64::try_from(cut).expect("seed"));
            let mut store = disk.store(false);
            assert_opened(&disk.open(&mut store), 1);
            assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
            let mut recovered = Disk::new(&disk.items());
            let mut store = recovered.store_with_chunk(CHUNK);
            store.patch_sets(upgrade_sets(0));
            assert_opened(&recovered.open(&mut store), 1);
            read_row(&mut recovered, &mut store);
            assert!(!recovered.tree().iter().any(|(p, ..)| p.starts_with(b"generations/2")), "cut {cut} {target:?}");
        }
    }
}
#[test]
fn a_manifest_write_fault_refuses_failed_and_the_next_start_removes_the_copy() {
    let mut disk = Harness::new(&baseline());
    disk.fault_write = true;
    let mut store = disk.store(true);
    let events = disk.open(&mut store);
    assert!({
        if let [
            Event::Opened { pending: true, .. },
            Event::Census { .. },
            Event::OpenRefused { why: OpenRefusal::Failed, .. },
        ] = events.as_ref()
        {
            true
        } else {
            false
        }
    });
    assert_eq!(store.phase, Phase::Stopped);
    assert!(!disk.fault_write, "fault injected into the manifest's write");
    disk.assert_source_unchanged(1);
    assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
    let mut disk = disk.crash(19);
    let mut store = disk.store(false);
    assert_opened(&disk.open(&mut store), 1);
    let mut recovered = Disk::new(&disk.items());
    let mut store = recovered.store_with_chunk(CHUNK);
    store.patch_sets(upgrade_sets(0));
    assert_opened(&recovered.open(&mut store), 1);
    read_row(&mut recovered, &mut store);
    assert!(!recovered.tree().iter().any(|(p, ..)| p.starts_with(b"generations/2")));
}

#[test]
fn the_run_proofs_transcript_cut_precedes_the_pending_open_and_switch() {
    let mut items = baseline_with_proof(true);
    items.push(Item::directory(b"transcripts/0000000000000000"));
    items.push(Item::file(b"transcripts/0000000000000000/0000000000000001", b"uncommitted tail"));
    let mut disk = Harness::new(&items);
    let mut store = disk.store(true);
    assert_opened(&disk.open(&mut store), 3);
    let switch = disk.trace.iter().position(|(s, _)| *s == Stage::Switching).expect("switch");
    let cuts: Vec<_> = disk
        .trace
        .iter()
        .enumerate()
        .filter(|(_, (_, r))| *r == Trace::Cut)
        .map(|(index, (stage, _))| (index, *stage))
        .collect();
    assert_eq!(cuts.len(), 1);
    assert!(cuts.iter().all(|(index, stage)| *index < switch && *stage == Stage::Other));
    assert!(disk.bytes(b"transcripts/0000000000000000/0000000000000001").is_empty());
    disk.assert_source_unchanged(1);
}

#[test]
fn the_manifest_write_refuses_to_replace_a_standing_manifest() {
    let mut disk = Disk::new(&[
        Item::directory(b"generations"),
        Item::directory(b"generations/2"),
        Item::file(b"generations/2/manifest", b"standing"),
    ]);
    let mut below = fixtures::below();
    let mut writing = crate::generations::store_manifest(disk.root(), 2, Box::from(b"2\n".as_slice()), &mut below);
    let mut progress = crate::generations::Progress::Pending;
    disk.drive(&mut below, |io, _| {
        let Io::Transcripts(arrived) = io else {
            panic!("manifest uses driver 2");
        };
        progress = crate::generations::manifested(&mut writing, arrived);
    });
    assert_eq!(progress, crate::generations::Progress::Failed);
    assert_eq!(disk.bytes(b"generations/2/manifest").as_ref(), b"standing");
}

const FIXTURE_LAST: u64 = 3;
const SHELVES: u32 = 5;
fn fixture_key(prefix: u8, number: u32) -> Box<[u8]> {
    format!("{}/{number}", char::from(prefix)).into_bytes().into_boxed_slice()
}
fn fixture_baseline() -> Vec<Item> {
    let mut disk = Disk::new(&[]);
    let mut store = disk.store_with_chunk(CHUNK);
    store.patch_sets(BASE);
    store.application.version = 0;
    assert_opened(&disk.open(&mut store), 0);
    let mut rows = vec![
        Write::Put { key: fixture_key(b'l', 0), value: label() },
        Write::Put { key: fixture_key(b'l', 1), value: label() },
        // keys::application adds exactly tag 2, so this lands inside the
        // fixture-jig family's whole-map range [02 6e, 02 6f).
        Write::Put { key: Box::from(b"n/0".as_slice()), value: label() },
    ];
    let note_key = crate::keys::application(b"n/0", store.limits.store.key).expect("tag");
    assert!(note_key.as_ref() >= jig_test_patches_jig::keys::NOTES_FROM);
    assert!(note_key.as_ref() < jig_test_patches_jig::keys::NOTES_TO);
    for number in 0..SHELVES {
        let books = number.checked_add(1).expect("books");
        rows.push(Write::Put { key: fixture_key(b's', number), value: shelf(books, &label()) });
    }
    // Three rows and the header fit the fixture's four-write journal.
    let mut rows = rows.into_iter();
    for number in 1..=FIXTURE_LAST {
        let mut writes: Vec<_> = rows.by_ref().take(3).collect();
        writes.push(fixtures::header(number));
        if number == FIXTURE_LAST {
            // The last fixture page holds two rows, its header, and the start mark.
            writes.push(Write::Wires);
        }
        let events = disk.request(&mut store, fixtures::commit(number, writes.into_boxed_slice()));
        assert!(
            match events.as_ref() {
                [Event::Committed { number: found, .. }] => *found == number,
                _ => false,
            },
            "{events:?}"
        );
    }
    assert_eq!(store.stores.store.last(), FIXTURE_LAST);
    disk.items()
}

fn assert_sealed(store: &mut Store, last: u64) {
    let header_key = crate::keys::encode(
        &jig_domain_core_vocabulary::Key::Core(jig_domain_core_vocabulary::CoreKey::Deployment),
        store.limits.store.key,
    )
    .expect("header key");
    let header = fixtures::get_kv(store, &header_key).expect("sealed header");
    let expected = fixtures::header_put(fixtures::DEPLOYMENT, last);
    let skein_kv::Op::Put { value, .. } = expected else { panic!("header put") };
    assert_eq!(header, value);
    let key = crate::keys::encode_own(&crate::keys::Own::Format, store.limits.store.key).expect("format key");
    let mut expected = crate::format::current(store.application.version, fixtures::DEPLOYMENT);
    expected.levels = crate::patching::plan::binary(&store.patch_sets);
    assert_eq!(fixtures::get_kv(store, &key).expect("format"), expected.encode().expect("format bytes"));
    let key = crate::keys::encode_own(&crate::keys::Own::Layouts, store.limits.store.key).expect("layouts key");
    assert_eq!(fixtures::get_kv(store, &key), Some(store.layouts.encode()));
}

fn assert_refused(events: &[Event], store: &Store) {
    assert!(
        match events.last() {
            Some(Event::OpenRefused { owner, why: OpenRefusal::Failed }) => *owner == Token::new(1),
            _ => false,
        },
        "{events:?}"
    );
    assert_eq!(store.phase, Phase::Stopped);
}

fn assert_manifest(
    bytes: &[u8],
    generation: u64,
    before: crate::patching::plan::Levels,
    after: crate::patching::plan::Levels,
    last: u64,
) {
    assert_eq!(
        crate::codec::component::Manifest::decode(bytes),
        Ok(crate::codec::component::Manifest {
            generation,
            from: generation.checked_sub(1),
            before,
            after,
            counts: fixtures::manifest_counts(&[
                ("fixture-label", 0, 0),
                ("fixture-shelf", 0, 0),
                ("fixture-token", 0, 0)
            ]),
            store: last,
            secrets: Some(0),
            written: 0,
        })
    );
}

fn assert_check(events: &[Event], store: &Store, why: crate::Check) {
    let Some(Event::OpenRefused { owner, why: refusal }) = events.last() else { panic!("check refusal: {events:?}") };
    assert_eq!(*owner, Token::new(1));
    assert_eq!(*refusal, OpenRefusal::Check { why });
    assert_eq!(store.phase, Phase::Stopped);
}

fn fixture_with_record(record: jig_domain_core_vocabulary::Record) -> Vec<Item> {
    let mut disk = Disk::new(&baseline());
    let mut store = disk.store_with_chunk(CHUNK);
    store.patch_sets(upgrade_sets(0));
    disk.open(&mut store);
    let last = 2;
    let events =
        disk.request(&mut store, fixtures::commit(last, Box::new([fixtures::header(last), Write::Save(record)])));
    let [Event::Committed { .. }] = events.as_ref() else { panic!("base record committed: {events:?}") };
    disk.items()
}

#[test]
fn a_restart_row_past_the_deployments_lowered_limit_refuses_load() {
    let record = fixtures::live_task(fixtures::LIVE);
    let key = crate::keys::encode(&record.key(), fixtures::KEY_LIMIT).expect("live row key");
    let mut disk = Harness::new(&fixture_with_record(record));
    let mut store = disk.store(true);
    // The fixture task has a dependency, valid under the base's two-entry limit.
    store.limits.core.tasks.dependencies = 0;
    store.codec_limits = crate::codec::limits(&store.limits.core).expect("lower deployment limits");
    let events = disk.open(&mut store);
    assert_check(&events, &store, crate::Check::Load { key });
    assert_abandoned_at(disk, 2);
}

#[test]
fn a_secret_file_with_group_bits_or_another_link_refuses_private() {
    let (mut seeded, mut original) = source_with_sets(false, upgrade_sets(0));
    let events = seeded
        .request(&mut original, Request::Secrets { owner: Token::new(2), request: fixtures::sign_in_secret(1, 7) });
    let [Event::Kept { .. }] = events.as_ref() else { panic!("{events:?}") };
    let before = seeded.items();
    let prefix = b"generations/1/secrets/";
    let segment =
        before.iter().find_map(|item| item.path.strip_prefix(prefix.as_slice())).expect("source's secrets segment");
    // A refusal beyond a full listing page proves the scan does not stop at PAGE.
    let mut paged: Vec<_> = (0..crate::generations::PAGE)
        .map(|n| Item::file(format!("private-{n:02}").as_bytes(), b"page entry").mode(0o600))
        .collect();
    paged.push(Item::file(b"z-group", b"private check").mode(0o640));
    for (items, name) in [
        (vec![Item::file(b"!group", b"private check").mode(0o640)], b"!group".as_slice()),
        (vec![Item::hard_link(b"!linked", segment)], b"!linked".as_slice()),
        (paged, b"z-group".as_slice()),
    ] {
        let name = Box::from(name);
        let mut disk = Harness::new(&before);
        disk.check_entries = items;
        let mut store = disk.store(true);
        let events = disk.open(&mut store);
        assert_check(&events, &store, crate::Check::Private { name });
        assert_eq!(disk.check_entries.len(), 0);
        assert_abandoned_at(disk, 1);
    }
}

#[test]
fn a_file_fault_during_the_check_refuses_failed_before_the_switch() {
    let mut disk = Harness::new(&baseline());
    disk.fault_at = Some(Stage::Checking);
    let mut store = disk.store(true);
    let events = disk.open(&mut store);
    assert_refused(&events, &store);
    assert_eq!(disk.fault_at, None);
    assert_abandoned_at(disk, 1);
}

#[test]
fn a_restart_row_at_the_deployments_lowered_limit_loads() {
    let record = fixtures::live_task(fixtures::LIVE);
    let mut disk = Harness::new(&fixture_with_record(record));
    let mut store = disk.store(true);
    store.limits.core.tasks.dependencies = 1;
    store.codec_limits = crate::codec::limits(&store.limits.core).expect("lower deployment limits");
    let events = disk.open(&mut store);
    let [.., Event::Opened { .. }, Event::Census { .. }] = events.as_ref() else {
        panic!("row within its bound loads: {events:?}")
    };
    assert_eq!(disk.bytes(b"active").as_ref(), b"2\n");
}

fn jig_store(disk: &Disk, set: crate::PatchSet) -> Store {
    let mut limits = fixtures::store_limits();
    limits.store.budget = 2_097_152;
    limits.store.ops = 128;
    limits.commits.writes = 7;
    let mut store = disk.store_with_limits(&limits);
    store.patch_sets(crate::PatchSets {
        sets: Box::leak(Box::new([crate::SKEIN_PATCH_SET, crate::SMITH_PATCH_SET, set])),
    });
    store
}

const UPGRADE_JOB: crate::Job = crate::Job {
    name: "upgrade-fixture",
    keys: &[crate::KeyRange { from: b"l", to: b"m" }],
    converts: None,
    settles: false,
    counts: &[],
};
fn upgrade_sets(generation: usize) -> crate::PatchSets {
    let jobs: &'static [crate::Job] = match generation {
        0 => &[],
        1 => &[UPGRADE_JOB],
        2 => &[UPGRADE_JOB, crate::Job { name: "next-upgrade", ..UPGRADE_JOB }],
        _ => panic!("fixture generation"),
    };
    crate::PatchSets { sets: Box::leak(Box::new([crate::PatchSet { jobs, ..BASE.sets[0] }])) }
}
fn assert_abandoned_at(mut disk: Harness, last: u64) {
    disk.assert_source_unchanged(1);
    assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
    let mut disk = disk.crash(31);
    let mut store = disk.store(false);
    assert_opened(&disk.open(&mut store), last);
    assert!(!disk.items().iter().any(|item| item.path.starts_with(b"generations/2")));
}

fn finish_empty_jobs(disk: &mut Disk, store: &mut Store, first: Box<[Event]>) -> Vec<Event> {
    let mut queue = alloc::collections::VecDeque::from(first.into_vec());
    let mut events = Vec::new();
    while let Some(event) = queue.pop_front() {
        if let Event::Job { owner, page, .. } = event {
            assert!(page.end && page.rows.is_empty());
            queue.extend(
                disk.request(
                    store,
                    Request::Job {
                        owner,
                        answer: Ok(crate::JobAnswer { writes: Box::new([]), through: None, counts: Box::new([]) }),
                    },
                )
                .into_vec(),
            );
        } else if if let Event::Ran { .. } = event { false } else { true } {
            events.push(event);
        }
    }
    events
}

#[test]
fn a_job_upgrade_can_go_back_only_before_its_first_serving_commit() {
    for committed in [false, true] {
        let mut disk = Disk::new(&baseline());
        let mut store = upgrade(&mut disk, 1);
        if committed {
            let events = disk.request(&mut store, fixtures::commit(4, Box::new([fixtures::header(4)])));
            let [Event::Committed { number: 4, .. }] = events.as_ref() else { panic!("{events:?}") };
        }
        let mut disk = Disk::new(&disk.items());
        let mut store = disk.store_with_chunk(CHUNK);
        let root = disk.root();
        let events = disk.request(&mut store, Request::Back { owner: Token::new(1), root });
        if committed {
            let [Event::BackRefused { why: crate::BackRefusal::Committed { manifest: 3, last: 4 }, .. }] =
                events.as_ref()
            else {
                panic!("{events:?}")
            };
            assert_eq!(disk.bytes(b"active").as_ref(), b"2\n");
        } else {
            let [Event::Back { to: 1, .. }] = events.as_ref() else { panic!("{events:?}") };
            assert_eq!(disk.bytes(b"active").as_ref(), b"1\n");
            let mut disk = Disk::new(&disk.items());
            let mut store = disk.store_with_chunk(CHUNK);
            store.patch_sets(upgrade_sets(0));
            assert_opened(&disk.open(&mut store), 1);
            read_row(&mut disk, &mut store);
            assert!(!disk.tree().iter().any(|(path, ..)| path.starts_with(b"generations/2")));
        }
    }
}

#[test]
fn a_job_manifest_records_the_copys_secret_commit_and_write_clock() {
    const WRITTEN: u64 = 73;
    let (mut before, mut store) = source_with_sets(false, upgrade_sets(0));
    let events =
        before.request(&mut store, Request::Secrets { owner: Token::new(2), request: fixtures::sign_in_secret(1, 7) });
    let [Event::Kept { .. }] = events.as_ref() else { panic!("{events:?}") };
    let mut disk = Harness::new(&before.items());
    disk.wall = skein_lib::Wall::from_nanos(WRITTEN);
    let mut store = disk.store(true);
    assert_opened(&disk.open(&mut store), 3);
    let manifest = crate::codec::component::Manifest::decode(&disk.bytes(b"generations/2/manifest")).expect("manifest");
    assert_eq!((manifest.store, manifest.secrets, manifest.written), (3, Some(1), WRITTEN));
}

fn upgraded_jobs() -> (Disk, Store, Vec<Event>) {
    let mut disk = Disk::new(&fixture_baseline());
    let mut store = jobs::store(&disk, jobs::JOBS);
    let opened = disk.open(&mut store);
    let events = jobs::drive(&mut disk, &mut store, opened);
    (disk, store, events)
}

#[test]
fn the_manifest_counts_each_kept_family_before_and_after() {
    let (mut disk, mut store, _) = upgraded_jobs();
    let events = disk.request(&mut store, Request::Switch { owner: Token::new(1) });
    let [Event::Switched { .. }] = events.as_ref() else { panic!("{events:?}") };
    let manifest = crate::codec::component::Manifest::decode(&disk.bytes(b"generations/2/manifest")).expect("manifest");
    assert_eq!(
        manifest.counts,
        fixtures::manifest_counts(&[
            ("fixture-label", 2, 1),
            ("fixture-shelf", 5, 0),
            ("fixture-token", 0, 0),
            ("fixture-note", 1, 0),
        ])
    );
}

#[test]
fn the_job_upgraded_copys_open_reports_its_census() {
    let (_, store, events) = upgraded_jobs();
    let [.., Event::Opened { last, pending: true, .. }, Event::Census { census }] = events.as_slice() else {
        panic!("{events:?}")
    };
    assert_eq!(*last, FIXTURE_LAST.checked_add(13).expect("job commits"));
    assert_eq!(census.commit, *last);
    assert_eq!(census.since, store.stores.store.census().since());
    assert!(!census.keyspaces.is_empty());
    assert_eq!(census.keyspaces.len(), u32::try_from(store.stores.store.census().keyspaces().len()).unwrap());
    for (row, counted) in census.keyspaces.iter().zip(store.stores.store.census().keyspaces()) {
        assert_eq!(&*row.prefix, counted.prefix());
        assert_eq!((row.rows, row.puts, row.erases), (counted.rows, counted.puts, counted.erases));
    }
    assert!(census.keyspaces.iter().any(|row| *row.prefix == [2, b's'] && row.erases > 0), "job erasures counted");
    assert_eq!(store.stores.secrets.census().keyspaces(), []);
}
