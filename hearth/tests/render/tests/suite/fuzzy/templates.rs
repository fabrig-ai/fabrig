use hearth_render_world::{expressions, fixtures, reference, templates};
use skein_world::{
    sweep::{Case, cases},
    this_test,
};
use std::convert::Infallible;

pub(super) const SEED: u64 = 0x0015_F000;
const ROUNDS: u64 = 2_000;

#[test]
fn generated_templates_and_contexts_render_as_the_reference_does() -> Result<(), Case<Infallible>> {
    cases(this_test!("generated_templates_and_contexts_render_as_the_reference_does"), 0..ROUNDS, |seed| {
        let mut rng = super::reference_rng(SEED, seed);
        let limits = fixtures::limits(&mut rng);
        let roots = expressions::roots(&mut rng);
        let tree = templates::tree(&mut rng, &roots, templates::WIDE);
        assert_eq!(
            templates::run(&tree, &roots, &limits),
            reference::run(&tree, &roots, &limits),
            "seed {seed}: {tree:?}"
        );
        Ok(())
    })
}
