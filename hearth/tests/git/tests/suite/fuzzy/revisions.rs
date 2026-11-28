//! Revision and answer sweeps (skein/docs/design/testing/sweeps.md, 4).

use hearth_git_world::{fixtures::revisions as inputs, reference};
use hearth_protocol_git::invocation;
use skein_lib::Rng;
use skein_world::sweep::{Case, cases};
use std::convert::Infallible;

fn drawn(seed: u64) -> inputs::Case {
    inputs::case(&mut Rng::new(inputs::SEED.wrapping_add(seed)))
}

#[test]
fn rev_parse_answers_cut_and_mutated_decode_as_the_reference_does() -> Result<(), Case<Infallible>> {
    cases(
        skein_world::this_test!("rev_parse_answers_cut_and_mutated_decode_as_the_reference_does"),
        inputs::CASES,
        |seed| {
            let input = drawn(seed);
            for exit in inputs::EXITS {
                assert_eq!(
                    inputs::decoded(&input.stdout, *exit),
                    reference::answer(&input.stdout, *exit),
                    "seed {seed}: {:?}",
                    input.stdout
                );
            }
            Ok(())
        },
    )
}

#[test]
fn revision_bytes_obey_the_guard_alphabet_and_limit() -> Result<(), Case<Infallible>> {
    cases(skein_world::this_test!("revision_bytes_obey_the_guard_alphabet_and_limit"), inputs::CASES, |seed| {
        let input = drawn(seed);
        assert_eq!(
            invocation::revision(&input.revision).map(|value| value.bytes().to_vec()),
            reference::revision(&input.revision),
            "seed {seed}: {:?}",
            input.revision
        );
        Ok(())
    })
}

#[test]
fn revision_and_answer_draws_wrap_the_full_width_seed() {
    // u64::MAX plus the fixture's offset 0x0030_6000 wraps to 0x0030_5fff.
    const WRAPPED_SEED: u64 = 0x0030_5fff;
    let expected = inputs::case(&mut Rng::new(WRAPPED_SEED));
    let input = drawn(u64::MAX);
    assert_eq!((input.revision, input.stdout), (expected.revision, expected.stdout));
}
