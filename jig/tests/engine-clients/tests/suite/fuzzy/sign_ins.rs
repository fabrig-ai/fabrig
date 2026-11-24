//! Sign-ins at their pending bound (`jig/docs/design/domain/testing-workloads.md`, 7).

use jig_engine_clients_kit::{bounds::PENDING_EXCHANGES, open, workload};
use skein_world::{
    streams::Streams,
    sweep::{Sweep, each_seed},
};

#[test]
fn sign_ins_resend_and_fill_at_any_step() {
    let mut sweep = Sweep::new("sign_ins_resend_and_fill_at_any_step");
    sweep.configure(open::RESENDS_A_PENDING_REQUEST);
    sweep.configure(PENDING_EXCHANGES);
    for exclusion in open::EXCLUSIONS {
        sweep.exclude(exclusion).expect("P117's narrow exclusion");
    }
    // One seed is enough for both kinds to fall.
    each_seed(1..2, |seed| {
        eprintln!("seed {seed}");
        let ledger = workload::run(Streams::new(seed), &sweep).expect("sign-ins complete");
        sweep.ran(&ledger);
    });
    sweep.check_fell().expect("both the resend and the pending bound fell");
    assert!(sweep.left_out("a_parked_act_holds_no_exchange") > 0, "P117 leaves out a parked first request's resend");
}
