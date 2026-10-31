#![expect(clippy::disallowed_macros, reason = "boundary fixtures allocate names at their byte limits")]
use super::*;
use crate::kept;
use crate::reconcile::{MissingLink, Need};

fn forge(number: u16) -> ForgeRecord {
    ForgeRecord {
        number,
        kind: Kind::Forgejo,
        mode: Mode::Fake,
        removing: false,
        definition: b"{}".as_slice().into(),
        first: None,
    }
}
fn playground() -> Playground {
    Playground {
        state: PlaygroundState::Resetting(ResetStep::Project),
        forge: Some(5),
        project: Some(7),
        account: Some(9),
    }
}
fn act(key: u8, forge: u16) -> KeptAct {
    KeptAct { key: [key; 16], forge, act: KeptActKind::Cancel { project: 7, task: 11 } }
}
fn restore(d: &mut Domain, row: Stored) {
    assert!(outputs(d, Event::Restore { record: row }).is_empty());
}

#[test]
fn restored_forge_rows_are_held_and_numbers_follow_the_largest() {
    for numbers_first in [false, true] {
        let mut d = domain();
        if numbers_first {
            restore(&mut d, Stored::ForgeNumbers { next: 3 });
        }
        restore(&mut d, Stored::Forge(forge(5)));
        restore(&mut d, Stored::Forge(forge(2)));
        if !numbers_first {
            restore(&mut d, Stored::ForgeNumbers { next: 3 });
        }
        assert_eq!(d.forge_record(5), Some(&forge(5)));
        assert_eq!(d.forge_records().as_ref(), &[&forge(2), &forge(5)]);
        assert_eq!(d.forge_next(), 6);
        assert_eq!(d.restore_fit_failure(), None);
        restore(&mut d, Stored::ForgeNumbers { next: 10 });
        assert_eq!(d.forge_next(), 10);
    }
}
#[test]
fn restored_playground_and_kept_acts_are_read_back() {
    let mut d = domain();
    restore(&mut d, Stored::Playground(playground()));
    restore(&mut d, Stored::KeptAct(act(2, 5)));
    restore(&mut d, Stored::KeptAct(act(1, 5)));
    restore(&mut d, Stored::Forge(forge(5)));
    assert_eq!(d.playground(), Some(&playground()));
    assert_eq!(d.kept_acts().as_ref(), &[&act(1, 5), &act(2, 5)]);
    assert_eq!(d.restore_fit_failure(), None);
    assert_eq!(d.missing_link(&reconcile::empty_tasks()), None);
}
#[test]
fn a_definition_at_its_bound_is_held_and_saved() {
    let mut row = forge(5);
    row.definition = vec![b'x'; usize::try_from(LIMITS.definition_bytes).unwrap()].into_boxed_slice();
    let mut d = domain();
    restore(&mut d, Stored::Forge(row.clone()));
    assert_eq!(d.forge_record(5), Some(&row));
    let mut out = Queue::uncapped();
    assert_eq!(kept::save_forge(&mut d, &LIMITS, row.clone(), &mut out), Ok(()));
    assert_eq!(out.pop(), Some(Request::Save { record: Stored::Forge(row), read_afresh: None, release: None }));
}
#[test]
fn a_definition_past_its_bound_is_refused() {
    let mut row = forge(5);
    row.definition = vec![b'x'; usize::try_from(LIMITS.definition_bytes + 1).unwrap()].into_boxed_slice();
    let mut d = domain();
    restore(&mut d, Stored::Forge(row.clone()));
    assert_eq!(
        d.restore_fit_failure(),
        Some(&RestoreFit {
            family: Some(RecordKind::Forge),
            set: RestoreSet::DefinitionBytes,
            held: u64::from(LIMITS.definition_bytes + 1),
            additional: 0,
            limit: u64::from(LIMITS.definition_bytes)
        })
    );
    assert_eq!(d.forge_record(5), None);
    let mut out = Queue::uncapped();
    assert_eq!(kept::save_forge(&mut d, &LIMITS, row, &mut out), Err(Full));
    assert!(out.is_empty());
}
#[test]
fn a_kept_act_without_its_forge_is_dangling() {
    let mut d = domain();
    restore(&mut d, Stored::KeptAct(act(2, 1)));
    restore(&mut d, Stored::KeptAct(act(1, 2)));
    restore(&mut d, Stored::KeptAct(act(1, 2)));
    assert_eq!(
        d.missing_link(&reconcile::empty_tasks()),
        Some(MissingLink { kind: RecordKind::KeptAct, key: Some(Key::KeptAct([2; 16])), need: Need::Forge(1) })
    );
}
#[test]
fn new_tables_replace_at_capacity_and_refuse_another_key() {
    for (kind, limit) in [(RecordKind::Forge, LIMITS.forges), (RecordKind::KeptAct, LIMITS.tasks + 2 * LIMITS.forges)] {
        let mut d = domain();
        for number in 1..=limit {
            let row = if kind == RecordKind::Forge {
                Stored::Forge(forge(u16::try_from(number).unwrap()))
            } else {
                Stored::KeptAct(act(u8::try_from(number).unwrap(), 1))
            };
            restore(&mut d, row.clone());
            restore(&mut d, row);
        }
        assert_eq!(d.restore_fit_failure(), None);
        let row = if kind == RecordKind::Forge {
            Stored::Forge(forge(u16::try_from(limit + 1).unwrap()))
        } else {
            Stored::KeptAct(act(u8::try_from(limit + 1).unwrap(), 1))
        };
        restore(&mut d, row);
        assert_eq!(
            d.restore_fit_failure(),
            Some(&RestoreFit {
                family: Some(kind),
                set: RestoreSet::Rows(kind),
                held: u64::from(limit),
                additional: 1,
                limit: u64::from(limit)
            })
        );
    }
}
#[test]
fn singleton_rows_replace_without_using_another_slot() {
    let mut d = domain();
    for next in [2, 4] {
        restore(&mut d, Stored::ForgeNumbers { next });
    }
    for state in [PlaygroundState::Running, PlaygroundState::Resetting(ResetStep::Account)] {
        restore(&mut d, Stored::Playground(Playground { state, ..playground() }));
    }
    assert_eq!(d.restore_fit_failure(), None);
    assert_eq!(d.forge_next(), 4);
    assert_eq!(d.playground().unwrap().state, PlaygroundState::Resetting(ResetStep::Account));
}
fn first(bytes: u32, field: u8) -> FirstIntent {
    let name = vec![b'x'; usize::try_from(bytes).unwrap()].into_boxed_slice();
    let mut row = FirstIntent {
        playground: false,
        org: Box::new([]),
        repository: Box::new([]),
        project: SetUpProject::Existing(7),
        label: Box::new([]),
    };
    match field {
        0 => row.org = name,
        1 => row.repository = name,
        2 => row.label = name,
        _ => row.project = SetUpProject::New(name),
    }
    row
}
#[test]
fn first_intent_names_fit_at_their_bound() {
    for field in 0..4 {
        let row = ForgeRecord { first: Some(first(FIRST_NAME_BYTES, field)), ..forge(5) };
        let mut d = domain();
        restore(&mut d, Stored::Forge(row.clone()));
        assert_eq!(d.forge_record(5), Some(&row));
        let mut out = Queue::uncapped();
        assert_eq!(kept::save_forge(&mut d, &LIMITS, row, &mut out), Ok(()));
    }
}
#[test]
fn first_intent_names_past_their_bound_are_refused() {
    for field in 0..4 {
        let row = ForgeRecord { first: Some(first(FIRST_NAME_BYTES + 1, field)), ..forge(5) };
        let mut d = domain();
        restore(&mut d, Stored::Forge(row.clone()));
        assert_eq!(d.restore_fit_failure().unwrap().set, RestoreSet::FirstNameBytes);
        assert_eq!(d.forge_record(5), None);
        let mut out = Queue::uncapped();
        assert_eq!(kept::save_forge(&mut d, &LIMITS, row, &mut out), Err(Full));
        assert!(out.is_empty());
    }
}
#[test]
fn malformed_forge_numbers_are_refused_before_insertion() {
    for row in [Stored::Forge(forge(0)), Stored::Forge(forge(u16::MAX)), Stored::ForgeNumbers { next: 0 }] {
        let mut d = domain();
        restore(&mut d, row);
        assert!(d.restore_fit_failure().is_some());
        assert!(d.forge_records().is_empty());
        assert_eq!(d.forge_next(), 1);
    }
}
#[test]
fn number_requests_are_monotone_and_stop_at_exhaustion() {
    let mut d = domain();
    let mut out = Queue::uncapped();
    assert_eq!(kept::give_number(&mut d, &mut out), Some(1));
    assert_eq!(
        out.pop(),
        Some(Request::Save { record: Stored::ForgeNumbers { next: 2 }, read_afresh: None, release: None })
    );
    restore(&mut d, Stored::Forge(forge(u16::MAX - 1)));
    assert_eq!(d.forge_next(), u16::MAX);
    assert_eq!(kept::give_number(&mut d, &mut out), None);
    assert!(out.is_empty());
}
#[test]
fn request_makers_hold_and_erase_their_rows() {
    let mut d = domain();
    let mut out = Queue::uncapped();
    kept::save_forge(&mut d, &LIMITS, forge(5), &mut out).unwrap();
    kept::save_playground(&mut d, playground(), &mut out);
    kept::keep_act(&mut d, act(1, 5), &mut out).unwrap();
    assert_eq!(d.forge_record(5), Some(&forge(5)));
    assert_eq!(d.playground(), Some(&playground()));
    assert_eq!(d.kept_acts().as_ref(), &[&act(1, 5)]);
    kept::erase_act(&mut d, [1; 16], &mut out);
    kept::erase_playground(&mut d, &mut out);
    kept::erase_forge(&mut d, 5, &mut out);
    assert!(d.kept_acts().is_empty());
    assert_eq!(d.playground(), None);
    assert_eq!(d.forge_record(5), None);
    let expected = [
        &Request::Save { record: Stored::Forge(forge(5)), read_afresh: None, release: None },
        &Request::Save { record: Stored::Playground(playground()), read_afresh: None, release: None },
        &Request::Save { record: Stored::KeptAct(act(1, 5)), read_afresh: None, release: None },
        &Request::Erase { key: Key::KeptAct([1; 16]) },
        &Request::Erase { key: Key::Playground },
        &Request::Erase { key: Key::Forge(5) },
    ];
    assert_eq!(out.len(), u32::try_from(expected.len()).unwrap());
    for request in expected {
        assert_eq!(out.pop().as_ref(), Some(request));
    }
}
#[test]
fn forge_limits_reject_outside_their_bounds() {
    for limits in [
        Limits { forges: 0, ..LIMITS },
        Limits { forges: u32::from(u16::MAX) + 1, ..LIMITS },
        Limits { definition_bytes: 255, ..LIMITS },
    ] {
        assert!(!valid(&limits));
    }
    for limits in
        [Limits { forges: 1, definition_bytes: 256, ..LIMITS }, Limits { forges: u32::from(u16::MAX), ..LIMITS }]
    {
        assert!(valid(&limits));
    }
}

#[test]
fn forge_byte_charges_include_definition_and_first_intent_names() {
    let first = FirstIntent {
        playground: false,
        org: b"org".as_slice().into(),
        repository: b"repo".as_slice().into(),
        label: b"label".as_slice().into(),
        project: SetUpProject::New(b"project".as_slice().into()),
    };
    // Two definition bytes, then 3 org + 4 repository + 5 label + 7 project.
    for (first, bytes) in
        [(None, 2), (Some(first.clone()), 21), (Some(FirstIntent { project: SetUpProject::Existing(7), ..first }), 14)]
    {
        assert_eq!(stored_bytes(&Stored::Forge(ForgeRecord { first, ..forge(5) })), Some(bytes));
    }
    for row in [Stored::ForgeNumbers { next: 1 }, Stored::Playground(playground()), Stored::KeptAct(act(1, 5))] {
        assert_eq!(stored_bytes(&row), Some(0));
    }
}
#[test]
fn full_kept_maps_refuse_save_requests_without_mutation() {
    let mut d = domain();
    let mut out = Queue::uncapped();
    for number in 1..=LIMITS.forges {
        kept::save_forge(&mut d, &LIMITS, forge(u16::try_from(number).unwrap()), &mut out).unwrap();
    }
    for number in 1..=crate::limits::kept_act_bound(&LIMITS).unwrap() {
        kept::keep_act(&mut d, act(u8::try_from(number).unwrap(), 1), &mut out).unwrap();
    }
    let held_requests = out.len();
    kept::save_forge(&mut d, &LIMITS, forge(1), &mut out).unwrap();
    kept::keep_act(&mut d, act(1, 1), &mut out).unwrap();
    assert_eq!(out.len(), held_requests + 2);
    assert_eq!(
        kept::save_forge(&mut d, &LIMITS, forge(u16::try_from(LIMITS.forges + 1).unwrap()), &mut out),
        Err(Full)
    );
    assert_eq!(
        kept::keep_act(
            &mut d,
            act(u8::try_from(crate::limits::kept_act_bound(&LIMITS).unwrap() + 1).unwrap(), 1),
            &mut out
        ),
        Err(Full)
    );
    assert_eq!(out.len(), held_requests + 2);
    assert_eq!(d.forge_records().len(), usize::try_from(LIMITS.forges).unwrap());
    assert_eq!(d.kept_acts().len(), usize::try_from(crate::limits::kept_act_bound(&LIMITS).unwrap()).unwrap());
}
