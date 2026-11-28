use std::convert::Infallible;

use jig_fleet_world::turns::{self, Settings};
use skein_world::{
    sweep::{Case, cases, gate},
    this_test,
};

#[test]
fn turn_world_sweeps_replay_backpressure_and_fencing() -> Result<(), Case<Infallible>> {
    let at_gate = gate();
    let mut busy = 0;
    cases(this_test!("turn_world_sweeps_replay_backpressure_and_fencing"), 0..100, |seed| {
        let report = turns::run(Settings {
            drops: seed % 2 == 0,
            restart: seed % 3 != 0,
            cancel: seed % 5 == 0,
            capacity: 1 + u32::try_from(seed % 3).expect("capacity at most three"),
            ..Settings::new(seed)
        });
        if at_gate {
            busy += report.parent_busy;
        }
        assert!(report.committed <= 8);
        Ok(())
    })?;
    if at_gate {
        assert!(busy > 0);
    }
    Ok(())
}
