//! File prefixes, terminal ordering, memory and replay under every append fault
//! (`skein/docs/design/testing/sweeps.md`, 4.2–4.5).

use std::collections::BTreeSet;
use std::convert::Infallible;

use skein_io_world::append::{CONTENTS, Story, simulated};
use skein_sim::{Config, Event, Fault};
use skein_world::sweep::{cases, gate};
use skein_world::{Memory, this_test};

/// The gate's append fault census covers seeds 0 through 256.
const SEEDS: u64 = 257;

#[test]
fn appends_under_chaos_settle_replay_and_every_applicable_fault_falls() {
    let mut seen = BTreeSet::new();
    let at_gate = gate();
    let test = this_test!("appends_under_chaos_settle_replay_and_every_applicable_fault_falls");
    cases::<_, Infallible, _>(test, 0..SEEDS, |seed| {
        for story in [Story::Finish, Story::Close, Story::Abort] {
            let mut first = simulated(seed, Config::chaos(), story, Memory::Checked);
            let mut second = simulated(seed, Config::chaos(), story, Memory::Checked);
            assert_eq!(first.trace, second.trace, "seed {seed}: exact replay");
            assert_eq!(first.procs[0].events, second.procs[0].events);
            let bytes = first.machine.bytes();
            assert_eq!(bytes, second.machine.bytes());
            assert!(CONTENTS.starts_with(&bytes), "seed {seed}: only an ordered prefix reached the file: {bytes:?}");
            if at_gate {
                for entry in &first.trace {
                    if let Event::Fault(fault) = entry.event {
                        seen.insert(fault);
                    }
                }
            }
            first.machine.finish();
            second.machine.finish();
        }
        Ok(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    if at_gate {
        let required = BTreeSet::from([
            Fault::Latency,
            Fault::ShortWrite,
            Fault::NoBuffer,
            Fault::NoSpace,
            Fault::ReadOnly,
            Fault::IoError,
            Fault::Hung,
            Fault::CancelRace,
            Fault::CancelUnsubmitted,
        ]);
        assert!(
            required.is_subset(&seen),
            "append faults not yet covered: {:?}",
            required.difference(&seen).collect::<Vec<_>>()
        );
        eprintln!("append census: {SEEDS} seeds, all nine applicable faults covered");
    }
}
