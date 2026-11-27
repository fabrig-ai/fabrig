//! Whole-file replacements and loads replay under every applicable disk fault
//! (`skein/docs/design/testing/sweeps.md`, 4.2–4.5).

use std::collections::BTreeSet;
use std::convert::Infallible;

use skein_io_world::files::{NEW, OLD, Story, world};
use skein_sim::{Config, Event, Fault};
use skein_world::sweep::{cases, gate};
use skein_world::this_test;

/// The gate's disk fault census covers seeds 0 through 256.
const SEEDS: u64 = 257;

#[test]
fn stores_and_loads_under_chaos_settle_replay_and_every_disk_fault_falls() {
    let mut seen = BTreeSet::new();
    let at_gate = gate();
    let test = this_test!("stores_and_loads_under_chaos_settle_replay_and_every_disk_fault_falls");
    cases::<_, Infallible, _>(test, 0..SEEDS, |seed| {
        let mut first = world(seed, Config::chaos(), Story::Replace, None, 3).run();
        let mut second = world(seed, Config::chaos(), Story::Replace, None, 3).run();
        assert_eq!(first.trace, second.trace, "seed {seed}: exact replay");
        assert_eq!(first.procs[0].events, second.procs[0].events, "seed {seed}: terminal replay");
        let bytes = first.machine.bytes(b"record");
        assert_eq!(bytes, second.machine.bytes(b"record"));
        assert!(bytes == OLD || bytes == NEW, "seed {seed}: whole old or new target");
        let mut names = vec![b"record".to_vec()];
        if let skein_io::file::Event::Failed { residue: Some(residue), committed, .. } = &first.procs[0].events[0] {
            assert!(!committed, "seed {seed}: a committed store has no temporary residue");
            assert_eq!(bytes, OLD, "seed {seed}: cleanup refusal preserves old bytes");
            assert!(residue.name.starts_with(b"record.skein-"));
            names.push(residue.name.to_vec());
        }
        assert_eq!(first.machine.names(), names, "seed {seed}: all temporary residue is reported");
        assert_eq!(second.machine.names(), names, "seed {seed}: residue replay");
        if at_gate {
            for entry in &first.trace {
                if let Event::Fault(fault) = entry.event {
                    seen.insert(fault);
                }
            }
        }
        first.machine.finish();
        second.machine.finish();
        Ok(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    if at_gate {
        let required = BTreeSet::from([
            Fault::Latency,
            Fault::ShortWrite,
            Fault::ShortRead,
            Fault::NoBuffer,
            Fault::NoSpace,
            Fault::ReadOnly,
            Fault::IoError,
            Fault::Hung,
            Fault::CancelRace,
            Fault::CancelUnsubmitted,
        ]);
        assert!(required.is_subset(&seen), "file faults missing: {:?}", required.difference(&seen).collect::<Vec<_>>());
        eprintln!("files census: {SEEDS} seeds, all ten applicable faults covered");
    }
}
