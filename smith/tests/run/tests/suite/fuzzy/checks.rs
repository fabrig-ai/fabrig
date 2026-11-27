//! An independent counted-time model against random stderr lines and read splits.

use std::convert::Infallible;

use crate::checks::held::{Check, draw_marker};
use skein_lib::{Duration, Time, Token};
use skein_world::{
    faults::{FaultLedger, Struck},
    streams::{Stream, Streams},
    sweep::{Case, Sweep, cases, gate},
};

#[test]
fn marker_sequences_and_read_splits_keep_the_held_bound_and_absolute_cap() -> Result<(), Case<Infallible>> {
    let mut sweep = Sweep::new("checks_waiting_for_a_place_fall");
    sweep.configure(smith_faults::CHECK_WAITS);
    let mut fell_in = 0;
    let mut intervals = 0;
    cases(
        skein_world::this_test!("marker_sequences_and_read_splits_keep_the_held_bound_and_absolute_cap"),
        0..96,
        |seed| {
            let streams = Streams::new(seed);
            let mut faults = streams.rng(Stream::Faults, "checks");
            let mut workload = streams.rng(Stream::Workload, "runs");
            let mut ledger = FaultLedger::new();
            let mut held = None;
            let mut check = Check::new(500, 600);
            let mut counted = 0_u64;
            let mut waiting = false;
            let mut now = 0_u64;
            let mut line = Vec::new();
            let mut deadline = 60_u64;
            'sequence: for _ in 0..100 {
                let (chunks, stderr) = draw_marker(&mut workload, &mut faults);
                let pipe = Token::new(if stderr { 42 } else { 41 });
                for bytes in chunks {
                    let later = now + faults.below(5);
                    if later >= deadline {
                        if !waiting {
                            counted += deadline - now;
                        }
                        assert!(counted <= 60, "seed {seed}");
                        break 'sequence;
                    }
                    if !waiting {
                        counted += later - now;
                    }
                    now = later;
                    check.at(now);
                    check.bytes(pipe, &bytes);
                    if stderr {
                        for byte in &bytes {
                            line.push(*byte);
                            if *byte == b'\n' {
                                if line == b"smith-check: waiting\n" {
                                    if !waiting {
                                        held = Some(ledger.inject(
                                            smith_faults::CHECK_WAITS,
                                            Time::ZERO.saturating_add(Duration::from_secs(now)),
                                            Struck { request: Some("check 1".into()), ..Struck::default() },
                                        ));
                                    }
                                    waiting = true;
                                } else if line == b"smith-check: admitted\n" {
                                    if let Some(entry) = held.take() {
                                        ledger
                                            .stop(entry, Time::ZERO.saturating_add(Duration::from_secs(now)))
                                            .expect("admission ends the check's wait");
                                    }
                                    waiting = false;
                                }
                                line.clear();
                            }
                        }
                    }
                    deadline = if waiting { 500 } else { (now + 60 - counted).min(500) };
                    assert!(counted <= 60, "seed {seed}");
                    assert!(deadline <= 500, "seed {seed}");
                    assert_eq!(
                        check.deadline(),
                        Time::ZERO.saturating_add(Duration::from_secs(deadline)),
                        "seed {seed}, counted {counted}, waiting {waiting}"
                    );
                }
            }
            check.settle_at_deadline(deadline);
            if let Some(entry) = held {
                ledger
                    .stop(entry, Time::ZERO.saturating_add(Duration::from_secs(deadline)))
                    .expect("the check's end ends its wait");
            }
            assert!(ledger.entries().iter().all(|entry| entry.until.is_some()), "every wait ended");
            fell_in += usize::from(!ledger.entries().is_empty());
            intervals += ledger.entries().len();
            sweep.ran(&ledger);
            Ok(())
        },
    )?;
    sweep.check_fell().expect("checks waited for a place");
    if gate() {
        eprintln!("check waits: {intervals} intervals in {fell_in}/96 seeds");
    }
    Ok(())
}
