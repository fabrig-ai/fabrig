//! Sign-ins at their pending bound (`jig/docs/design/domain/testing-workloads.md`, 7).

use jig_engine_clients_kit::{bounds::PENDING_EXCHANGES, open, workload};
use skein_world::{
    streams::Streams,
    sweep::{Sweep, cases},
    this_test,
};

#[test]
fn sign_ins_resend_and_fill_at_any_step() -> Result<(), String> {
    let mut sweep = Sweep::new("sign_ins_resend_and_fill_at_any_step");
    sweep.configure(open::RESENDS_A_PENDING_REQUEST);
    sweep.configure(PENDING_EXCHANGES);
    // One seed is enough for both kinds to fall.
    cases(this_test!("sign_ins_resend_and_fill_at_any_step"), 1..2, |seed| -> Result<(), String> {
        let ledger = workload::run(Streams::new(seed), &sweep)?;
        sweep.ran(&ledger);
        Ok(())
    })
    .map_err(|case| case.to_string())?;
    sweep.check_fell()
}
