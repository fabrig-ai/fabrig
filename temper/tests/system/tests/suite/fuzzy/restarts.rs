//! The system row of temper/docs/design/testing/restarts.md, section 9.
use skein_fake_llm_domain::api::Line;
use skein_lib::Duration;
use skein_world::{faults::FaultLedger, sweep::Sweep};
use temper_faults::restarts::MID_TURN;
use temper_system_world::composed::{World, restarts::Injector};

use crate::restarts::{WORDS, assert_adopted, finish, script};

#[test]
fn seeded_engine_restarts_mid_turn_finish_the_run() {
    let mut sweep = Sweep::new("system engine restarts mid-turn");
    for kind in MID_TURN {
        sweep.configure(kind);
    }
    for seed in [0, 1] {
        eprintln!("seed {seed}");
        let script = script();
        // Every call in this script is the host's read_docs, not a local tool.
        let host_calls =
            script.turns.iter().flat_map(|turn| &turn.lines).filter(|line| matches!(line, Line::Call { .. })).count();
        let (point, restart) = Injector::new(seed).draw(script.turns.len(), host_calls);
        eprintln!("holding {point:?}, {restart:?}");
        let mut world = World::mid_turn_chat(seed, script, WORDS, point);
        world.until_mid_turn(point);
        let attempt = world.assignment().attempt;
        let mut ledger = FaultLedger::new();
        let injection = world.inject_mid_turn(restart, &mut ledger);
        world.elapse_absence(Duration::from_millis(100));
        world.open_after_mid_turn(&mut ledger, injection);
        world.join_link();
        sweep.ran(&ledger);
        assert_adopted(&world, attempt, jig_domain_link::Phase::Active);
        finish(&mut world, attempt, point);
    }
    sweep.check_fell().expect("both engine restart kinds fell");
}
