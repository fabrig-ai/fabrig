//! Compatibility and recovery boundaries of the real Store and FileIo over a fake disk.
//! smith/docs/design/testing.md, 2.5.

mod fixture;

use fixture::{Entry, Fixture, Kind, LIMITS, Run, Snapshot, Terminal};
use skein_fake_machine::{Is, Item};
use skein_lib::Token;
use smith_local_domain::{self as local, Older, StoreFamily};
use smith_local_protocol::local_store_v1::Record;
use smith_local_protocol::{History, StoreRequest, format, local_store_v1 as wire, save_state};

const RESIDUE: &[u8] = b"format.skein-0123456789abcdef";

fn state(activation: u64) -> local::ChatState {
    local::ChatState {
        identity: smith_domain::run::Identity::new([42; 16]),
        activation,
        next_message: 17,
        read: Some(Token::new(11)),
    }
}

fn empty(event: &local::Event) {
    assert!(
        matches!(event, local::Event::Loaded { state: None, transcript: None, deliveries } if deliveries.is_empty()),
        "empty Load: {event:?}"
    );
}

fn loaded_state(event: &local::Event, expected: local::ChatState) {
    assert!(
        matches!(event, local::Event::Loaded { state: Some(value), transcript: None, deliveries } if *value == expected && deliveries.is_empty()),
        "{event:?}"
    );
}

fn marker_bytes() -> Box<[u8]> {
    format::file(&format::layouts()).bytes
}

fn mismatched_marker(store: bool, transcript: bool, position: usize) -> Box<[u8]> {
    let mut layouts = format::layouts();
    if store {
        layouts.store[position] ^= 1;
    }
    if transcript {
        layouts.transcript[position] ^= 1;
    }
    layouts.encode(&wire::CEILINGS).expect("layouts have no variable bounds")
}

fn marker_entry(snapshot: &Snapshot) -> &Entry {
    snapshot.root.iter().find(|entry| entry.name.as_ref() == b"format").expect("format entry")
}

fn old_records() -> Vec<Item> {
    let state = save_state(state(9));
    vec![
        Item::file(b"state", &state.bytes),
        Item::file(b"fresh", &state.bytes),
        Item::file(b"0000000001.turn", b"old transcript bytes"),
        Item::file(b"00000000000000000001.0000000001.0000000000", b"old delivery bytes"),
    ]
}

fn read_only(terminal: &Terminal) {
    assert!(terminal.requests.iter().all(|kind| matches!(kind, Kind::Load | Kind::Scan)), "{:?}", terminal.requests);
}

fn unchanged(run: &Run<'_>, index: usize) {
    let before = if index == 0 { &run.before } else { &run.after[index - 1] };
    assert_eq!(&run.after[index], before);
    assert_eq!(&run.closed, before);
}

fn refused(run: &Run<'_>, index: usize, why: Option<Older>) {
    let terminal = run.terminal(index);
    let local::Event::StoreFailed { turn: None, reason } = &terminal.event else {
        panic!("refusal: {:?}", terminal.event);
    };
    let expected = match why {
        Some(why) => local::StoreFailure::OlderBuild { why },
        None => local::StoreFailure::Read,
    };
    assert_eq!(*reason, expected);
    let requests: &[Kind] = match why {
        Some(Older::Unrecorded) => &[Kind::Load, Kind::Scan],
        Some(Older::Layout(_)) | None => &[Kind::Load],
    };
    assert_eq!(terminal.requests, requests);
    assert!(terminal.starts_with_marker_load);
    read_only(terminal);
    unchanged(run, index);
}

#[test]
fn empty_create_is_durable_and_a_new_store_reopens_without_mutation() {
    let disk = Fixture::new(&[]);
    let create = disk.run(|| [StoreRequest::Load]);
    let terminal = create.terminal(0);
    empty(&terminal.event);
    assert_eq!(terminal.requests, [Kind::Load, Kind::Scan, Kind::Store]);
    assert_eq!(marker_entry(&create.after[0]).bytes.as_slice(), marker_bytes().as_ref());
    assert_eq!(create.closed, create.after[0]);

    let reopened = disk.run(|| [StoreRequest::Load, StoreRequest::Load]);
    assert_eq!(reopened.before, create.closed);
    let first = reopened.terminal(0);
    empty(&first.event);
    read_only(first);
    assert_eq!(reopened.after[0], reopened.before);
    let checked = reopened.terminal(1);
    empty(&checked.event);
    // The same owned root skips only the marker load.
    assert_eq!(checked.requests, [Kind::Load, Kind::Load, Kind::Scan]);
    unchanged(&reopened, 1);
}

#[test]
fn every_unrecorded_old_record_is_refused_before_any_mutation() {
    for name in [
        "state",
        "fresh",
        "0000000001.turn",
        "00000000000000000001.0000000001.0000000000",
        "foreign",
        "turn-00000001",
        "delivery",
        "unrelated",
    ] {
        let disk = Fixture::new(&[Item::file(name.as_bytes(), b"retained bytes")]);
        refused(&disk.run(|| [StoreRequest::Load]), 0, Some(Older::Unrecorded));
    }
}

#[test]
fn mismatches_precede_valid_fresh_journal_filename_deletion() {
    for (store, transcript, family) in
        [(true, false, StoreFamily::Store), (false, true, StoreFamily::Transcript), (true, true, StoreFamily::Store)]
    {
        for position in 0..32 {
            let mut items = old_records();
            items.push(Item::file(b"format", &mismatched_marker(store, transcript, position)));
            let disk = Fixture::new(&items);
            refused(&disk.run(|| [StoreRequest::Load]), 0, Some(Older::Layout(family)));
        }
    }
}

#[test]
fn malformed_truncated_wrong_version_and_oversized_markers_refuse_unchanged() {
    let valid = marker_bytes();
    let mut wrong = valid.to_vec();
    wrong[1] = 2;
    let mut trailing = valid.to_vec();
    trailing.extend_from_slice(b"trailing record");
    let mut markers = vec![b"not a marker".to_vec(), wrong, vec![0; 67], trailing];
    for prefix in 0..valid.len() {
        markers.push(valid[..prefix].to_vec());
    }
    for bytes in markers {
        let oversized = bytes.len() > usize::try_from(format::max_bytes()).unwrap();
        let mut items = old_records();
        items.push(Item::file(b"format", &bytes));
        let disk = Fixture::new(&items);
        refused(&disk.run(|| [StoreRequest::Load]), 0, (!oversized).then_some(Older::Layout(StoreFamily::Store)));
    }
}

#[test]
fn marker_symlinks_and_directories_are_io_refusals_and_stay_untouched() {
    for symlink in [false, true] {
        let mut items = old_records();
        if symlink {
            items.push(Item::file(b"valid-target", &marker_bytes()));
            items.push(Item::link(b"format", b"valid-target"));
        } else {
            items.push(Item::directory(b"format"));
        }
        let disk = Fixture::new(&items);
        refused(&disk.run(|| [StoreRequest::Load]), 0, None);
    }
}

#[test]
fn near_miss_and_nonregular_reserved_residue_refuses_unchanged() {
    for name in [
        "format.skein-0123456789abcdeF",
        "format.skein-0123456789abcde",
        "format.skein-0123456789abcdef0",
        "format.skein-0123456789abcdeg",
        "format.skein-0123456789abcdef.extra",
        "state.skein-0123456789abcdef",
        "xformat.skein-0123456789abcdef",
        ".format.skein-0123456789abcdef",
    ] {
        let disk = Fixture::new(&[Item::file(name.as_bytes(), b"unknown")]);
        refused(&disk.run(|| [StoreRequest::Load]), 0, Some(Older::Unrecorded));
    }
    for items in [
        vec![Item::link(RESIDUE, b"missing")],
        vec![Item::directory(RESIDUE)],
        vec![
            Item::file(RESIDUE, b"partial"),
            Item::file(b"format.skein-fedcba9876543210", b"partial"),
            Item::file(b"fresh", b"old pending recovery"),
        ],
    ] {
        let disk = Fixture::new(&items);
        refused(&disk.run(|| [StoreRequest::Load]), 0, Some(Older::Unrecorded));
    }
}

#[test]
fn selecting_a_different_root_resets_the_check() {
    for mismatched in [false, true] {
        let mut items = vec![
            Item::directory(b"current"),
            Item::file(b"current/format", &marker_bytes()),
            Item::directory(b"older"),
            Item::file(b"older/fresh", &save_state(state(9)).bytes),
        ];
        if mismatched {
            items.push(Item::file(b"older/format", &mismatched_marker(false, true, 0)));
        }
        let disk = Fixture::new(&items).watch_child(b"older");
        let run = disk.run(|| {
            [
                StoreRequest::SelectChat { name: b"current".as_slice().into(), create: false },
                StoreRequest::Load,
                StoreRequest::SaveState { state: state(1), history: History::Kept },
                StoreRequest::SelectChat { name: b"older".as_slice().into(), create: false },
                StoreRequest::Load,
            ]
        });
        assert!(matches!(run.terminal(0).event, local::Event::ChatSelected { result: Ok(()), .. }));
        empty(&run.terminal(1).event);
        assert!(matches!(run.terminal(2).event, local::Event::StateSaved));
        assert!(matches!(run.terminal(3).event, local::Event::ChatSelected { result: Ok(()), .. }));
        let expected = if mismatched { Older::Layout(StoreFamily::Transcript) } else { Older::Unrecorded };
        refused(&run, 4, Some(expected));
    }
}

#[test]
fn a_checked_current_store_saves_and_reopens_a_real_turn_with_its_read_fence() {
    fn turn() -> smith_domain::Turn {
        smith_domain::Turn {
            endpoint: Some(smith_domain::session::llm::Endpoint(2)),
            ..smith_local_domain_fixtures::turn()
        }
    }

    let disk = Fixture::new(&[]);
    let written = disk.run(|| {
        [
            StoreRequest::Load,
            StoreRequest::SaveState { state: state(1), history: History::Kept },
            StoreRequest::SaveTurn { number: 1, read: Some(Token::new(1)), opens_window: true, turn: turn() },
        ]
    });
    empty(&written.terminal(0).event);
    assert!(matches!(written.terminal(1).event, local::Event::StateSaved));
    assert!(matches!(written.terminal(2).event, local::Event::TurnSaved { number: 1 }));

    let reopened = disk.run(|| [StoreRequest::Load]);
    let terminal = reopened.terminal(0);
    let local::Event::Loaded { state: Some(loaded), transcript: Some(transcript), deliveries } = &terminal.event else {
        panic!("current marked turn reload: {:?}", terminal.event);
    };
    assert_eq!(loaded.read, Some(Token::new(1)));
    assert_eq!(transcript.value().turns.as_ref(), &[turn()]);
    assert!(deliveries.is_empty());
    unchanged(&reopened, 0);
    read_only(terminal);
}

#[test]
fn marker_at_file_bound_is_adopted() {
    let bounds = smith_local_protocol::StoreLimits { file_bytes: format::max_bytes(), ..LIMITS };
    let disk = Fixture::bounded(&[Item::file(b"format", &marker_bytes())], bounds);
    let run = disk.run(|| [StoreRequest::Load]);
    empty(&run.terminal(0).event);
    unchanged(&run, 0);
}

#[test]
fn compatible_marker_loads_the_exact_public_state() {
    let expected = state(23);
    let disk =
        Fixture::new(&[Item::file(b"format", &marker_bytes()), Item::file(b"state", &save_state(expected).bytes)]);
    let run = disk.run(|| [StoreRequest::Load]);
    loaded_state(&run.terminal(0).event, expected);
    unchanged(&run, 0);
}

#[test]
fn repeated_load_refreshes_state_on_the_checked_root() {
    let disk = Fixture::new(&[Item::file(b"format", &marker_bytes())]);
    let expected = state(29);
    let mut planted = None;
    let run = disk.run_with(
        || [StoreRequest::Load, StoreRequest::Load],
        |completed| {
            if completed == 1 {
                disk.create(b"state", &save_state(expected).bytes);
                planted = Some(disk.snapshot());
            }
        },
    );
    empty(&run.terminal(0).event);
    assert_eq!(run.before, run.after[0]);
    loaded_state(&run.terminal(1).event, expected);
    assert_eq!(run.after[1], planted.unwrap());
    assert_eq!(run.closed, run.after[1]);
}

#[test]
fn reopening_a_checked_root_rechecks_its_marker() {
    let disk = Fixture::new(&[Item::file(b"format", &marker_bytes())]);
    let checked = disk.run_with(
        || [StoreRequest::Load],
        |_| {
            disk.remove(b"format");
            disk.create(b"state", &save_state(state(31)).bytes);
        },
    );
    empty(&checked.terminal(0).event);
    let reopened = disk.run(|| [StoreRequest::Load]);
    refused(&reopened, 0, Some(Older::Unrecorded));
}

#[test]
fn fresh_history_keeps_the_original_layout_marker() {
    let initial = state(37);
    let disk =
        Fixture::new(&[Item::file(b"format", &marker_bytes()), Item::file(b"state", &save_state(initial).bytes)]);
    let expected = state(41);
    let saved = disk.run(|| [StoreRequest::Load, StoreRequest::SaveState { state: expected, history: History::Fresh }]);
    let marker = marker_entry(&saved.before);
    loaded_state(&saved.terminal(0).event, initial);
    assert!(matches!(saved.terminal(1).event, local::Event::StateSaved));
    for snapshot in [&saved.after[0], &saved.after[1], &saved.closed] {
        assert_eq!(marker_entry(snapshot), marker);
    }
    let reopened = disk.run(|| [StoreRequest::Load]);
    loaded_state(&reopened.terminal(0).event, expected);
    for snapshot in [&reopened.before, &reopened.after[0], &reopened.closed] {
        assert_eq!(marker_entry(snapshot), marker);
    }
}

#[test]
fn interrupted_create_reserves_the_marker_entry_at_both_scan_bounds() {
    for (count, admitted) in [(7_u32, true), (8_u32, false)] {
        let items: Vec<_> =
            (0..count).map(|index| Item::file(format!("format.skein-{index:016x}").as_bytes(), b"partial")).collect();
        reserved_entry(&Fixture::new(&items), admitted);
    }
    let entry = u64::try_from(size_of::<skein_io::file::Entry>()).unwrap();
    for (scan_bytes, admitted) in [(2 * entry + 35, true), (2 * entry + 34, false)] {
        let bounds = smith_local_protocol::StoreLimits { scan_bytes, ..LIMITS };
        reserved_entry(&Fixture::bounded(&[Item::file(RESIDUE, b"partial")], bounds), admitted);
    }
}

fn reserved_entry(disk: &Fixture, admitted: bool) {
    let run = disk.run(|| [StoreRequest::Load]);
    if admitted {
        empty(&run.terminal(0).event);
        let after_create = &run.after[0];
        let marker = marker_entry(after_create);
        assert_eq!(marker.kind, Is::File);
        assert_eq!(marker.bytes.as_slice(), marker_bytes().as_ref());
        let retained: Vec<_> = after_create.root.iter().filter(|entry| entry.name.as_ref() != b"format").collect();
        assert_eq!(retained, run.before.root.iter().collect::<Vec<_>>());
        assert_eq!(&run.closed, after_create);
        let reopened = disk.run(|| [StoreRequest::Load]);
        empty(&reopened.terminal(0).event);
        assert_eq!(&reopened.before, after_create);
        unchanged(&reopened, 0);
    } else {
        refused(&run, 0, Some(Older::Unrecorded));
        assert!(run.closed.root.iter().all(|entry| entry.name.as_ref() != b"format"));
    }
}
