//! A connector's store refusal is a start outcome (domain/testing-fakes.md, 1).

use jig_core_world::world::{Setup, World};
use jig_domain_core::start::{ApplicationWhy, Failure, Phase};
use jig_test_connector as connector;
use jig_test_domain as root;

#[test]
fn a_connector_restore_refusal_stops_the_start_and_names_the_setting() {
    const SEED: u64 = 71;
    const KIND: u32 = 91;
    const SCOPE: u32 = 17;
    const NUMBER: u32 = 3;
    let mut world = World::new(Setup { seed: SEED, ..Setup::default() });
    assert!(world.domain.ready());
    world.seed(Box::new([root::Record::Connector {
        number: 1,
        record: connector::Record::SettingDefault {
            kind: KIND,
            scope: SCOPE,
            number: NUMBER,
            value: b"stored".as_slice().into(),
        },
    }]));
    world.restart(SEED, false);
    assert!(world.stopped);
    assert!(!world.domain.ready());
    assert_eq!(
        world.domain.start_failure(),
        Some((
            Some(Phase::RestoreConnector { connector: 1 }),
            Failure::Application { kind: KIND, scope: SCOPE, number: NUMBER, why: ApplicationWhy::Shape },
        ))
    );
    assert!(!world.restart_steps.contains(&Phase::Switch));
    // The deliberately unowned stored setting also refuses the closing restart.
    world.broken();
}
