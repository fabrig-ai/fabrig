use super::*;

const EARLIER_PARTY: u64 = 0;
const PARTY: u64 = 7;
const OTHER_PARTY: u64 = 8;
// Four member bindings times two projects (the shared LIMITS fixture).
const DISMISSALS: u32 = 8;

fn row(party: u64, project: u32) -> Stored {
    Stored::Dismissed { party, what: Dismissal::ToBegin { project } }
}
fn restore_rows(count: u32) -> Domain {
    let mut d = domain();
    for project in 0..count {
        outputs(&mut d, Event::Restore { record: row(PARTY, project) });
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    d
}

#[test]
fn dismissals_are_the_partys_own_in_key_order() {
    let mut d = domain();
    for record in [row(OTHER_PARTY, 1), row(PARTY, u32::MAX), row(EARLIER_PARTY, 4), row(PARTY, 0), row(PARTY, 3)] {
        outputs(&mut d, Event::Restore { record });
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    for (party, expected) in [
        (
            PARTY,
            Box::<[Dismissal]>::from([
                Dismissal::ToBegin { project: 0 },
                Dismissal::ToBegin { project: 3 },
                Dismissal::ToBegin { project: u32::MAX },
            ]),
        ),
        (OTHER_PARTY, Box::<[Dismissal]>::from([Dismissal::ToBegin { project: 1 }])),
        (u64::MAX, Box::<[Dismissal]>::from([])),
    ] {
        assert_eq!(d.dismissals(party).as_ref(), expected.as_ref());
    }
}

#[test]
fn dismissals_restore_at_the_bound() {
    let d = restore_rows(DISMISSALS);
    assert_eq!(LIMITS.dismissed(), DISMISSALS);
    assert_eq!(d.restore_fit_failure(), None);
    let mut expected = skein_lib::List::with_capacity(DISMISSALS);
    for project in 0..DISMISSALS {
        expected.push(Dismissal::ToBegin { project }).expect("expected dismissal room");
    }
    assert_eq!(d.dismissals(PARTY).as_ref(), expected.into_boxed().as_ref());
}

#[test]
fn a_dismissal_past_the_bound_is_refused_at_restart() {
    let d = restore_rows(DISMISSALS.checked_add(1).unwrap());
    assert_eq!(
        d.restore_fit_failure(),
        Some(&RestoreFit {
            family: Some(RecordKind::Dismissed),
            set: RestoreSet::Rows(RecordKind::Dismissed),
            held: u64::from(DISMISSALS),
            additional: 1,
            limit: u64::from(DISMISSALS),
        })
    );
    assert_eq!(d.dismissals(PARTY).len(), usize::try_from(DISMISSALS).unwrap());
}
