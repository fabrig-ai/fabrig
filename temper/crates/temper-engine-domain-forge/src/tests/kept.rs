#![expect(clippy::disallowed_macros, reason = "boundary tests allocate byte strings at and past their limits")]
use super::*;
use crate::kept;
use crate::reconcile::{MissingLink, Need};

fn forge(number: u16) -> ForgeRecord {
    ForgeRecord {
        number,
        kind: Kind::Forgejo,
        mode: Mode::Fake,
        removing: false,
        definition: Box::from(b"{}".as_slice()),
        first: None,
    }
}
fn playground() -> Playground {
    Playground {
        state: PlaygroundState::Resetting(ResetStep::Forge),
        forge: Some(5),
        project: Some(7),
        account: Some(9),
    }
}
fn act(key: u8, forge: u16) -> KeptAct {
    KeptAct { key: [key; 16], forge, act: KeptActKind::Cancel { project: 7, task: 8 } }
}
fn restore(d: &mut Domain, row: Stored) {
    assert!(outputs(d, Event::Restore { record: row }).is_empty());
}

#[test]
fn restored_forge_rows_are_held_and_numbers_follow_the_largest() {
    for numbers_first in [false, true] {
        let mut d = domain();
        assert_eq!(d.forge_next(), 1);
        if numbers_first {
            restore(&mut d, Stored::ForgeNumbers { next: 3 });
        }
        restore(&mut d, Stored::Forge(forge(5)));
        restore(&mut d, Stored::Forge(forge(2)));
        if !numbers_first {
            restore(&mut d, Stored::ForgeNumbers { next: 3 });
        }
        assert_eq!(d.forge_record(5), Some(&forge(5)));
        assert_eq!(d.forge_records().as_ref(), &[forge(2), forge(5)]);
        assert_eq!(d.forge_next(), 6);
        restore(&mut d, Stored::ForgeNumbers { next: 20 });
        restore(&mut d, Stored::ForgeNumbers { next: 2 });
        assert_eq!(d.forge_next(), 20);
        assert_eq!(d.restore_fit_failure(), None);
    }
}

#[test]
fn restored_playground_and_kept_acts_are_read_back() {
    let mut d = domain();
    assert_eq!(d.playground(), None);
    restore(&mut d, Stored::Playground(playground()));
    restore(&mut d, Stored::KeptAct(act(3, 5)));
    restore(&mut d, Stored::KeptAct(act(1, 5)));
    restore(&mut d, Stored::Forge(forge(5)));
    assert_eq!(d.playground(), Some(&playground()));
    assert_eq!(d.kept_acts().as_ref(), &[act(1, 5), act(3, 5)]);
    let tasks = reconcile::empty_tasks();
    assert_eq!(d.missing_link(&tasks), None);
}

#[test]
fn a_definition_past_its_bound_is_refused() {
    for extra in [0, 1] {
        let mut d = domain();
        let mut row = forge(1);
        row.definition = vec![0; usize::try_from(LIMITS.definition_bytes).unwrap() + extra].into_boxed_slice();
        restore(&mut d, Stored::Forge(row.clone()));
        assert_eq!(d.restore_fit_failure().is_some(), extra == 1);
        assert_eq!(d.forge_record(1).is_some(), extra == 0);
        let mut d = domain();
        let mut out = Queue::uncapped();
        assert_eq!(kept::save_forge(&mut d, &LIMITS, row, &mut out), if extra == 0 { Ok(()) } else { Err(Full) });
        assert_eq!(out.len(), u32::from(extra == 0));
    }
}

#[test]
fn a_kept_act_without_its_forge_is_dangling() {
    let mut d = domain();
    // Forge 1 is configured through the identity, but no durable record exists.
    restore(&mut d, Stored::KeptAct(act(3, 1)));
    restore(&mut d, Stored::KeptAct(act(1, 5)));
    restore(&mut d, Stored::Forge(forge(5)));
    let tasks = reconcile::empty_tasks();
    assert_eq!(
        d.missing_link(&tasks),
        Some(MissingLink { kind: RecordKind::KeptAct, key: Some(Key::KeptAct([3; 16])), need: Need::Forge(1) })
    );
    restore(&mut d, Stored::Forge(forge(1)));
    assert_eq!(d.missing_link(&tasks), None);
}

#[test]
fn retained_forge_and_act_tables_replace_at_capacity_and_refuse_an_extra_row() {
    for family in [RecordKind::Forge, RecordKind::KeptAct] {
        let mut d = domain();
        let limit = if family == RecordKind::Forge { LIMITS.forges } else { kept_act_bound(&LIMITS).unwrap() };
        for number in 1..=limit {
            let row = if family == RecordKind::Forge {
                Stored::Forge(forge(u16::try_from(number).unwrap()))
            } else {
                Stored::KeptAct(act(u8::try_from(number).unwrap(), 1))
            };
            restore(&mut d, row.clone());
            restore(&mut d, row);
        }
        assert_eq!(d.restore_fit_failure(), None);
        let row = if family == RecordKind::Forge {
            Stored::Forge(forge(u16::try_from(limit + 1).unwrap()))
        } else {
            Stored::KeptAct(act(u8::try_from(limit + 1).unwrap(), 1))
        };
        restore(&mut d, row);
        assert_eq!(
            d.restore_fit_failure(),
            Some(&RestoreFit {
                family: Some(family),
                set: RestoreSet::Rows(family),
                held: u64::from(limit),
                additional: 1,
                limit: u64::from(limit),
            })
        );
    }
}

#[test]
fn first_intent_names_are_bounded_for_restore_and_save() {
    for field in 0_u8..4 {
        for extra in [0, 1] {
            let mut row = forge(1);
            let name = vec![0; usize::try_from(FIRST_NAME_BYTES).unwrap() + extra].into_boxed_slice();
            let mut first = FirstIntent {
                playground: true,
                org: Box::new([]),
                repository: Box::new([]),
                project: SetUpProject::Existing(1),
                label: Box::new([]),
            };
            match field {
                0 => first.org = name,
                1 => first.repository = name,
                2 => first.label = name,
                3 => first.project = SetUpProject::New(name),
                _ => unreachable!(),
            }
            row.first = Some(first);
            let mut d = domain();
            restore(&mut d, Stored::Forge(row.clone()));
            assert_eq!(d.restore_fit_failure().is_some(), extra == 1);
            let mut d = domain();
            let mut out = Queue::uncapped();
            assert_eq!(kept::save_forge(&mut d, &LIMITS, row, &mut out).is_ok(), extra == 0);
        }
    }
}

#[test]
fn malformed_numbers_are_refused_and_exhaustion_saves_nothing() {
    for row in [Stored::Forge(forge(0)), Stored::Forge(forge(u16::MAX)), Stored::ForgeNumbers { next: 0 }] {
        let mut d = domain();
        restore(&mut d, row);
        assert!(d.restore_fit_failure().is_some());
        assert!(d.forge_records().is_empty());
        assert_eq!(d.forge_next(), 1);
    }
    let mut d = domain();
    restore(&mut d, Stored::Forge(forge(u16::MAX - 1)));
    assert_eq!(d.forge_next(), u16::MAX);
    let mut out = Queue::uncapped();
    assert_eq!(kept::give_number(&mut d, &mut out), None);
    assert!(out.is_empty());
}

#[test]
fn request_makers_save_and_erase_their_held_rows() {
    let mut d = domain();
    let mut out = Queue::uncapped();
    assert_eq!(kept::give_number(&mut d, &mut out), Some(1));
    assert_eq!(
        out.pop(),
        Some(Request::Save { read_afresh: None, release: None, record: Stored::ForgeNumbers { next: 2 } })
    );
    kept::save_forge(&mut d, &LIMITS, forge(1), &mut out).unwrap();
    kept::save_playground(&mut d, playground(), &mut out);
    kept::keep_act(&mut d, act(1, 1), &mut out).unwrap();
    for row in [Stored::Forge(forge(1)), Stored::Playground(playground()), Stored::KeptAct(act(1, 1))] {
        assert_eq!(out.pop(), Some(Request::Save { read_afresh: None, release: None, record: row }));
    }
    assert_eq!(d.forge_record(1), Some(&forge(1)));
    assert_eq!(d.playground(), Some(&playground()));
    assert_eq!(d.kept_acts().as_ref(), &[act(1, 1)]);
    kept::erase_act(&mut d, [1; 16], &mut out);
    kept::erase_playground(&mut d, &mut out);
    kept::erase_forge(&mut d, 1, &mut out);
    for key in [Key::KeptAct([1; 16]), Key::Playground, Key::Forge(1)] {
        assert_eq!(out.pop(), Some(Request::Erase { key }));
    }
    assert!(d.forge_records().is_empty());
    assert!(d.kept_acts().is_empty());
    assert_eq!(d.playground(), None);
    assert_eq!(d.forge_next(), 2);
}

#[test]
fn save_helpers_refuse_full_tables_without_emitting_saves() {
    let mut d = domain();
    let mut out = Queue::uncapped();
    for n in 1..=LIMITS.forges {
        kept::save_forge(&mut d, &LIMITS, forge(u16::try_from(n).unwrap()), &mut out).unwrap();
    }
    assert_eq!(kept::save_forge(&mut d, &LIMITS, forge(1), &mut out), Ok(()));
    let before = out.len();
    assert_eq!(kept::save_forge(&mut d, &LIMITS, forge(100), &mut out), Err(Full));
    assert_eq!(out.len(), before);
    for n in 1..=kept_act_bound(&LIMITS).unwrap() {
        kept::keep_act(&mut d, act(u8::try_from(n).unwrap(), 1), &mut out).unwrap();
    }
    assert_eq!(kept::keep_act(&mut d, act(1, 1), &mut out), Ok(()));
    let before = out.len();
    assert_eq!(kept::keep_act(&mut d, act(100, 1), &mut out), Err(Full));
    assert_eq!(out.len(), before);
}

#[test]
fn new_limits_validate_their_bounds_and_capacity_arithmetic() {
    for (forges, definition_bytes, admitted) in
        [(0, 8192, false), (1, 255, false), (1, 256, true), (u32::from(u16::MAX), 8192, true), (65_536, 8192, false)]
    {
        assert_eq!(valid(&Limits { forges, definition_bytes, ..LIMITS }), admitted);
    }
    assert_eq!(kept_act_bound(&Limits { tasks: u32::MAX, ..LIMITS }), None);
}

#[test]
fn singleton_rows_replace_and_the_number_remains_monotone() {
    let mut d = domain();
    restore(&mut d, Stored::Playground(playground()));
    let running = Playground { state: PlaygroundState::Running, forge: None, project: None, account: None };
    restore(&mut d, Stored::Playground(running));
    assert_eq!(d.playground(), Some(&running));
    restore(&mut d, Stored::ForgeNumbers { next: u16::MAX });
    restore(&mut d, Stored::ForgeNumbers { next: 1 });
    assert_eq!(d.forge_next(), u16::MAX);
    assert_eq!(d.restore_fit_failure(), None);
}

#[test]
fn stored_forge_bytes_include_definition_and_each_first_intent_name() {
    let mut row = forge(1);
    assert_eq!(stored_bytes(&Stored::Forge(row.clone())), Some(2));
    row.first = Some(FirstIntent {
        playground: true,
        org: Box::new([1; 3]),
        repository: Box::new([2; 4]),
        project: SetUpProject::New(Box::new([3; 5])),
        label: Box::new([4; 6]),
    });
    assert_eq!(stored_bytes(&Stored::Forge(row.clone())), Some(20));
    row.first.as_mut().unwrap().project = SetUpProject::Existing(7);
    assert_eq!(stored_bytes(&Stored::Forge(row)), Some(15));
}
