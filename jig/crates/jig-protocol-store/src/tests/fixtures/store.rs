//! Memory component fixtures (protocol/store/testing.md, section 1).
use super::{
    APPLICATION, CORE_LIMITS, ENDED, INSTALLATION, KEY_LIMIT, LIVE, cold_task, deployment_record, header_record,
    kv_limits,
};
use crate::{Arrived, Below, Io, Root, SecretOwner, Secrets};
use crate::{Event, Limits, Request, Store, Stores, Write, codec, format, keys};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{CoreKey, Key, Record};
use skein_io::file_layer::{FileIo, Limits as FileLimits};
use skein_io::{file, kernel::Fd};
use skein_lib::{Env, List, Queue, Token};

pub const ROOT: Root = Root { token: Token::new(900), fd: Fd::new(3) };

pub fn limits() -> Limits {
    Limits {
        core: CORE_LIMITS,
        store: kv_limits(),
        secrets: skein_kv::Limits { record: 1, ..kv_limits() },
        commits: crate::sizes::Commits { in_flight: 2, writes: 4 },
        joins: 32,
        queue: 2,
        files: 2,
        reads: 4,
        fetch: 16_777_216,
        checkpoints: crate::checkpoints::Checkpoints { keep: 2 },
        orphan_erases: 1,
        held: 2,
    }
}

pub fn stores() -> Stores {
    stores_with(&limits())
}

fn stores_with(limits: &Limits) -> Stores {
    Stores { store: skein_kv::Store::memory(&limits.store), secrets: skein_kv::Store::memory(&limits.secrets) }
}

pub fn plant(stores: &mut Stores, ops: &[skein_kv::Op]) {
    let mut planted = List::uncapped();
    let mut format = false;
    let mut floor = false;
    for op in ops {
        match op {
            skein_kv::Op::Put { key, .. } => {
                format |= keys::decode_own(key) == Some(keys::Own::Format);
                floor |= keys::decode_own(key) == Some(keys::Own::Floor);
            }
            skein_kv::Op::Erase { key } => floor |= keys::decode_own(key) == Some(keys::Own::Floor),
            skein_kv::Op::Append { .. } | skein_kv::Op::Drop { .. } => {}
        }
        planted.push(op.clone()).expect("uncapped fixture");
    }
    if stores.store.last() == 0 && format && !floor {
        planted.push(floor_put()).expect("uncapped fixture");
    }
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    skein_kv::down(
        &mut stores.store,
        &crate::memory_env(kv_limits()),
        skein_kv::Request::Commit { owner: Token::new(0), ops: planted.into_boxed() },
        &mut above,
        &mut below,
    );
    assert!(below.is_empty(), "memory commits issue no files");
    let Some(skein_kv::Event::Committed { .. }) = above.pop() else {
        panic!("fixture commit must fit");
    };
    assert!(above.is_empty());
}

pub fn format_put(deployment: [u8; 16]) -> skein_kv::Op {
    skein_kv::Op::Put {
        key: keys::encode_own(&keys::Own::Format, KEY_LIMIT).expect("format key fits"),
        value: format::current(APPLICATION.version, deployment).encode().expect("format fits"),
    }
}

pub fn floor_put() -> skein_kv::Op {
    skein_kv::Op::Put {
        key: keys::encode_own(&keys::Own::Floor, KEY_LIMIT).expect("floor key fits"),
        value: crate::rows::floor(""),
    }
}

pub fn header_put(deployment: [u8; 16], commits: u64) -> skein_kv::Op {
    let record = deployment_record(deployment, commits);
    skein_kv::Op::Put {
        key: keys::encode(&Key::Core(CoreKey::Deployment), KEY_LIMIT).expect("header key fits"),
        value: codec::encode(&record, &codec::limits(&CORE_LIMITS).expect("fixture limits"), &[]).expect("header fits"),
    }
}

pub fn installation_put(id: [u8; 16]) -> skein_kv::Op {
    skein_kv::Op::Put {
        key: keys::installation(KEY_LIMIT).expect("installation key fits"),
        value: codec::accounts::Installation { id }.encode(),
    }
}

pub fn over(stores: Stores) -> Store {
    Store::over(&crate::check(&limits()).expect("fixture store limits"), APPLICATION, &[], stores)
}

pub fn reopen(store: Store) -> Store {
    Store::over(&store.checked(), APPLICATION, &[], store.into_stores())
}

pub fn run(store: &mut Store, request: Request) -> Box<[Event]> {
    let mut above = Queue::uncapped();
    let mut below = below();
    crate::down(store, &env(), request, &mut above, &mut below);
    let mut events = List::with_capacity(above.len());
    for _ in 0..above.len() {
        if let Some(event) = above.pop() {
            events.push(event).expect("one terminal");
        }
    }
    events.into_boxed()
}

pub fn opened() -> Store {
    opened_with_budget(16_384)
}

pub fn opened_with_budget(budget: u64) -> Store {
    let mut limits = limits();
    limits.store.budget = budget;
    let mut store =
        Store::over(&crate::check(&limits).expect("fixture store limits"), APPLICATION, &[], stores_with(&limits));
    let events = run(
        &mut store,
        Request::Open { purpose: crate::Purpose::Serve, owner: Token::new(1), root: ROOT, installation: INSTALLATION },
    );
    let [Event::Opened { owner: _, last: 0, installation: _, pending: false }, Event::Census { .. }] = &*events else {
        panic!("empty store opens: {events:?}");
    };
    store
}

pub fn commit(number: u64, writes: Box<[Write]>) -> Request {
    Request::Commit { number, writes: whole_tasks(writes) }
}

pub fn header(commits: u64) -> Write {
    save(header_record(commits))
}

pub fn save(record: Record) -> Write {
    Write::Save(record)
}

pub fn get_kv(store: &mut Store, key: &[u8]) -> Option<Box<[u8]>> {
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    skein_kv::down(
        &mut store.stores.store,
        &crate::memory_env(store.limits.store),
        skein_kv::Request::Get { owner: Token::new(0), key: Box::from(key) },
        &mut above,
        &mut below,
    );
    assert!(below.is_empty(), "memory gets issue no files");
    let Some(skein_kv::Event::Got { value, .. }) = above.pop() else {
        panic!("memory get answers immediately");
    };
    assert!(above.is_empty());
    value
}

pub fn committed(store: &mut Store, writes: Box<[Write]>) {
    let number = store.stores.store.last().checked_add(1).expect("fixture next commit");
    let mut extra = 0_u32;
    for write in &writes {
        if let Write::Save(Record::Tasks(jig_domain_tasks::Stored::Live(_))) = write {
            extra = extra.checked_add(1).unwrap();
        }
    }
    let allowance = store.limits.commits.writes;
    store.limits.commits.writes = allowance.checked_add(extra).unwrap();
    let events = run(store, commit(number, writes));
    store.limits.commits.writes = allowance;
    let [Event::Committed { number: found, .. }] = &*events else {
        panic!("fixture commit fits: {events:?}");
    };
    assert_eq!(*found, number);
}

/// Fixture callers save whole tasks; production callers name each part explicitly.
fn whole_tasks(writes: Box<[Write]>) -> Box<[Write]> {
    let mut complete = List::uncapped();
    for write in writes {
        if let Write::Save(Record::Tasks(jig_domain_tasks::Stored::Live(row))) = &write {
            complete.push(save(cold_task(row))).expect("fixture writes");
        }
        complete.push(write).expect("fixture writes");
    }
    complete.into_boxed()
}

pub fn env() -> Env<Limits> {
    Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: limits() }
}
pub fn below() -> Below {
    Below {
        store: Queue::uncapped(),
        secrets: Queue::uncapped(),
        transcripts: Queue::uncapped(),
        checkpoints: Queue::uncapped(),
    }
}
fn unseeded_driver() -> FileIo {
    FileIo::with_limits(&FileLimits { read: 1, whole: 1, ..super::disk::file_limits() })
}
pub fn directory() -> (Store, Below) {
    let mut drivers = [unseeded_driver(), unseeded_driver(), unseeded_driver(), unseeded_driver()];
    let [store, secrets, transcripts, checkpoints] = &mut drivers;
    // Scripted terminals need no randomness; reuse the fixture's installation bytes as their seed.
    let seeded = crate::seed(
        [store, secrets, transcripts, checkpoints],
        u64::from_le_bytes(INSTALLATION.get(..8).expect("installation seed").try_into().expect("eight bytes")),
    );
    let mut store =
        Store::on_directory(&crate::check(&limits()).expect("fixture store limits"), APPLICATION, &[], seeded);
    let mut below = below();
    let mut above = Queue::uncapped();
    crate::down(
        &mut store,
        &env(),
        Request::Open { purpose: crate::Purpose::Serve, owner: Token::new(1), root: ROOT, installation: INSTALLATION },
        &mut above,
        &mut below,
    );
    assert!(above.is_empty());
    (store, below)
}
pub fn answer(store: &mut Store, io: Io) -> (Box<[Event]>, Below) {
    let mut above = Queue::uncapped();
    let mut below = below();
    crate::up(store, &env(), io, &mut above, &mut below);
    (events(&mut above), below)
}
pub fn events(above: &mut Queue<Event>) -> Box<[Event]> {
    let mut events = List::with_capacity(above.len());
    for _ in 0..above.len() {
        if let Some(event) = above.pop() {
            events.push(event).expect("bounded events");
        }
    }
    events.into_boxed()
}
pub fn sign_in_secret(number: u64, byte: u8) -> Secrets {
    Secrets::Keep {
        owner: SecretOwner::SignIn,
        number,
        value: crate::SecretValue::Digest { expiry: 0, digest: [byte; 32] },
    }
}
pub fn opened_with_transcripts() -> (Store, Below) {
    let mut store = opened();
    committed_with_places(&mut store, Box::new([header(1)]), &[LIVE, ENDED, 23]);
    (store, below())
}

pub fn committed_with_places(store: &mut Store, writes: Box<[Write]>, tasks: &[u64]) {
    // These fixtures exercise files belonging to tasks with transcript places.
    let number = store.stores.store.last().checked_add(1).expect("fixture commit");
    let crate::Phase::Ready { format_due, .. } = store.phase else { panic!("ready fixture") };
    let mut pending = crate::commits::operations(store, number, writes, format_due).expect("fixture operations");
    let mut ops = List::with_capacity(32);
    for op in pending.ops {
        ops.push(op).expect("first operations");
    }
    for task in tasks {
        ops.push(skein_kv::Op::Put {
            key: keys::encode_own(&keys::Own::Transcript { task: *task }, KEY_LIMIT).expect("old_place key"),
            value: codec::component::Transcript { length: 0, window: 0 }.encode().expect("old_place"),
        })
        .expect("old_place slot");
    }
    pending.ops = ops.into_boxed();
    store.phase = crate::Phase::Ready { next: number.checked_add(1).unwrap(), format_due: false };
    store.transcripts.current = Some(pending);
    let mut above = Queue::uncapped();
    let mut below = below();
    crate::transcripts::write::fire(store, &env(), &mut above, &mut below);
    let Some(Event::Committed { number: found, .. }) = above.pop() else { panic!("fixture commit") };
    assert_eq!(found, number);
    assert!(above.is_empty());
}
#[derive(Clone)]
pub enum Answer {
    Opened { len: u64 },
    Missing,
    Made,
    Written,
    Synced,
    Closed,
    Read { bytes: Box<[u8]> },
    Cut,
    Removed,
    NotFound,
}
pub fn script(store: &mut Store, mut below: Below, answers: &[Answer]) -> Box<[Event]> {
    // A scripted file answer can produce a batch of store terminals.
    const EVENTS_PER_ANSWER: u32 = 128;
    let count =
        u32::try_from(answers.len()).expect("bounded script").checked_mul(EVENTS_PER_ANSWER).expect("bounded events");
    let mut found = List::with_capacity(count);
    let base =
        store.transcripts.touch.checked_mul(128).expect("bounded script").checked_add(1_000).expect("fixture token");
    for (index, answer) in answers.iter().enumerate() {
        let request = below.transcripts.pop().expect("transcript request");
        assert!(below.transcripts.is_empty());
        let event = match answer {
            Answer::Opened { len } => {
                let (file::Request::OpenWriteNoFollow { owner, .. }
                | file::Request::CreateNoFollow { owner, .. }
                | file::Request::OpenDirectory { owner, .. }
                | file::Request::OpenRead { owner, .. }) = request
                else {
                    panic!("open operation");
                };
                let token = base.checked_add(u64::try_from(index).expect("script index")).expect("fixture token");
                file::Event::Opened { owner, file: Token::new(token), len: *len }
            }
            Answer::Missing => {
                let (file::Request::OpenWriteNoFollow { owner, .. } | file::Request::OpenRead { owner, .. }) = request
                else {
                    panic!("missing open");
                };
                file::Event::Failed { owner, error: skein_io::kernel::Error::NotFound, committed: false, residue: None }
            }
            Answer::Made => {
                let file::Request::MakeDirectory { owner, .. } = request else {
                    panic!("make directory");
                };
                file::Event::Made { owner }
            }
            Answer::Written => {
                let file::Request::WriteAt { owner, .. } = request else {
                    panic!("write frame");
                };
                file::Event::Written { owner }
            }
            Answer::Synced => {
                let (file::Request::Sync { owner, .. } | file::Request::SyncDirectory { owner, .. }) = request else {
                    panic!("sync operation");
                };
                file::Event::Synced { owner }
            }
            Answer::Read { bytes } => {
                let file::Request::ReadAt { owner, max, .. } = request else {
                    panic!("read frame");
                };
                assert!(bytes.len() <= skein_lib::length::usize_of(max));
                file::Event::Read { owner, bytes: bytes.clone() }
            }
            Answer::Removed => {
                let file::Request::Remove { owner, .. } = request else {
                    panic!("remove operation");
                };
                file::Event::Removed { owner }
            }
            Answer::NotFound => {
                let file::Request::Remove { owner, .. } = request else {
                    panic!("missing removal");
                };
                file::Event::Failed { owner, error: skein_io::kernel::Error::NotFound, committed: false, residue: None }
            }
            Answer::Cut => {
                let file::Request::Cut { owner, .. } = request else {
                    panic!("cut operation");
                };
                file::Event::Cut { owner }
            }
            Answer::Closed => {
                let file::Request::Close { owner, .. } = request else {
                    panic!("close operation");
                };
                file::Event::Closed { owner }
            }
        };
        let (events, next) = self::answer(store, Io::Transcripts(Arrived { event, fd: Some(Fd::new(10)) }));
        for event in events {
            found.push(event).expect("bounded script events");
        }
        below = next;
    }
    found.into_boxed()
}

pub fn written(store: &mut Store, mut below: Below, turns: &[jig_domain_core_vocabulary::TurnRecord]) -> Box<[u8]> {
    let mut disk = List::with_capacity(4096);
    for _ in 0..turns.len().checked_mul(3).expect("bounded answers").checked_add(1).expect("opening") {
        let Some(request) = below.transcripts.pop() else { break };
        let event = match request {
            file::Request::OpenWriteNoFollow { owner, .. } => {
                file::Event::Opened { owner, file: Token::new(700), len: 0 }
            }
            file::Request::WriteAt { owner, bytes, .. } => {
                for byte in bytes {
                    disk.push(byte).expect("fixture disk");
                }
                file::Event::Written { owner }
            }
            file::Request::Sync { owner, .. } => file::Event::Synced { owner },
            file::Request::MakeDirectory { .. }
            | file::Request::OpenPrivate { .. }
            | file::Request::Create { .. }
            | file::Request::CreateNoFollow { .. }
            | file::Request::OpenRead { .. }
            | file::Request::OpenReadNoFollow { .. }
            | file::Request::OpenWrite { .. }
            | file::Request::OpenDirectory { .. }
            | file::Request::Load { .. }
            | file::Request::Scan { .. }
            | file::Request::Store { .. }
            | file::Request::Stat { .. }
            | file::Request::ReadAt { .. }
            | file::Request::Cut { .. }
            | file::Request::Close { .. }
            | file::Request::SyncDirectory { .. }
            | file::Request::HardLink { .. }
            | file::Request::Rename { .. }
            | file::Request::Remove { .. }
            | file::Request::RemoveIn { .. }
            | file::Request::ListPage { .. }
            | file::Request::Lock { .. }
            | file::Request::List { .. } => panic!("unexpected transcript write"),
        };
        let (events, next) = answer(store, Io::Transcripts(Arrived { event, fd: None }));
        if !events.is_empty() {
            let [Event::Committed { .. }] = &*events else {
                panic!("fixture turn commit");
            };
        }
        below = next;
    }
    assert!(below.transcripts.is_empty());
    disk.into_boxed()
}
