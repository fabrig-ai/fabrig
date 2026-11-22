//! Open exclusions are accepted only for their combination (jig/docs/design/domain/testing.md, 7.3).

use skein_world::sweep::{Exclusion, Sweep};

fn accepts_only_while_condition_holds(exclusions: impl IntoIterator<Item = Exclusion>) {
    let mut sweep = Sweep::new("open problems");
    for exclusion in exclusions {
        let Exclusion { scenario, fault, when, fixed_by } = exclusion;
        assert_eq!(when.len(), 1, "{scenario} has one while condition");
        sweep.configure(fault);
        sweep.exclude(exclusion).expect("a configured open problem has a well-formed exclusion");
        let found = sweep.leaves_out(fault, when).expect("the combination leaves out its fault");
        assert_eq!(found.scenario, scenario);
        assert_eq!(found.fixed_by, fixed_by);
        assert_eq!(found.fault, fault);
        assert_eq!(found.when, when);
        assert!(sweep.leaves_out(fault, &[]).is_none(), "{scenario} stays in without its while condition");
    }
}

#[test]
fn core_exclusions_are_accepted_only_while_their_conditions_hold() {
    accepts_only_while_condition_holds(jig_core_kit::open::EXCLUSIONS);
}

#[test]
fn engine_clients_exclusions_are_accepted_only_while_their_conditions_hold() {
    accepts_only_while_condition_holds(jig_engine_clients_kit::open::EXCLUSIONS);
}
