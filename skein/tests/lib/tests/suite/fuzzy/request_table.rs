//! Request-table promises with bounded outputs and restarts, one stream per case.
//! Stream reach is checked only at the gate (skein/docs/design/testing/sweeps.md, 4.2 and 4.5).

use core::convert::Infallible;
use skein_lib_tests::request_table::check_with_reach;
use skein_world::{
    sweep::{cases, gate},
    this_test,
};

#[test]
fn many_request_sequences_follow_the_obligation_model() {
    let at_gate = gate();
    cases(this_test!("many_request_sequences_follow_the_obligation_model"), [0xA5_31], |seed| {
        let _trace = check_with_reach(seed, 4_096, at_gate);
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}
