//! The journal against a plain queue model, with all sequences from one stream
//! in one case (skein/docs/design/testing/sweeps.md, 4.2).

use core::convert::Infallible;
use skein_lib_tests::journal::check;
use skein_world::{sweep::cases, this_test};

#[test]
fn many_journal_sequences_follow_the_model() {
    cases(this_test!("many_journal_sequences_follow_the_model"), [0x3A_01], |seed| {
        check(seed, 20_000);
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}
