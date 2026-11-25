//! P63's residual: restored fixed charter parts are checked before the first run.
//! temper/docs/design/domain/root.md, 9; domain/agent/room.md, 4.
use jig_domain_core::{self as core, settings};
use temper_engine_domain::{Delivery, Record, StartStop, room::Refusal};
use temper_engine_domain_world::{direct::Driver, fixtures, walking};

const SEED: u64 = 63;
const START_STEPS: u32 = 200; // The direct world's bounded cold-start drain.
const ALLOWANCE: u64 = 1_024;
const INSTRUCTIONS: usize = 1_100; // Plus the eight-byte model name, with no other host costs.
const WORDS: &[u8] = b"charter 1 on a first wake is 1,108 bytes owned before its brief, past its allowance of 1,024; trim its text, parts or tool families";

#[test]
fn a_stored_charter_is_checked_at_start() {
    let limits = walking::limits();
    let mut store = fixtures::restored_store(walking::config(SEED), &limits);
    let mut charter = walking::charter();
    // Synthetic stored setting: its payload fits the journal, but not the
    // receiving profile's smaller fixed-part allowance at this restart.
    charter.policy.instructions = vec![b'x'; INSTRUCTIONS].into_boxed_slice();
    let number = charter.number;
    store.seed(
        Box::new([Record::Core(core::Record::Core(core::CoreRecord::Settings(settings::Record::Changed {
            setting: settings::Setting::Charter(number),
            changed: settings::Changed { value: Some(settings::Value::Charter(Box::new(charter))), revision: 1 },
        })))]),
        Box::new([]),
    );
    let mut configuration = walking::config(SEED);
    configuration.host_part.allowance = ALLOWANCE;
    let mut restarted = Driver::cold(store, configuration, &limits);
    restarted.broken("synthetic stored charter exceeds the receiving profile's fixed-part allowance");
    for _ in 0..START_STEPS {
        restarted.advance(true);
        if restarted.stopped || restarted.root.ready() {
            break;
        }
    }
    assert!(
        restarted.stopped && !restarted.root.ready(),
        "start must refuse the stored charter before opening: stop={:?}; ready={}",
        restarted.root.start_stop(),
        restarted.root.ready(),
    );
    let Some(StartStop::Room(Refusal::Fixed(unfit))) = restarted.root.start_stop() else {
        panic!("expected a fixed-part refusal, got {:?}", restarted.root.start_stop());
    };
    assert_eq!(unfit.charter, number);
    assert_eq!(unfit.words().as_ref(), WORDS);
    assert!(
        !restarted.delivered.iter().any(|delivery| matches!(
            delivery,
            Delivery::Assigned { assignment, .. } if assignment.charter == number
        )),
        "no first assignment may precede the stored charter's refusal",
    );
}
