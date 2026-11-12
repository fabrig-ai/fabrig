//! Stopped-deployment tools over fixture patch sets (protocol/store/opening.md 5).
use jig_protocol_store::{BackRefusal, Event, KeptWire, OpenRefusal, Owner, PatchSet, PatchSets, Request, Wire, Write};
use jig_store_world::{World, patches::{self, BASE, Records, SETS, flights}, root};
use skein_lib::Token;

const SEED: u64 = 0x5245_4845;
const OWNER: Token = Token::new(900);
const WIRES: &[Wire] = &[Wire { name: "fixture-worker", version: 7 }];
const MID: PatchSets = PatchSets { sets: &[
    PatchSet { families: &[SETS.sets[0].families[0], BASE.sets[0].families[1], BASE.sets[0].families[2]], patches: &[jig_test_patches::patches::PATCHES[0]], ..SETS.sets[0] },
    BASE.sets[1],
] };

fn lower() -> World {
    let mut world = Records::story().committed(SEED, BASE);
    world.patch_sets(MID);
    let last = world.open_switched().expect("level one start");
    assert!(world.wires(WIRES));
    assert_eq!(world.commit(Box::new([Write::Save(root::header(last + 1)), Write::Wires])), Some(last + 1));
    world.crash(SEED);
    world
}
fn close(world: &mut World) {
    let events = world.request(Request::Close { owner: OWNER });
    assert!(matches!(events.as_slice(), [Event::Closed { owner: OWNER, removed: true }]), "{events:?}");
    assert!(world.scratch_contents().is_empty(), "scratch emptied");
}
fn rehearsal_lines(events: &[Event], wires: Option<&[KeptWire]>, fresh: bool) {
    let Some(Event::Rehearsing { owner: OWNER, wires: found, fresh: actual }) = events.first() else { panic!("first line: {events:?}") };
    assert_eq!(found.as_deref(), wires);
    assert_eq!(*actual, fresh);
}

#[test]
fn rehearsal_reports_kept_wires_and_patch_counts_without_writing_source() {
    let mut world = lower();
    world.patch_sets(SETS);
    let before = world.contents();
    let events = world.rehearse();
    rehearsal_lines(&events, Some(&[KeptWire { name: "fixture-worker".into(), version: 7 }]), false);
    assert!(events.iter().any(|event| matches!(event, Event::Patched { of: Owner::Application, patch: 2, counts, .. } if counts.records == u64::from(patches::STORY_SHELVES) + patches::STORY_TOKENS && counts.commits == 3)), "{events:?}");
    assert!(matches!(events.last(), Some(Event::Opened { pending: true, .. })), "{events:?}");
    Records::story().check_public(&mut world, 2, &"rehearsed copy");
    assert_eq!(world.contents(), before, "every source path and byte unchanged");
    close(&mut world);
    assert_eq!(world.contents(), before, "close only empties scratch");
}

#[test]
fn equal_rehearsal_opens_its_own_generation_without_wires_or_patches() {
    let mut world = Records::story().committed(SEED, BASE);
    let before = world.contents();
    let events = world.rehearse();
    rehearsal_lines(&events, None, false);
    assert!(matches!(events.as_slice(), [Event::Rehearsing { .. }, Event::Opened { last: 1, pending: false, .. }]), "{events:?}");
    assert_eq!(world.scratch_contents().get(b"active".as_slice()).and_then(Option::as_deref), Some(b"1\n".as_slice()));
    assert_eq!(world.contents(), before);
    close(&mut world);
}

#[test]
fn rehearsal_refusals_leave_every_source_file_unchanged() {
    let mut records = Records::story();
    records.shelves[2].books = 0;
    let mut world = records.committed(SEED, BASE);
    world.patch_sets(SETS);
    let before = world.contents();
    let events = world.rehearse();
    rehearsal_lines(&events, None, false);
    assert!(matches!(events.last(), Some(Event::OpenRefused { why: OpenRefusal::Patch { owner: Owner::Application, patch: 2, .. }, .. })), "{events:?}");
    assert_eq!(world.contents(), before);
    close(&mut world);

    let mut world = flights::committed(1);
    world.patch_sets(flights::CHANGES_FLIGHT);
    let before = world.contents();
    let events = world.rehearse();
    rehearsal_lines(&events, None, false);
    assert!(matches!(events.last(), Some(Event::OpenRefused { why: OpenRefusal::InFlight { .. }, .. })), "{events:?}");
    assert_eq!(world.contents(), before);
    close(&mut world);
}

#[test]
fn fresh_rehearsal_reads_shared_transcripts_without_cutting_them() {
    let mut world = patches::fresh_committed(SEED, patches::FRESH_WINDOWS, false);
    world.patch_sets(patches::FRESH_AFTER);
    let before = world.contents();
    let events = world.rehearse();
    rehearsal_lines(&events, None, true);
    assert!(matches!(events.last(), Some(Event::Opened { pending: true, .. })), "{events:?}");
    assert_eq!(world.contents(), before);
    close(&mut world);
}

#[test]
fn rehearsal_refuses_switch_and_close_is_only_for_rehearsals() {
    let mut world = lower();
    world.patch_sets(SETS);
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
    let mut world = lower();
    world.patch_sets(SETS);
    let _ = world.rehearse();
    let old = world.scratch_contents();
    assert!(old.keys().any(|path| path.starts_with(b"generations/3")));
    world.crash(SEED);
    world.patch_sets(MID);
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
    world.patch_sets(SETS);
    let last = world.open_switched().expect("patched and switched");
    world.crash(SEED);
    (world, last)
}
#[test]
fn back_before_own_commit_reopens_previous_records_and_transcript_lengths() {
    let mut world = patches::fresh_committed(SEED, patches::FRESH_WINDOWS, false);
    let before = world.contents();
    world.patch_sets(patches::FRESH_AFTER);
    world.open_switched().expect("fresh patch switched");
    world.crash(SEED);
    let events = world.back();
    assert!(matches!(events.as_slice(), [Event::Back { owner: OWNER, to: 1 }]), "{events:?}");
    assert_eq!(world.active(), b"1\n");
    world.crash(SEED);
    world.patch_sets(patches::FRESH_BEFORE);
    world.open().expect("previous generation opens");
    for (path, bytes) in before.iter().filter(|(path, _)| path.starts_with(b"transcripts/")) {
        assert_eq!(world.contents().get(path), Some(bytes), "previous transcript length");
    }
    assert!(!world.paths().iter().any(|path| path.starts_with(b"generations/2")));
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
    assert!(matches!(events.as_slice(), [Event::BackRefused { why: BackRefusal::Committed { manifest, last: found }, .. }] if *manifest == last && *found == last + 1), "{events:?}");
    assert_eq!(world.contents(), before);

    let mut world = Records::story().committed(SEED, BASE);
    let before = world.contents();
    assert!(matches!(world.back().as_slice(), [Event::BackRefused { why: BackRefusal::NoPrevious, .. }]));
    assert_eq!(world.contents(), before);
    let (mut world, _) = switched();
    world.rewrite(b"generations/2/manifest", b"unreadable");
    let before = world.contents();
    assert!(matches!(world.back().as_slice(), [Event::BackRefused { why: BackRefusal::Unreadable, .. }]));
    assert_eq!(world.contents(), before);
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
            world.patch_sets(BASE);
            world.open().expect("old generation whole");
            Records::story().check_public(&mut world, 1, &cut);
        } else {
            assert_eq!(active, b"2\n");
            world.open().expect("patched generation whole");
            Records::story().check_public(&mut world, 2, &cut);
        }
    }
}
