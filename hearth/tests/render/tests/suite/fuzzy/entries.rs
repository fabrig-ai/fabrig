use hearth_render_world::{declarations, expressions, fixtures, reference};
use skein_world::{
    sweep::{Case, cases},
    this_test,
};
use std::convert::Infallible;

pub(super) const SEED: u64 = 0x0016_F000;
const ROUNDS: u64 = 5_000;

#[test]
fn generated_declarations_and_contexts_expand_as_the_reference_does() -> Result<(), Case<Infallible>> {
    cases(this_test!("generated_declarations_and_contexts_expand_as_the_reference_does"), 0..ROUNDS, |seed| {
        let mut rng = super::reference_rng(SEED, seed);
        let limits = fixtures::limits(&mut rng);
        let roots = expressions::roots(&mut rng);
        let tree = declarations::tree(&mut rng, &roots);
        assert_eq!(
            declarations::expand(&tree, &roots, &limits),
            reference::expand(&tree, &roots, &limits),
            "seed {seed}: {tree:?}"
        );
        Ok(())
    })
}
