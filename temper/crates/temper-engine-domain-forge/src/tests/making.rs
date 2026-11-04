use super::*;
use skein_lib::Wall;

const FORGE: u16 = 1;
const OTHER_FORGE: u16 = 2;
pub(crate) fn row(number: u32) -> RepositoryMaking {
    RepositoryMaking {
        key: MakingKey { forge: FORGE, org: Box::from(b"owner".as_slice()), name: number.to_be_bytes().into() },
        from: MakeFrom::Copy { address: Box::from(b"https://source.invalid/owner/repo".as_slice()) },
        into: Some(MakeInto { project: 1, home: true, host: Box::from(b"forge.invalid".as_slice()) }),
        asked: Wall::from_nanos(u64::from(number)),
        copying: true,
        state: MakingState::Waiting,
    }
}
fn restore(d: &mut Domain, row: RepositoryMaking) {
    let requests = outputs(d, Event::Restore { record: Stored::Making(row) });
    assert!(requests.is_empty());
}
#[test]
fn makings_restore_at_the_bound() {
    let mut d = domain();
    for number in 1..=LIMITS.makings {
        restore(&mut d, row(number));
    }
    assert_eq!(d.restore_fit_failure(), None);
    let mut numbers = 1..=LIMITS.makings;
    for making in d.makings(FORGE) {
        let number = numbers.next().expect("no row past the bound");
        assert_eq!(making, &row(number));
    }
    assert_eq!(numbers.next(), None);
    assert_eq!(d.makings(OTHER_FORGE).count(), 0);
}
#[test]
fn a_making_past_the_bound_is_refused_at_restart() {
    let mut d = domain();
    for number in 1..=LIMITS.makings + 1 {
        restore(&mut d, row(number));
    }
    assert_eq!(
        d.restore_fit_failure(),
        Some(&RestoreFit {
            family: Some(RecordKind::Making),
            set: RestoreSet::Rows(RecordKind::Making),
            held: u64::from(LIMITS.makings),
            additional: 1,
            limit: u64::from(LIMITS.makings)
        })
    );
    assert_eq!(d.making(&row(LIMITS.makings + 1).key), None);
}
#[test]
fn a_row_making_at_restart_comes_back_waiting_with_its_mark() {
    let mut d = domain();
    let expected = row(1);
    let mut making = expected.clone();
    making.state = MakingState::Making;
    restore(&mut d, making);
    assert_eq!(d.making(&expected.key), Some(&expected));
}
#[test]
fn next_making_is_the_oldest_waiting() {
    let mut d = domain();
    let mut older = row(2);
    older.asked = Wall::EPOCH;
    restore(&mut d, row(1));
    restore(&mut d, older.clone());
    assert_eq!(d.next_making(FORGE), Some(&older));
    assert_eq!(d.next_making(OTHER_FORGE), None);
    let mut tied = row(1);
    tied.asked = Wall::EPOCH;
    restore(&mut d, tied.clone());
    assert_eq!(d.next_making(FORGE), Some(&tied));
    tied.state = MakingState::Made { id: 42, branch: Box::from(b"main".as_slice()) };
    restore(&mut d, tied.clone());
    assert_eq!(d.next_making(FORGE), Some(&older));
    assert_eq!(d.making_due(FORGE), Some(tied.key));
    assert_eq!(d.making_due(OTHER_FORGE), None);
}
