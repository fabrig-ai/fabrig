//! Each simulated restart fault, including its ledger interval.
use skein_world::{domain::Verdict, faults::Struck};
use temper_faults::restarts::{OLDER_LAYOUT, UNIT_SIGNALLED};
use temper_one_process_world::{fixtures, restarts};

#[test]
fn a_unit_signal_interrupts_the_held_chat_and_reopens_the_same_disk() {
    let outcome = restarts::unit_signalled(fixtures::calm(fixtures::SEED), 0);
    let signal = outcome.signal.expect("unit signalled");
    assert_eq!(signal.iteration, 0);
    assert!(signal.members.contains(&"service"));
    assert!(signal.members.contains(&"agent"), "the live agent gets its own signal");
    assert_eq!(signal.received.iter().filter(|&&name| name == "service").count(), 1);
    assert_eq!(
        signal.received.iter().filter(|&&name| name == "agent").count(),
        signal.members.iter().filter(|&&name| name == "agent").count(),
        "each live agent reads its own termination signal",
    );
    let [entry] = outcome.ledger.entries() else { panic!("one unit signal") };
    assert_eq!(entry.kind, UNIT_SIGNALLED);
    assert_eq!(entry.struck, Struck::default());
    let [first, reopened] = outcome.openings.as_slice() else { panic!("both starts opened") };
    assert!(*first <= signal.at);
    assert_eq!(entry.from, signal.at);
    assert_eq!(entry.until, Some(*reopened));
    assert!(*reopened > signal.at);
    let run = outcome.fresh.expect("fresh chat story");
    assert_eq!(run.verdict, Verdict::Passed);
    assert!(run.drained, "service and every program drained");
    assert_eq!(run.restarts, 1);
    let stopped = run.stopped.expect("held run at the signal");
    assert!(stopped.next.is_some_and(|next| next > stopped.attempt), "the next start takes the run again");
}

#[test]
fn an_older_layout_opens_and_serves_the_kept_person() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("golden/dev");
    let outcome = restarts::older_layout(fixtures::calm(fixtures::SEED), &directory);
    assert!(outcome.signal.is_none());
    assert!(outcome.fresh.is_none());
    let [entry] = outcome.ledger.entries() else { panic!("one older-layout start") };
    assert_eq!(entry.kind, OLDER_LAYOUT);
    assert_eq!(entry.struck, Struck::default());
    assert_eq!(entry.from, skein_lib::Time::ZERO);
    let [opened] = outcome.openings.as_slice() else { panic!("one opening") };
    assert_eq!(entry.until, Some(*opened));
    // The engine may open the image start in the run's first instant, before its connectors catch up.
    assert!(*opened >= entry.from);
}
