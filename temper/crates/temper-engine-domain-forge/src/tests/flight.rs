use super::fixtures::{HEAD, answer, next_call, open_pull};
use super::*;

const TASK: u64 = 7;
const ENTRY: u64 = 11;
const MERGED: skein_forge::api::Commit = [9; 32];
fn restore_flights(count: u32) -> Domain {
    let mut domain = domain();
    for entry in 0..u64::from(count) {
        outputs(&mut domain, Event::Restore { record: Stored::Flight { task: TASK, entry } });
    }
    domain
}
#[test]
fn flight_restoration_takes_the_entry_capacity() {
    assert_eq!(restore_flights(LIMITS.entries).restore_fit_failure(), None);
}
#[test]
fn flight_restoration_refuses_one_past_the_entry_capacity() {
    let d = restore_flights(LIMITS.entries + 1);
    assert_eq!(
        d.restore_fit_failure(),
        Some(&RestoreFit {
            family: Some(RecordKind::Flight),
            set: RestoreSet::Rows(RecordKind::Flight),
            held: u64::from(LIMITS.entries),
            additional: 1,
            limit: u64::from(LIMITS.entries)
        })
    );
}

#[test]
fn a_merge_in_the_forges_queue_keeps_its_flight_until_the_merge_is_proved() {
    let mut domain = domain();
    let mut entry = held_issue();
    entry.effect = skein_forge_client::Effect {
        write: skein_forge::api::Write::Merge { number: 1, head: HEAD, how: skein_forge::api::MergeHow::Style },
        condition: skein_forge_client::Condition::Merge { base: Box::from(b"main".as_slice()) },
    };
    entry.queued = true;
    outputs(&mut domain, Event::Restore { record: Stored::Repository(adopted()) });
    outputs(&mut domain, Event::Restore { record: Stored::Entry { entry, held: false } });
    outputs(&mut domain, Event::Restore { record: Stored::Flight { task: TASK, entry: ENTRY } });
    outputs(&mut domain, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut domain, Event::ReadAfresh);
    outputs(&mut domain, Event::SettleOutbox);
    let (call, op) = next_call(&mut domain);
    assert_eq!(op, skein_forge::api::Op::Read(skein_forge::api::Read::Pull { number: 1 }));
    let mut pull = open_pull();
    pull.state = skein_forge::api::State::Closed;
    pull.merged = Some(MERGED);
    let requests = answer(&mut domain, call, Ok(skein_forge::api::Answer::Pull(pull)));
    assert!(requests.contains(&Request::Erase { key: Key::Flight { task: TASK, entry: ENTRY } }));
    assert!(requests.contains(&Request::Erase { key: Key::Entry(ENTRY) }));
}
