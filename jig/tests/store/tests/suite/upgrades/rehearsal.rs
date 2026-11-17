//! Stopped-deployment tools over fixture sets (protocol/store/opening.md 5).
use jig_protocol_store::{
    BackRefusal, Event, KeptMap, KeptWire, OpenRefusal, Owner, OwnerSets, Request, Undecoded, Wire, Write,
};
use jig_store_world::{
    World,
    patches::{self, BASE, Records, SETS},
    root,
    world::Change,
};
use skein_lib::Token;

const SEED: u64 = 0x5245_4845;
const OWNER: Token = Token::new(900);
const WIRES: &[Wire] = &[Wire { name: "fixture-worker", version: 7 }];
const MID: OwnerSets = SETS;
const AFTER: OwnerSets = patches::NEXT;

fn previous() -> World {
    let mut world = Records::story().committed(SEED, BASE);
    world.owner_sets(MID);
    let last = world.open_switched().expect("first job upgrade");
    assert!(world.wires(WIRES));
    assert_eq!(world.commit(Box::new([Write::Save(root::header(last + 1)), Write::Wires])), Some(last + 1));
    world.crash(SEED);
    world
}
fn without_wires(records: &Records) -> World {
    let mut world = World::unopened_observed(SEED, patches::limits(), None);
    world.owner_sets(BASE);
    world.open().expect("source start");
    let writes =
        records.writes().into_vec().into_iter().filter(|write| !matches!(write, Write::Wires)).collect::<Vec<_>>();
    world.commit(writes.into_boxed_slice()).expect("source without start mark");
    world.crash(SEED);
    world
}
fn close(world: &mut World) {
    let events = world.request(Request::Close { owner: OWNER });
    assert!(matches!(events.as_slice(), [Event::Closed { owner: OWNER, removed: true }]), "{events:?}");
    assert!(world.scratch_contents().is_empty(), "scratch emptied");
}
fn rehearsal_lines(events: &[Event], wires: Option<&[KeptWire]>) {
    let Some(Event::Rehearsing { owner: OWNER, wires: found, openings }) = events.first() else {
        panic!("first line: {events:?}")
    };
    assert!(openings.is_empty());
    assert_eq!(found.as_deref(), wires);
}

#[test]
fn rehearsal_reports_kept_wires_and_job_counts_without_writing_source() {
    let mut world = previous();
    world.owner_sets(AFTER);
    let before = world.contents();
    let events = world.rehearse();
    rehearsal_lines(&events, Some(&[KeptWire { name: "fixture-worker".into(), version: 7 }]));
    assert!(events.iter().any(|event| matches!(event, Event::Decoded { owner: OWNER, records: 10 })), "{events:?}");
    assert!(events.iter().any(|event| matches!(event, Event::Ran { of: Owner::Application, job, rows: 3, commits: 2, .. } if job.as_ref() == patches::NEXT.sets[0].jobs[1].name)), "{events:?}");
    assert!(matches!(events.last(), Some(Event::Opened { pending: true, .. })), "{events:?}");
    Records::story().check_public(&mut world, &"rehearsed copy");
    assert_eq!(world.contents(), before, "every source path and byte unchanged");
    close(&mut world);
    assert_eq!(world.contents(), before, "close only empties scratch");
}

#[test]
fn equal_rehearsal_opens_its_own_generation_without_wires_or_pending_job() {
    let mut world = without_wires(&Records::story());
    let before = world.contents();
    let events = world.rehearse();
    rehearsal_lines(&events, None);
    assert!(
        matches!(
            events.as_slice(),
            [
                Event::Rehearsing { .. },
                Event::Decoded { owner: OWNER, records: 7 },
                Event::Opened { last: 1, pending: false, .. }
            ]
        ),
        "{events:?}"
    );
    assert_eq!(world.scratch_contents().get(b"active".as_slice()).and_then(Option::as_deref), Some(b"1\n".as_slice()));
    assert_eq!(world.contents(), before);
    close(&mut world);
}

#[test]
fn rehearsal_decodes_unchanged_families_before_planning_but_start_never_walks() {
    let mut world = previous();
    let last = world.open().expect("equal source start");
    let relative = b"l/1";
    assert_eq!(
        world.commit(Box::new([
            Write::Save(root::header(last + 1)),
            Write::Put { key: Box::from(relative.as_slice()), value: Box::from(b"\x00\x01refused".as_slice()) },
        ])),
        Some(last + 1)
    );
    world.crash(SEED);
    world.owner_sets(AFTER);
    let before = world.contents();
    let events = world.rehearse();
    let [
        Event::Rehearsing { .. },
        Event::Undecodable { owner: OWNER, family, version: 1, at: Undecoded::Record { map: KeptMap::Store, key } },
    ] = events.as_slice()
    else {
        panic!("{events:?}")
    };
    assert_eq!(family.as_ref(), "fixture-label");
    assert_eq!(
        key.as_ref(),
        [jig_protocol_store::keys::APPLICATION].into_iter().chain(relative.iter().copied()).collect::<Vec<_>>()
    );
    assert_eq!(world.contents(), before);
    close(&mut world);
    assert_eq!(world.contents(), before, "failed walk and close leave source unchanged");
    world.crash(SEED);
    world.owner_sets(AFTER);
    assert!(
        world.open_switched().is_some(),
        "start upgrades without decoding the unchanged label: {:?}",
        world.refusal()
    );
}

#[test]
fn job_rehearsal_reads_shared_transcripts_without_cutting_them() {
    let mut world = patches::frame_committed(SEED, patches::FRAME_WINDOWS, false);
    world.owner_sets(patches::FRAME_JOBS);
    let before = world.contents();
    let events = world.rehearse();
    rehearsal_lines(&events, None);
    assert!(matches!(events.last(), Some(Event::Opened { pending: true, .. })), "{events:?}");
    assert_eq!(
        world.get(jig_domain_core::Key::Core(jig_domain_core::CoreKey::RunProof(root::LIVE))),
        patches::frame_proof(root::LIVE, true, false)
    );
    let turns =
        world.transcript(jig_protocol_store::TranscriptRead::Page { task: root::LIVE, backwards: false, after: None });
    assert_eq!(turns.len(), 2, "shared transcript read through the rehearsed copy");
    assert_eq!(world.contents(), before);
    close(&mut world);
}

#[test]
fn rehearsal_refuses_switch_and_close_is_only_for_rehearsals() {
    let mut world = previous();
    world.owner_sets(AFTER);
    let _ = world.rehearse();
    let before = world.contents();
    let events = world.request(Request::Switch { owner: OWNER });
    assert!(matches!(events.as_slice(), [Event::OpenRefused { why: OpenRefusal::Failed, .. }]), "{events:?}");
    assert_eq!(world.contents(), before);
    close(&mut world);
    let mut ordinary = World::new(SEED);
    let before = ordinary.contents();
    let events = ordinary.request(Request::Close { owner: OWNER });
    assert!(matches!(events.as_slice(), [Event::OpenRefused { why: OpenRefusal::Failed, .. }]));
    assert_eq!(ordinary.contents(), before);
}

#[test]
fn next_rehearsal_empties_crashed_scratch_and_close_drains_an_open() {
    let mut world = previous();
    world.owner_sets(AFTER);
    let _ = world.rehearse();
    let old = world.scratch_contents();
    assert!(old.keys().any(|path| path.starts_with(b"generations/3")));
    world.crash(SEED);
    world.owner_sets(MID);
    let events = world.rehearse();
    assert!(matches!(events.last(), Some(Event::Opened { pending: false, .. })), "{events:?}");
    assert!(!world.scratch_contents().keys().any(|path| path.starts_with(b"generations/3")));
    close(&mut world);
    world.crash(SEED);
    let request = world.rehearsal_request();
    world.enqueue(request);
    close(&mut world);
}

fn switched() -> (World, u64) {
    let mut world = Records::story().committed(SEED, BASE);
    world.owner_sets(AFTER);
    let last = world.open_switched().expect("upgraded and switched");
    world.crash(SEED);
    (world, last)
}
fn wires_start() -> (World, u64) {
    let (mut world, last) = switched();
    assert_eq!(world.open(), Some(last), "switched generation opens before its own commit");
    assert!(world.wires(WIRES));
    (world, last)
}

/// Only the mark is traced: its next header and wires form the first commit past the manifest.
pub(crate) fn wires_commit_cuts() -> (u32, u32) {
    let (mut complete, last) = wires_start();
    complete.trace_writes();
    assert_eq!(complete.mark(), Some(last + 1));
    let first = complete.written().first().expect("mark's first write").0;
    let sync = complete
        .changes()
        .iter()
        .find(|(operation, change)| *operation > first && *change == Change::Sync)
        .expect("mark's durability sync")
        .0;
    (first, sync)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WiresOutcome {
    Lost,
    Durable,
}

/// Power loss at the wires' commit keeps going back open or closes it durably
/// (protocol/store/opening.md 2, step 8; domain/testing.md 9).
pub(crate) fn wires_cuts(cuts: &[u32]) -> Vec<WiresOutcome> {
    use std::io::{Read, Seek, Write as _};
    let (mut world, last) = wires_start();
    let records = Records::story();
    // An unlinked file conveys the complete crash image between sibling children.
    // Neither the simulated disk nor a child shares its mutable state.
    let mut image = tempfile::tempfile().expect("anonymous crash-image channel");
    world.trace_writes();
    let sweep = crate::sweep::Sweep::start_with(&mut world, cuts.to_vec(), move |world, cut| {
        let goes_back = crate::sweep::fork_case(world, cut, |world| {
            world.cut_at(cut);
            assert!(world.reached_cut(), "wires cut {cut} reached");
            assert_eq!(world.operation_count(), cut, "wires cut reached exactly");
            world.crash(SEED);
            image.rewind().expect("rewind crash image");
            image.set_len(0).expect("empty crash image");
            let before = world.contents();
            image.write_all(&crash_image(&before)).expect("send complete crash image");
            a_wires_cut_recovers(world, &records, &before, last, cut) == WiresOutcome::Lost
        });
        if goes_back {
            image.rewind().expect("rewind received crash image");
            let mut before = Vec::new();
            image.read_to_end(&mut before).expect("receive complete crash image");
            crate::sweep::fork_case(world, cut, |retry| {
                retry.cut_at(cut);
                assert_eq!(retry.operation_count(), cut, "forward wires cut reached exactly");
                retry.crash(SEED);
                let contents = retry.contents();
                assert_eq!(crash_image(&contents), before, "cut {cut} replays the same crash");
                let manifest = contents
                    .get(b"generations/2/manifest".as_slice())
                    .expect("switched manifest")
                    .as_ref()
                    .expect("manifest file");
                let replay_last =
                    jig_protocol_store::codec::component::Manifest::decode(manifest).expect("manifest decodes").store;
                assert_eq!(replay_last, last);
                assert_eq!(retry.active(), b"2\n");
                let events = retry.request(retry.open_request());
                assert!(
                    matches!(events.as_slice(), [Event::Opened { last: found, pending: false, .. }] if *found == last),
                    "no patch or site on the next start: cut {cut}: {events:?}"
                );
                retry.accept_recovery(last);
                records.check_public(retry, &cut);
                assert!(retry.wires(WIRES));
                assert_eq!(retry.mark(), Some(last + 1), "retry commits its wires: cut {cut}");
                true
            });
        }
        goes_back
    });
    assert_eq!(world.mark(), Some(last + 1));
    sweep
        .finish(&mut world)
        .into_iter()
        .map(|goes_back| if goes_back { WiresOutcome::Lost } else { WiresOutcome::Durable })
        .collect()
}

fn a_wires_cut_recovers(world: &mut World, records: &Records, before: &Contents, last: u64, cut: u32) -> WiresOutcome {
    let events = world.back();
    match events.as_slice() {
        [Event::Back { owner: OWNER, to: 1 }] => {
            assert_eq!(world.active(), b"1\n");
            world.crash(SEED);
            world.owner_sets(BASE);
            assert_eq!(world.open(), Some(1), "previous generation opens: cut {cut}");
            records.check_public(world, &cut);

            WiresOutcome::Lost
        }
        [Event::BackRefused { owner: OWNER, why: BackRefusal::Committed { manifest, last: found } }] => {
            assert_eq!((*manifest, *found), (last, last + 1), "wires commit recovered: cut {cut}");
            unchanged_except_recovery_empty_segments(before, &world.contents(), Some(2));
            WiresOutcome::Durable
        }
        other => panic!("back after wires cut {cut}: {other:?}"),
    }
}

#[test]
fn cuts_at_the_wires_commit_keep_back_open_or_refuse_as_committed() {
    let (first, sync) = wires_commit_cuts();
    let mut lost = false;
    let mut durable = false;
    // A single write can be both the first write and the last operation before sync.
    let mut cuts = vec![first, sync - 1, sync, sync + 1];
    cuts.dedup();
    for (cut, outcome) in cuts.iter().zip(wires_cuts(&cuts)) {
        eprintln!("wires cut {cut}: {outcome:?}");
        lost |= outcome == WiresOutcome::Lost;
        durable |= outcome == WiresOutcome::Durable;
    }
    assert!(lost && durable, "cuts reached both a lost and a durable wires commit");
}

#[test]
fn back_before_own_commit_reopens_previous_records_and_transcript_lengths() {
    let mut world = patches::frame_committed(SEED, patches::FRAME_WINDOWS, false);
    let before = world.contents();
    world.owner_sets(patches::FRAME_JOBS);
    world.open_switched().expect("frame fixture job switched");
    world.crash(SEED);
    let (path, bytes) = before
        .iter()
        .find(|(path, bytes)| path.starts_with(b"generations/1/store/logs/") && bytes.is_some())
        .expect("committed turns log");
    let mut tail = bytes.as_ref().expect("transcript bytes").clone();
    tail.extend_from_slice(b"uncommitted tail");
    world.rewrite(path, &tail);
    let events = world.back();
    assert_eq!(world.contents().get(path).and_then(Option::as_ref), Some(&tail), "Back does not cut turns logs");
    assert!(matches!(events.as_slice(), [Event::Back { owner: OWNER, to: 1 }]), "{events:?}");
    assert_eq!(world.active(), b"1\n");
    world.crash(SEED);
    world.owner_sets(patches::FRAME_BASE);
    world.open().expect("previous generation opens");
    for (name, bytes) in before.iter().filter(|(path, _)| path.starts_with(b"generations/1/store/logs/")) {
        if name == path {
            assert_eq!(
                world.contents().get(name).and_then(Option::as_ref),
                Some(&tail),
                "past the extent stays garbage"
            );
        } else {
            assert_eq!(world.contents().get(name), Some(bytes), "previous turns log");
        }
    }
    let turns =
        world.transcript(jig_protocol_store::TranscriptRead::Page { task: root::LIVE, backwards: false, after: None });
    // Going back preserves the stored bytes; the equal-level binary still
    // patches each frame at read, without changing the previous log.
    let tokens =
        u32::try_from(crate::frame_reads::FRAME_KEY_BYTES + patches::FRAME_REPLAY.len()).expect("fixture tokens");
    let expected = patches::fixture_turn(2, patches::FRAME_SAID, &[], tokens);
    assert_eq!(turns, [root::turn(root::LIVE, 1, 1, &expected), root::turn(root::LIVE, 1, 2, &expected)]);
    assert!(!world.paths().iter().any(|path| path.starts_with(b"generations/2")));
}

type Contents = std::collections::BTreeMap<Vec<u8>, Option<Vec<u8>>>;

// Lengths and the file/directory tag preserve every path and byte exactly in
// the sibling oracle channel; no filesystem or production stored form uses it.
fn crash_image(contents: &Contents) -> Vec<u8> {
    let mut image = Vec::new();
    for (path, bytes) in contents {
        image.extend(path.len().to_ne_bytes());
        image.extend(path);
        if let Some(bytes) = bytes {
            image.push(1);
            image.extend(bytes.len().to_ne_bytes());
            image.extend(bytes);
        } else {
            image.push(0);
        }
    }
    image
}

/// Back only permits skein-kv's recovery to add empty files in the active main map.
fn unchanged_except_recovery_empty_segments(before: &Contents, after: &Contents, recovery: Option<u64>) {
    for (path, bytes) in before {
        assert_eq!(after.get(path), Some(bytes), "existing path changed: {}", String::from_utf8_lossy(path));
    }
    for (path, bytes) in after.iter().filter(|(path, _)| !before.contains_key(*path)) {
        let generation = recovery.expect("a refusal before recovery creates no path");
        let prefix = format!("generations/{generation}/store/log-");
        assert!(path.starts_with(prefix.as_bytes()), "only recovery's next segment may be new");
        assert_eq!(bytes.as_deref(), Some(b"".as_slice()), "only an empty segment may be new");
    }
}

#[test]
fn back_refuses_committed_missing_previous_and_unreadable_manifest() {
    let (mut world, last) = switched();
    world.open().expect("active patched generation opens");
    assert!(world.wires(WIRES));
    world.commit(Box::new([Write::Save(root::header(last + 1)), Write::Wires])).expect("start mark");
    world.crash(SEED);
    let before = world.contents();
    let events = world.back();
    assert!(
        matches!(events.as_slice(), [Event::BackRefused { why: BackRefusal::Committed { manifest, last: found }, .. }] if *manifest == last && *found == last + 1),
        "{events:?}"
    );
    unchanged_except_recovery_empty_segments(&before, &world.contents(), Some(2));

    let mut world = Records::story().committed(SEED, BASE);
    let before = world.contents();
    assert!(matches!(world.back().as_slice(), [Event::BackRefused { why: BackRefusal::NoPrevious, .. }]));
    unchanged_except_recovery_empty_segments(&before, &world.contents(), None);
    let (mut world, _) = switched();
    world.rewrite(b"generations/2/manifest", b"unreadable");
    let before = world.contents();
    assert!(matches!(world.back().as_slice(), [Event::BackRefused { why: BackRefusal::Unreadable, .. }]));
    unchanged_except_recovery_empty_segments(&before, &world.contents(), None);
}

#[test]
fn back_refuses_busy_without_touching_any_file() {
    let (mut world, _) = switched();
    world.open().expect("another process keeps deployment open");
    let before = world.contents();
    let events = world.competing_back();
    assert!(matches!(events.as_slice(), [Event::BackRefused { why: BackRefusal::Busy, .. }]), "{events:?}");
    unchanged_except_recovery_empty_segments(&before, &world.contents(), None);
}

#[test]
fn back_durable_replace_cuts_open_one_whole_generation() {
    use jig_store_world::world::Change;
    let (mut trace, _) = switched();
    trace.trace_writes();
    let _ = trace.back();
    let rename = trace.changes().iter().find(|(_, kind)| *kind == Change::Rename).expect("back rename").0;
    for cut in [rename, rename + 1] {
        let (mut world, _) = switched();
        world.trace_writes();
        world.cut_at(cut);
        let _ = world.back();
        assert!(world.reached_cut());
        world.crash(SEED);
        let active = world.active();
        if active == b"1\n" {
            world.owner_sets(BASE);
            world.open().expect("old generation whole");
            Records::story().check_public(&mut world, &cut);
        } else {
            assert_eq!(active, b"2\n");
            world.open().expect("upgraded generation whole");
            Records::story().check_public(&mut world, &cut);
        }
    }
}
