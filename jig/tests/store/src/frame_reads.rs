//! Public transcript decode against fixture bytes, with deterministic file replies.
//! Shared by the fuzzy model and the isolated heap test; step tests exercise real file drivers.
use crate::root;
use jig_protocol_store::{self as store, Event, OwnerSets, Request, Root, TranscriptRead, codec, keys};
use jig_test_layouts::frames;
use skein_io::{file, kernel::Fd};
use skein_lib::{Env, Queue, Time, Token, Wall, length::len_u64};

const OWNER: Token = Token::new(9);
const COMMITTED: u64 = 1024;
pub use crate::patches::FRAME_SETS as SETS;
#[must_use]
pub fn floor() -> Box<[u8]> {
    use jig_protocol_store::core_v6::Record as _;
    let row = store::component_v3::Floor { number: store::FLOOR, build: Box::new([]) };
    row.encode(&jig_protocol_store::component_v3::CEILINGS).expect("measured floor")
}
fn env(limits: &store::Limits) -> Env<store::Limits> {
    Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits }
}
#[must_use]
pub fn closed(sets: OwnerSets, limits: &store::Limits) -> (store::Store, Env<store::Limits>) {
    let maps = store::Stores {
        store: skein_kv::Store::memory(&limits.store),
        secrets: skein_kv::Store::memory(&limits.secrets),
    };
    let checked = store::check(limits).expect("frame fixture limits");
    let mut component = store::Store::over(&checked, root::APPLICATION, &[], maps);
    component.owner_sets(sets);
    let env = env(limits);
    open(&mut component, &env);
    let mut above = Queue::uncapped();
    let mut below = store::Below {
        store: Queue::uncapped(),
        secrets: Queue::uncapped(),
        transcripts: Queue::uncapped(),
        checkpoints: Queue::uncapped(),
    };
    store::down(
        &mut component,
        &env,
        Request::Commit { number: 1, writes: Box::new([store::Write::Save(root::header(1))]) },
        &mut above,
        &mut below,
    );
    assert!(matches!(above.pop(), Some(Event::Committed { number: 1, .. })));
    let mut maps = component.into_stores();
    let codec = codec::limits(&limits.core).expect("codec");
    let own = |key| keys::encode_own(&key, limits.store.key).expect("component key");
    let ops = Box::new([
        skein_kv::Op::Put {
            key: keys::encode(&root::header(2).key(), limits.store.key).expect("header key"),
            value: codec::encode(&root::header(2), &codec, &[]).expect("header"),
        },
        skein_kv::Op::Put {
            key: own(keys::Own::Transcript { task: frames::TASK }),
            value: codec::component::Transcript { length: COMMITTED, window: 0 }.encode().expect("places"),
        },
        skein_kv::Op::Put {
            key: own(keys::Own::Attempt { task: frames::TASK, attempt: frames::ATTEMPT }),
            value: codec::component::Attempt { first: 0 }.encode().expect("attempt"),
        },
    ]);
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    skein_kv::down(
        &mut maps.store,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.store },
        skein_kv::Request::Commit { owner: OWNER, ops },
        &mut above,
        &mut below,
    );
    let mut component = store::Store::over(&checked, root::APPLICATION, &[], maps);
    component.owner_sets(sets);
    (component, env)
}
pub fn open(component: &mut store::Store, env: &Env<store::Limits>) {
    let mut above = Queue::uncapped();
    let mut below = store::Below {
        store: Queue::uncapped(),
        secrets: Queue::uncapped(),
        transcripts: Queue::uncapped(),
        checkpoints: Queue::uncapped(),
    };
    store::down(
        component,
        env,
        Request::Open {
            purpose: jig_protocol_store::Purpose::Serve,
            owner: OWNER,
            root: Root { token: Token::new(1), fd: Fd::new(3) },
            installation: root::DEPLOYMENT,
        },
        &mut above,
        &mut below,
    );
    assert!(matches!(above.pop(), Some(Event::Opened { pending: false, .. })));
    // These transcript checks do not consume the map census (protocol/store/growth.md, 1.3).
    let Some(Event::Census { .. }) = above.pop() else { panic!("opened census") };
}
pub use crate::patches::fixture_turn as turn;
// At most six file operations cover one small fixture frame.
const STEPS: u32 = 8;

pub fn decode(component: &mut store::Store, env: &Env<store::Limits>, raw: &[u8]) -> Option<Box<[u8]>> {
    let mut above = Queue::uncapped();
    let mut below = store::Below {
        store: Queue::uncapped(),
        secrets: Queue::uncapped(),
        transcripts: Queue::uncapped(),
        checkpoints: Queue::uncapped(),
    };
    store::down(
        component,
        env,
        Request::Transcript {
            owner: OWNER,
            read: TranscriptRead::Turn { task: frames::TASK, attempt: frames::ATTEMPT, turn: frames::TURN },
            rows: 1,
            bytes: COMMITTED,
        },
        &mut above,
        &mut below,
    );
    for _ in 0..STEPS {
        if let Some(event) = above.pop() {
            match event {
                // These transcript checks do not consume the census (protocol/store/growth.md, 1.3).
                Event::Census { .. } => continue,
                Event::Page { .. } | Event::PageRefused { .. } => {
                    unreachable!("a start's pages need a start-mode store")
                }
                Event::Turns { turns, .. } => return Some(turns.first().expect("selected turn").transcript.clone()),
                Event::Unread { .. } => return None,
                other @ (Event::Checkpointed { .. }
                | Event::CheckpointRefused { .. }
                | Event::Restored { .. }
                | Event::RestoreRefused { .. }
                | Event::CheckpointHeld { .. }
                | Event::CheckpointRemoved { .. }
                | Event::Switched { .. }
                | Event::Opened { .. }
                | Event::OpenRefused { .. }
                | Event::Committed { .. }
                | Event::Failed { .. }
                | Event::FrameTooLarge { .. }
                | Event::Loaded { .. }
                | Event::Read { .. }
                | Event::Got { .. }
                | Event::Kept { .. }
                | Event::Carried { .. }
                | Event::Stale { .. }
                | Event::Erased { .. }
                | Event::Secret { .. }
                | Event::SecretsLoaded { .. }
                | Event::SecretsNotReady { .. }
                | Event::WindowFailed { .. }
                | Event::Corrupt { .. }
                | Event::Decoded { .. }
                | Event::Undecodable { .. }
                | Event::Rehearsing { .. }
                | Event::Back { .. }
                | Event::BackRefused { .. }
                | Event::Closed { .. }
                | Event::Job { .. }
                | Event::Ran { .. }) => panic!("decode event {other:?}"),
            }
        }
        let event = match below.transcripts.pop().expect("read or stop") {
            file::Request::OpenRead { owner, .. } => {
                file::Event::Opened { owner, file: Token::new(2), len: len_u64(raw.len()) }
            }
            file::Request::ReadAt { owner, offset, max, .. } => {
                let start = usize::try_from(offset).expect("offset");
                let end = start.saturating_add(skein_lib::length::usize_of(max)).min(raw.len());
                file::Event::Read { owner, bytes: raw.get(start..end).unwrap_or_default().into() }
            }
            other @ (file::Request::Lock { .. }
            | file::Request::MakeDirectory { .. }
            | file::Request::OpenPrivate { .. }
            | file::Request::Create { .. }
            | file::Request::CreateNoFollow { .. }
            | file::Request::OpenReadNoFollow { .. }
            | file::Request::OpenSiblingFile { .. }
            | file::Request::OpenWrite { .. }
            | file::Request::OpenWriteNoFollow { .. }
            | file::Request::OpenDirectory { .. }
            | file::Request::Load { .. }
            | file::Request::Scan { .. }
            | file::Request::Store { .. }
            | file::Request::Stat { .. }
            | file::Request::WriteAt { .. }
            | file::Request::Cut { .. }
            | file::Request::Sync { .. }
            | file::Request::Close { .. }
            | file::Request::SyncDirectory { .. }
            | file::Request::HardLink { .. }
            | file::Request::Rename { .. }
            | file::Request::Remove { .. }
            | file::Request::RemoveIn { .. }
            | file::Request::List { .. }
            | file::Request::ListPage { .. }) => panic!("read operation {other:?}"),
        };
        store::up(component, env, store::Io::Transcripts(store::Arrived { event, fd: None }), &mut above, &mut below);
    }
    panic!("bounded frame did not settle");
}
