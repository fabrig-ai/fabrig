//! P63's residual: restored fixed charter parts are checked before the first run.
//! temper/docs/design/domain/root.md, 9; domain/agent/room.md, 4.
use jig_domain_core::{self as core, settings};
use temper_engine_domain::{Delivery, Record};
use temper_engine_domain_world::{direct::Driver, fixtures, walking};

const SEED: u64 = 63;
const START_STEPS: u32 = 200; // The direct world's bounded cold-start drain.
const ALLOWANCE: u64 = 1024;

#[test]
#[ignore = "bounded waits: P63, refuse restored fixed charter parts at start"]
fn a_stored_charter_is_checked_at_start() {
    let limits = walking::limits();
    let mut store = fixtures::restored_store(walking::config(SEED), &limits);
    let mut charter = walking::charter();
    // Synthetic stored setting: its payload fits the journal, but not the
    // receiving profile's smaller fixed-part allowance at this restart.
    charter.policy.instructions = vec![b'x'; 1100].into_boxed_slice();
    store.seed(
        Box::new([Record::Core(core::Record::Core(core::CoreRecord::Settings(settings::Record::Changed {
            setting: settings::Setting::Charter(charter.number),
            changed: settings::Changed {
                value: Some(settings::Value::Charter(Box::new(charter.clone()))),
                revision: 1,
            },
        })))]),
        Box::new([]),
    );
    let mut configuration = walking::config(SEED);
    configuration.host_part.allowance = ALLOWANCE;
    configuration.host_part.check_fixed(&walking::charter()).expect("startup default still fits");
    let unfit = configuration.host_part.check_fixed(&charter).expect_err("stored fixed part exceeds its room");
    assert_eq!(unfit.charter, charter.number);
    assert!(unfit.bytes > unfit.allowance, "{}", String::from_utf8_lossy(&unfit.words()));
    // Bounded waits must add and pin the emitted fixed-start diagnostic's words;
    // today's StartStop only carries the distinct rendered-room failure.
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
        "start must refuse the stored charter before opening: {}; stop={:?}; ready={}",
        String::from_utf8_lossy(&unfit.words()),
        restarted.root.start_stop(),
        restarted.root.ready(),
    );
    assert!(
        !restarted.delivered.iter().any(|delivery| matches!(
            delivery,
            Delivery::Assigned { assignment, .. } if assignment.charter == charter.number
        )),
        "no first assignment may precede the stored charter's refusal",
    );
}
