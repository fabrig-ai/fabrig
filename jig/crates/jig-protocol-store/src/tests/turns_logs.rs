//! New turns use compressed payload logs, with the same reads as retained files (protocol/store/logs.md, 5).
use super::fixtures::{self, LIVE};
use crate::{Event, Request, Store, TranscriptRead, Write, codec, keys, logs};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::{CoreRecord, Record, TurnRecord, reads};
use skein_lib::{List, Queue, Token};

const OWNER: Token = Token::new(801);
fn own(store: &mut Store, key: keys::Own) -> Option<Box<[u8]>> {
    fixtures::get_kv(store, &keys::encode_own(&key, fixtures::KEY_LIMIT).unwrap())
}
fn submit(store: &mut Store, turns: &[TurnRecord]) {
    let number = store.stores.store.last().checked_add(1).unwrap();
    let mut writes = List::with_capacity(4);
    writes.push(fixtures::header(number)).unwrap();
    for turn in turns {
        writes.push(Write::Save(Record::Core(CoreRecord::Turn(turn.clone())))).unwrap();
    }
    fixtures::committed(store, writes.into_boxed());
}
fn opened() -> Store {
    let mut store = fixtures::opened();
    fixtures::committed(&mut store, Box::new([fixtures::header(1)]));
    store
}
fn end(store: &mut Store) -> u64 {
    let bytes = fixtures::get_kv(store, &logs::extent_key(logs::turns(LIVE).unwrap())).unwrap();
    logs::extent(&bytes).unwrap()
}
fn length(store: &Store, turn: &TurnRecord) -> u64 {
    store.stores.store.frame(&codec::core::turn(turn, &store.codec_limits).unwrap()).unwrap().len()
}
fn ask(store: &mut Store, disk: Option<&[u8]>, request: Request) -> Box<[Event]> {
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    crate::down(store, &fixtures::env(), request, &mut above, &mut below);
    if !above.is_empty() {
        return fixtures::events(&mut above);
    }
    if let Some(disk) = disk {
        return super::transcripts::answer_reads(store, below, disk, 4096);
    }
    for _ in 0_u32..128 {
        assert!(below.store.is_empty() && below.transcripts.is_empty());
        crate::fire(store, &fixtures::env(), &mut above, &mut below);
        if !above.is_empty() {
            return fixtures::events(&mut above);
        }
    }
    panic!("memory turns read must settle");
}
fn transcript(read: TranscriptRead, rows: u32, bytes: u64) -> Request {
    Request::Transcript { owner: OWNER, read, rows, bytes }
}
fn page(events: Box<[Event]>) -> (Box<[TurnRecord]>, Option<u64>, Option<u32>) {
    let mut events = events.into_iter();
    let Some(Event::Turns { owner: OWNER, turns, next, cut }) = events.next() else { panic!("turns page") };
    assert!(events.next().is_none());
    (turns, next, cut)
}
fn turns() -> [TurnRecord; 3] {
    [
        fixtures::window_turn(LIVE, 1, 1, b"old"),
        fixtures::window_turn(LIVE, 2, 1, b"one"),
        fixtures::turn(LIVE, 2, 2, b"two"),
    ]
}

#[test]
fn new_turns_append_bodies_and_keep_window_and_first_attempt_places() {
    let mut store = opened();
    let turns = turns();
    let first = length(&store, &turns[0]);
    let second = length(&store, &turns[1]);
    submit(&mut store, &turns[..2]);
    submit(&mut store, &turns[2..]);
    assert_eq!(end(&mut store), first.checked_add(second).unwrap().checked_add(length(&store, &turns[2])).unwrap());
    assert_eq!(own(&mut store, keys::Own::Transcript { task: LIVE }), None);
    let row = own(&mut store, keys::Own::TurnLog { task: LIVE }).unwrap();
    assert_eq!(codec::component::TurnLog::decode(&row).unwrap().window, Some(first));
    for (attempt, offset) in [(1, 0), (2, first)] {
        let row = own(&mut store, keys::Own::LogAttempt { task: LIVE, attempt }).unwrap();
        assert_eq!(codec::component::LogAttempt::decode(&row).unwrap().first, offset);
    }
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    skein_kv::down(
        &mut store.stores.store,
        &crate::memory_env(store.limits.store),
        skein_kv::Request::ReadLog {
            owner: OWNER,
            log: logs::turns(LIVE).unwrap(),
            at: 0,
            direction: skein_kv::Direction::Forward,
            max: 4096,
        },
        &mut above,
        &mut below,
    );
    let Some(skein_kv::Event::LogRead { records, next: None, .. }) = above.pop() else { panic!("log records") };
    assert_eq!(records.len(), turns.len());
    for (record, turn) in records.iter().zip(&turns) {
        assert_eq!(record.record.as_deref(), Some(codec::core::turn(turn, &store.codec_limits).unwrap().as_ref()));
    }
    let repeated = fixtures::turn(LIVE, 2, 3, &[b'x'; 1024]);
    assert!(length(&store, &repeated) < 1024, "skein's framing compresses the body");
}

#[test]
fn log_read_kinds_match_files_and_keep_directional_offset_cursors() {
    let turns = turns();
    let (mut file, _) = fixtures::opened_with_transcripts();
    let number = 2;
    let writes = Box::new([
        fixtures::header(number),
        Write::Save(Record::Core(CoreRecord::Turn(turns[0].clone()))),
        Write::Save(Record::Core(CoreRecord::Turn(turns[1].clone()))),
        Write::Save(Record::Core(CoreRecord::Turn(turns[2].clone()))),
    ]);
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    crate::down(&mut file, &fixtures::env(), fixtures::commit(number, writes), &mut above, &mut below);
    let disk = fixtures::written(&mut file, below, &turns);
    let mut log = opened();
    submit(&mut log, &turns);
    for read in [
        TranscriptRead::Window { task: LIVE, attempt: 2, turn: 1, after: None },
        TranscriptRead::Turn { task: LIVE, attempt: 2, turn: 2 },
        TranscriptRead::Page { task: LIVE, backwards: false, after: None },
        TranscriptRead::Page { task: LIVE, backwards: true, after: None },
    ] {
        let expected = page(ask(&mut file, Some(&disk), transcript(read, 2, 4096)));
        let got = page(ask(&mut log, None, transcript(read, 2, 4096)));
        assert_eq!(got.0, expected.0);
        assert_eq!(got.2, expected.2);
        assert_eq!(got.1.is_some(), expected.1.is_some());
    }
    for backwards in [false, true] {
        let mut after = None;
        for expected in if backwards { [2, 1, 0] } else { [0, 1, 2] } {
            let (got, next, cut) =
                page(ask(&mut log, None, transcript(TranscriptRead::Page { task: LIVE, backwards, after }, 1, 4096)));
            assert_eq!(got.as_ref(), &turns[expected..=expected]);
            assert_eq!(cut, None);
            after = next;
        }
        assert_eq!(after, None);
    }
    let (_, next, _) = page(ask(
        &mut log,
        None,
        transcript(TranscriptRead::Window { task: LIVE, attempt: 2, turn: 1, after: None }, 1, 4096),
    ));
    let (got, next, _) = page(ask(
        &mut log,
        None,
        transcript(TranscriptRead::Window { task: LIVE, attempt: 2, turn: 1, after: next }, 1, 4096),
    ));
    assert_eq!(got.as_ref(), &turns[2..]);
    assert_eq!(next, None);
}

fn party_scene(file: bool) -> (Store, Box<[u8]>) {
    let mut store = fixtures::opened();
    let mut writes = List::with_capacity(4);
    writes.push(fixtures::header(1)).unwrap();
    let Record::Tasks(jig_domain_tasks::Stored::Ended(mut owner)) =
        fixtures::ended_of(LIVE, jig_domain_tasks::Party::Person(fixtures::ALICE))
    else {
        panic!("ended task")
    };
    owner.activity = 2;
    writes.push(fixtures::save(Record::Tasks(jig_domain_tasks::Stored::Ended(owner)))).unwrap();
    for turn in 1..=2 {
        writes
            .push(fixtures::save(Record::Tasks(jig_domain_tasks::Stored::Activity(jig_domain_tasks::ActivityLine {
                task: LIVE,
                position: u64::from(turn),
                at: skein_lib::Wall::EPOCH,
                line: jig_domain_tasks::Line::Turn { reply: u64::from(turn), attempt: 1, turn, read: None },
            }))))
            .unwrap();
    }
    if file {
        fixtures::committed_with_places(&mut store, writes.into_boxed(), &[LIVE]);
    } else {
        fixtures::committed(&mut store, writes.into_boxed());
    }
    let turns = [fixtures::window_turn(LIVE, 1, 1, b"one"), fixtures::turn(LIVE, 1, 2, &[b'x'; 1024])];
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    crate::down(
        &mut store,
        &fixtures::env(),
        fixtures::commit(
            2,
            Box::new([
                fixtures::header(2),
                fixtures::save(Record::Core(CoreRecord::Turn(turns[0].clone()))),
                fixtures::save(Record::Core(CoreRecord::Turn(turns[1].clone()))),
            ]),
        ),
        &mut above,
        &mut below,
    );
    let disk = if file { fixtures::written(&mut store, below, &turns) } else { Box::new([]) };
    (store, disk)
}

#[test]
fn activity_and_party_turn_answers_match_files_including_oversize_evidence() {
    let (mut file, disk) = party_scene(true);
    let (mut log, _) = party_scene(false);
    let mut range = fixtures::read(fixtures::ALICE, reads::Scope::Turn { task: LIVE, attempt: 1, turn: 2 });
    range.bytes = 65_536;
    let events = ask(&mut log, None, Request::Read { owner: OWNER, range });
    let [Event::Read { result: reads::Loaded::Page { rows, .. }, .. }] = events.as_ref() else { panic!("sized turn") };
    let tight = rows.first().unwrap().owned_bytes().unwrap().checked_sub(1).unwrap();
    for backwards in [false, true] {
        for bytes in [65_536, tight, 1] {
            for scope in [reads::Scope::Activity { task: LIVE }, reads::Scope::Turn { task: LIVE, attempt: 1, turn: 2 }]
            {
                let mut range = fixtures::read(fixtures::ALICE, scope);
                range.backwards = backwards;
                range.bytes = bytes;
                let expected = ask(&mut file, Some(&disk), Request::Read { owner: OWNER, range: range.clone() });
                let got = ask(&mut log, None, Request::Read { owner: OWNER, range });
                let ([Event::Read { result: got, .. }], [Event::Read { result: expected, .. }]) =
                    (got.as_ref(), expected.as_ref())
                else {
                    panic!("party read answers")
                };
                if bytes == 65_536 {
                    let reads::Loaded::Page { rows, .. } = expected else { panic!("complete page: {expected:?}") };
                    assert!(!rows.is_empty(), "complete turns page");
                    for row in rows {
                        let record = row.related.as_ref().unwrap_or(&row.value);
                        let Record::Core(CoreRecord::Turn(_)) = record else { panic!("joined turn: {row:?}") };
                    }
                }
                super::history_logs::equal(got, expected);
            }
        }
    }
}

#[test]
fn log_window_reads_a_record_alone_when_the_page_has_no_room() {
    let mut store = opened();
    let turns = [fixtures::window_turn(LIVE, 2, 1, b"one"), fixtures::turn(LIVE, 2, 2, &[b'x'; 1024])];
    submit(&mut store, &turns);
    let first = length(&store, &turns[0]);
    let read = |after| TranscriptRead::Window { task: LIVE, attempt: 2, turn: 1, after };
    let (got, next, cut) = page(ask(&mut store, None, transcript(read(None), 2, 67)));
    assert_eq!(got.as_ref(), &turns[..1]);
    assert_eq!(next, Some(first));
    assert_eq!(cut, None);
    let (got, next, cut) = page(ask(&mut store, None, transcript(read(next), 2, 67)));
    let mut expected = turns[1].clone();
    expected.transcript = Box::new([]);
    assert_eq!(got.as_ref(), &[expected]);
    assert_eq!(next, None);
    assert_eq!(cut, Some(1024));
}

#[test]
fn a_log_frame_past_the_page_bound_refuses_the_whole_commit() {
    let mut store = opened();
    let turn = fixtures::turn(LIVE, 1, 1, b"one");
    let bytes = u32::try_from(length(&store, &turn)).unwrap();
    store.limits.store.page.bytes = bytes.checked_sub(1).unwrap();
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    crate::down(
        &mut store,
        &fixtures::env(),
        fixtures::commit(2, Box::new([fixtures::header(2), Write::Save(Record::Core(CoreRecord::Turn(turn)))])),
        &mut above,
        &mut below,
    );
    let Some(Event::FrameTooLarge { number: 2, task: LIVE, bytes: got, limit }) = above.pop() else {
        panic!("named frame refusal")
    };
    assert_eq!((got, limit), (bytes, bytes.checked_sub(1).unwrap()));
    assert!(above.is_empty() && below.store.is_empty() && below.transcripts.is_empty());
    assert_eq!(store.stores.store.last(), 1);
}

#[test]
fn dropping_a_turns_log_erases_its_place_and_sweeps_orphans_after_reopen() {
    let mut store = opened();
    submit(&mut store, &turns());
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    crate::down(
        &mut store,
        &fixtures::env(),
        fixtures::commit(3, Box::new([fixtures::header(3), Write::DropTranscript { task: LIVE }])),
        &mut above,
        &mut below,
    );
    assert!(above.is_empty());
    fixtures::script(
        &mut store,
        below,
        &[
            fixtures::Answer::Opened { len: 0 },
            fixtures::Answer::Removed,
            fixtures::Answer::Synced,
            fixtures::Answer::Closed,
        ],
    );
    assert_eq!(store.stores.store.log_end(logs::turns(LIVE).unwrap()), 0);
    assert_eq!(own(&mut store, keys::Own::TurnLog { task: LIVE }), None);
    assert!(own(&mut store, keys::Own::LogAttempt { task: LIVE, attempt: 1 }).is_some());
    let events = ask(
        &mut store,
        None,
        transcript(TranscriptRead::Window { task: LIVE, attempt: 2, turn: 1, after: None }, 1, 4096),
    );
    let [Event::Unread { .. }] = events.as_ref() else { panic!("dropped log is unread") };
    let mut store = fixtures::over(store.into_stores());
    fixtures::run(
        &mut store,
        Request::Open { owner: OWNER, root: fixtures::ROOT, installation: fixtures::INSTALLATION },
    );
    submit(&mut store, &[]);
    assert_eq!(own(&mut store, keys::Own::LogAttempt { task: LIVE, attempt: 1 }), None);
    assert!(own(&mut store, keys::Own::LogAttempt { task: LIVE, attempt: 2 }).is_some());
    submit(&mut store, &[]);
    assert_eq!(own(&mut store, keys::Own::LogAttempt { task: LIVE, attempt: 2 }), None);
}

#[test]
fn a_record_skein_cannot_read_is_an_unread_frame() {
    use super::fixtures::disk::Disk;
    use skein_io::file;
    let mut disk = Disk::new(&[]);
    let mut store = disk.store();
    disk.open(&mut store);
    let turn = fixtures::window_turn(LIVE, 1, 1, b"one");
    let events = disk.request(
        &mut store,
        fixtures::commit(1, Box::new([fixtures::header(1), fixtures::save(Record::Core(CoreRecord::Turn(turn)))])),
    );
    let [Event::Committed { number: 1, .. }] = events.as_ref() else { panic!("durable turn: {events:?}") };
    let mut log_path = None;
    for (path, _, _) in disk.tree() {
        if path.ends_with(b"/logs/0000000000000000/000000000000002b") {
            log_path = Some(path);
        }
    }
    let path = log_path.expect("turns log");
    let root = disk.root().fd;
    let arrived = disk.file(0, file::Request::OpenWriteNoFollow { owner: OWNER, root, name: path });
    let file::Event::Opened { file, .. } = arrived.event else { panic!("open log to corrupt it") };
    disk.file(0, file::Request::WriteAt { owner: OWNER, file, offset: 4, bytes: Box::new([2]) });
    disk.file(0, file::Request::Sync { owner: OWNER, file });
    for read in [
        TranscriptRead::Window { task: LIVE, attempt: 1, turn: 1, after: None },
        TranscriptRead::Turn { task: LIVE, attempt: 1, turn: 1 },
        TranscriptRead::Page { task: LIVE, backwards: true, after: None },
    ] {
        let events = disk.request(&mut store, transcript(read, 2, 4096));
        let [Event::Unread { owner: OWNER }] = events.as_ref() else { panic!("unread log frame: {events:?}") };
    }
}

#[test]
fn log_places_include_turns_queued_behind_a_files_sync() {
    const NEW: u64 = 24;
    let (mut store, _) = fixtures::opened_with_transcripts();
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    crate::down(
        &mut store,
        &fixtures::env(),
        fixtures::commit(
            2,
            Box::new([
                fixtures::header(2),
                Write::Save(Record::Core(CoreRecord::Turn(fixtures::turn(LIVE, 1, 1, b"stored")))),
            ]),
        ),
        &mut above,
        &mut below,
    );
    let turns = [fixtures::window_turn(NEW, 1, 1, b"one"), fixtures::window_turn(NEW, 2, 1, b"two")];
    let (first, second) = (length(&store, &turns[0]), length(&store, &turns[1]));
    for (number, turn) in [(3, &turns[0]), (4, &turns[1])] {
        let events = fixtures::run(
            &mut store,
            fixtures::commit(
                number,
                Box::new([fixtures::header(number), Write::Save(Record::Core(CoreRecord::Turn(turn.clone())))]),
            ),
        );
        assert!(events.is_empty());
    }
    fixtures::script(
        &mut store,
        below,
        &[fixtures::Answer::Opened { len: 0 }, fixtures::Answer::Written, fixtures::Answer::Synced],
    );
    assert_eq!(store.stores.store.log_end(logs::turns(NEW).unwrap()), first.checked_add(second).unwrap());
    let row = own(&mut store, keys::Own::TurnLog { task: NEW }).unwrap();
    assert_eq!(codec::component::TurnLog::decode(&row).unwrap().window, Some(first));
    let row = own(&mut store, keys::Own::LogAttempt { task: NEW, attempt: 2 }).unwrap();
    assert_eq!(codec::component::LogAttempt::decode(&row).unwrap().first, first);
    let (got, next, _) = page(ask(
        &mut store,
        None,
        transcript(TranscriptRead::Page { task: NEW, backwards: false, after: None }, 2, 4096),
    ));
    assert_eq!(got.as_ref(), &turns);
    assert_eq!(next, None);
}

#[test]
fn a_commit_refusal_with_the_private_log_token_is_not_lost() {
    let mut store = opened();
    let mut events = Queue::uncapped();
    let owner = crate::transcripts::read::LOG_OWNER;
    events.push(skein_kv::Event::Refused { owner, refusal: skein_kv::Refusal::Busy });
    let mut above = Queue::uncapped();
    let mut below = fixtures::below();
    crate::directory::events_up(&mut store, false, &mut events, &mut above, &mut below);
    let Some(Event::Failed { number, why: crate::CommitFailure::Refused(skein_kv::Refusal::Busy) }) = above.pop()
    else {
        panic!("a commit number can equal the read's private token");
    };
    assert_eq!(number, owner.raw());
    assert!(above.is_empty());
    assert_eq!(store.phase, crate::Phase::Stopped);
}
