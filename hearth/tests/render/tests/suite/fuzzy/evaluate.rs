use hearth_render_world::{expressions, fixtures, reference};
use skein_world::{
    sweep::{Case, cases, gate},
    this_test,
};
use std::{collections::BTreeSet, convert::Infallible};

pub(super) const SEED: u64 = 0x0014_F000;
const ROUNDS: u64 = 10_000;

#[test]
fn generated_expressions_and_contexts_evaluate_as_the_reference_does() -> Result<(), Case<Infallible>> {
    let mut fell = BTreeSet::new();
    let gate = gate();
    cases(this_test!("generated_expressions_and_contexts_evaluate_as_the_reference_does"), 0..ROUNDS, |seed| {
        let mut rng = super::reference_rng(SEED, seed);
        let limits = fixtures::limits(&mut rng);
        let roots = expressions::roots(&mut rng);
        let tree = expressions::tree(&mut rng, &roots, expressions::WIDE);
        let got = expressions::evaluate(&tree, &roots, &limits);
        assert_eq!(got, reference::evaluate(&tree, &roots, &limits), "seed {seed}: {tree:?}");
        if gate && let Err(kind) = got {
            fell.insert(kind);
        }
        Ok(())
    })?;
    if gate {
        fixtures::assert_fell(&fell, expressions::KINDS);
    }
    Ok(())
}
