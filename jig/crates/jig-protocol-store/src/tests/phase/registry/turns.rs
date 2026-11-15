//! Inner-version census over file frames and compressed log records (protocol/store/jobs.md, 5.4).
use super::*;
use crate::{JobCount, TranscriptRead, codec, keys, logs, transcripts::frame};
use jig_domain_core_vocabulary::TurnRecord;
use settling::{JOB, family, set};

const FILE: u64 = 101;
const SECOND: u64 = 102;
const LOG: u64 = 201;
const PATH: &[u8] = b"transcripts/0000000000000000/0000000000000065";
const SECOND_PATH: &[u8] = b"transcripts/0000000000000000/0000000000000066";
const LOG_PATH: &[u8] = b"generations/1/store/logs/0000000000000000/0000000000000193";

fn put(own: keys::Own, value: Box<[u8]>) -> skein_kv::Op {
    skein_kv::Op::Put { key: keys::encode_own(&own, fixtures::KEY_LIMIT).unwrap(), value }
}
fn place(task: u64, length: u64) -> skein_kv::Op {
    put(keys::Own::Transcript { task }, codec::component::Transcript { length, window: 0 }.encode().unwrap())
}
fn turn(task: u64, number: u32, version: u16, large: bool) -> TurnRecord {
    let mut payload = vec![b'x'; if large { 8192 } else { 2 }];
    payload[..2].copy_from_slice(&version.to_be_bytes());
    let mut turn = fixtures::window_turn(task, 1, number, &payload);
    // Exercise the longer head, whose transcript follows Some(read).
    turn.read = if large { Some(17) } else { None };
    turn
}
fn scene(owner: Owner) -> Vec<Item> {
    let mut disk = Disk::new(&settling::baseline(owner, &[]));
    let mut store = store(&disk, set(owner, family(owner), &[]));
    assert!(!disk.open(&mut store).iter().any(|e| if let Event::OpenRefused { .. } = e { true } else { false }));
    let first: Vec<u8> = [(1, 7), (2, 6), (3, 7), (4, 7)]
        .into_iter()
        .flat_map(|(number, version)| {
            frame::encode(&turn(FILE, number, version, number == 1), &store.codec_limits).unwrap()
        })
        .collect();
    let mut second: Vec<u8> = [(1, 7), (2, 13), (3, 7)]
        .into_iter()
        .flat_map(|(number, version)| {
            frame::encode(&turn(SECOND, number, version, false), &store.codec_limits).unwrap()
        })
        .collect();
    // A complete span whose body is too short to hold its head, followed by a readable span.
    second.extend_from_slice(&[0, 0, 0, 16, 0, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16]);
    second.extend_from_slice(&frame::encode(&turn(SECOND, 5, 7, false), &store.codec_limits).unwrap());
    let mut ops = vec![place(FILE, len_u64(first.len())), place(SECOND, len_u64(second.len()))];
    for task in [FILE, SECOND] {
        ops.push(put(
            keys::Own::Attempt { task, attempt: 1 },
            codec::component::Attempt { first: 0 }.encode().unwrap(),
        ));
    }
    for (task, versions) in [(LOG, vec![7, 7]), (LOG + 1, vec![6, 42])] {
        ops.push(put(keys::Own::TurnLog { task }, codec::component::TurnLog { window: Some(0) }.encode().unwrap()));
        ops.push(put(
            keys::Own::LogAttempt { task, attempt: 1 },
            codec::component::LogAttempt { first: 0 }.encode().unwrap(),
        ));
        for (i, version) in versions.into_iter().enumerate() {
            let body =
                codec::core::turn(&turn(task, u32::try_from(i + 1).unwrap(), version, true), &store.codec_limits)
                    .unwrap();
            ops.push(skein_kv::Op::Append {
                log: logs::turns(task).unwrap(),
                frame: store.stores.store.frame(&body).unwrap(),
            });
        }
    }
    // A log record whose body head is cut; KV's checksum and framing remain sound.
    ops.push(skein_kv::Op::Append {
        log: logs::turns(LOG + 1).unwrap(),
        frame: store.stores.store.frame(&[0, 6]).unwrap(),
    });
    for chunk in ops.chunks(8) {
        raw(&mut disk, &mut store, chunk.to_vec());
    }
    let mut items = disk.items();
    items.push(Item::directory(b"transcripts/0000000000000000"));
    // An uncommitted frame is not part of the census, nor is a file with no place row.
    let mut with_tail = first.clone();
    with_tail.extend_from_slice(&frame::encode(&turn(FILE, 5, 6, false), &store.codec_limits).unwrap());
    items.push(Item::file(PATH, &with_tail));
    items.push(Item::file(SECOND_PATH, &second));
    items.push(Item::file(b"transcripts/0000000000000000/000000000000ffff", &first));
    items
}
fn ran(events: &[Event], owner: Owner) -> Vec<JobCount> {
    events
        .iter()
        .find_map(|e| {
            if let Event::Ran { of, job, counts, .. } = e {
                assert_eq!(*of, owner);
                assert_eq!(job.as_ref(), JOB.name);
                Some(counts.to_vec())
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("no Ran: {events:?}"))
}
fn expected(turns: [u64; 4]) -> Vec<JobCount> {
    ["put", "retired", "turns-6", "turns-7", "turns-other", "turns-unread"]
        .into_iter()
        .zip([0, 0].into_iter().chain(turns))
        .map(|(name, count)| JobCount { name: name.into(), count })
        .collect()
}
type Read = Option<(Box<[TurnRecord]>, Option<u64>, Option<u32>)>;
fn reads(disk: &mut Disk, store: &mut Store) -> Vec<Read> {
    let mut answers = Vec::new();
    for task in [FILE, SECOND, LOG, LOG + 1] {
        for read in [
            TranscriptRead::Window { task, attempt: 1, turn: 1, after: None },
            TranscriptRead::Turn { task, attempt: 1, turn: 2 },
            TranscriptRead::Page { task, backwards: false, after: None },
            TranscriptRead::Page { task, backwards: true, after: None },
        ] {
            let events =
                disk.request(store, Request::Transcript { owner: Token::new(91), read, rows: 16, bytes: 1_000_000 });
            answers.push(match events.as_ref() {
                [Event::Turns { turns, next, cut, .. }] => Some((turns.clone(), *next, *cut)),
                [Event::Unread { .. }] => None,
                _ => panic!("{events:?}"),
            });
        }
    }
    answers
}
#[test]
fn settling_counts_inner_versions_once_without_changing_payloads_or_serving_reads() {
    let items = scene(Owner::Jig);
    let mut before_disk = Disk::new(&items);
    let mut before_store = store(&before_disk, set(Owner::Jig, family(Owner::Jig), &[]));
    before_disk.open(&mut before_store);
    let before_reads = reads(&mut before_disk, &mut before_store);
    assert!(before_reads.iter().any(Option::is_some));
    let files = [before_disk.bytes(PATH), before_disk.bytes(SECOND_PATH)];
    let log = before_disk.bytes(LOG_PATH);
    let mut disk = Disk::new(&items);
    let mut store = store(&disk, set(Owner::Jig, family(Owner::Jig), &[JOB]));
    let events = disk.open(&mut store);
    assert_eq!(ran(&events, Owner::Jig), expected([2, 8, 2, 2]));
    assert_eq!(disk.bytes(PATH), files[0]);
    assert_eq!(disk.bytes(SECOND_PATH), files[1]);
    assert_eq!(disk.bytes(LOG_PATH), log);
    assert_eq!(disk.bytes(b"generations/2/store/logs/0000000000000000/0000000000000193"), log);
    assert_eq!(reads(&mut disk, &mut store), before_reads);
}
#[test]
fn settling_file_census_reads_only_a_bounded_head_even_for_large_frames() {
    let mut disk = Disk::new(&scene(Owner::Jig));
    let mut store = store(&disk, set(Owner::Jig, family(Owner::Jig), &[JOB]));
    let events = disk.open(&mut store);
    assert_eq!(ran(&events, Owner::Jig), expected([2, 8, 2, 2]));
    let reads: Vec<u32> = disk
        .requests()
        .iter()
        .filter_map(|(lane, seen)| if let (2, Seen::Read { max }) = (lane, seen) { Some(*max) } else { None })
        .collect();
    assert_ne!(reads.len(), 0);
    assert!(reads.iter().all(|max| *max <= crate::transcripts::census::HEAD));
    assert_eq!(reads.iter().sum::<u32>(), 58 + 7 * 50 + 8);
}
#[test]
fn empty_jig_census_counts_zeros_and_application_settling_has_no_turn_counts() {
    for owner in [Owner::Jig, Owner::Application] {
        let items = if owner == Owner::Jig { settling::baseline(owner, &[]) } else { scene(owner) };
        let mut disk = Disk::new(&items);
        let mut store = store(&disk, set(owner, family(owner), &[JOB]));
        let counts = ran(&disk.open(&mut store), owner);
        if owner == Owner::Jig {
            assert_eq!(counts, expected([0; 4]));
        } else {
            assert_eq!(counts, expected([0; 4])[..2]);
        }
        assert!(!disk.requests().iter().any(|(lane, seen)| if let (2, Seen::Read { .. }) = (lane, seen) {
            true
        } else {
            false
        }));
    }
}
#[test]
fn census_marks_inaccessible_files_and_logs_once_and_continues_in_key_order() {
    let mut disk = Disk::new(&settling::baseline(Owner::Jig, &[]));
    let mut source = store(&disk, set(Owner::Jig, family(Owner::Jig), &[]));
    disk.open(&mut source);
    let good = frame::encode(&turn(303, 1, 7, false), &source.codec_limits).unwrap();
    let mut ops = vec![place(301, 16), place(302, 60), place(303, len_u64(good.len()))];
    for task in [401, 402] {
        let body = codec::core::turn(&turn(task, 1, 6, false), &source.codec_limits).unwrap();
        ops.push(put(keys::Own::TurnLog { task }, codec::component::TurnLog { window: Some(0) }.encode().unwrap()));
        ops.push(skein_kv::Op::Append {
            log: logs::turns(task).unwrap(),
            frame: source.stores.store.frame(&body).unwrap(),
        });
    }
    raw(&mut disk, &mut source, ops);
    let mut items: Vec<Item> = disk
        .contents()
        .into_iter()
        .filter(|(path, ..)| path.as_ref() != b"generations/1/store/logs/0000000000000000/0000000000000323")
        .map(|(path, kind, mode, bytes)| match kind {
            Kind::Directory => Item::directory(&path).mode(mode),
            Kind::File => Item::file(&path, &bytes).mode(mode),
            _ => panic!("regular fixture"),
        })
        .collect();
    items.push(Item::directory(b"transcripts/0000000000000000"));
    items.push(Item::file(b"transcripts/0000000000000000/000000000000012e", &[0, 0, 0, 60, 0, 6]));
    items.push(Item::file(b"transcripts/0000000000000000/000000000000012f", &good));
    let mut disk = Disk::new(&items);
    let mut store = store(&disk, set(Owner::Jig, family(Owner::Jig), &[JOB]));
    let events = disk.open(&mut store);
    assert_eq!(ran(&events, Owner::Jig), expected([1, 1, 0, 3]));
}
#[test]
fn rehearsal_reports_the_same_inner_version_counts_as_start() {
    let items = scene(Owner::Jig);
    let mut disk = Disk::new(&items);
    let mut store = store(&disk, set(Owner::Jig, family(Owner::Jig), &[JOB]));
    let scratch = disk.scratch_root();
    let directory = disk.root();
    let events = disk.request(
        &mut store,
        Request::Rehearse { owner: Token::new(1), directory, scratch, installation: fixtures::INSTALLATION },
    );
    assert_eq!(ran(&events, Owner::Jig), expected([2, 8, 2, 2]));
    let mut source = Disk::new(&items);
    assert_eq!(disk.bytes(PATH), source.bytes(PATH));
    assert_eq!(disk.bytes(LOG_PATH), source.bytes(LOG_PATH));
}
