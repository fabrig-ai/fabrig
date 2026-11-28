use crate::oracle::engine;
use engine::{Mismatch, Oracle};
use hearth_render_world::{
    expressions::{self, Roots, Style},
    fixtures::{self, oracle as inputs},
    spacing, templates,
};
use skein_lib::Rng;
use skein_world::{
    sweep::{Case, cases, gate},
    this_test,
};
use std::collections::BTreeSet;

fn checked_oracle() -> Oracle {
    let oracle = Oracle::new();
    oracle.check_differences().expect("listed differences keep their pinned outcomes");
    oracle
}

fn draw(seed: u64) -> (Rng, Roots) {
    fixtures::draw(inputs::TEMPLATE_SEED.wrapping_add(seed))
}

#[test]
fn oracle_draws_wrap_the_full_width_case_seed() {
    // The draw starts at the seed just before the template block.
    assert_eq!(draw(u64::MAX).0, fixtures::draw(0x0030_4fff).0);
}

#[test]
fn generated_expressions_at_wide_shapes_render_as_the_oracle_renders_them() -> Result<(), Case<Box<Mismatch>>> {
    let oracle = checked_oracle();
    cases(
        this_test!("generated_expressions_at_wide_shapes_render_as_the_oracle_renders_them"),
        inputs::SWEEP_CASES,
        |seed| {
            let (mut rng, roots) = draw(seed);
            let tree = expressions::tree(&mut rng, &roots, inputs::WIDE_EXPRESSIONS);
            let template = templates::output_of(&expressions::text(&tree, Style::Oracle));
            oracle.compare(&template, &roots).map_err(Box::new)
        },
    )
}

#[test]
fn generated_templates_at_wide_shapes_render_as_the_oracle_renders_them() -> Result<(), Case<Box<Mismatch>>> {
    let oracle = checked_oracle();
    let gate = gate();
    let mut drawn = BTreeSet::new();
    let mut dressed = BTreeSet::new();
    cases(
        this_test!("generated_templates_at_wide_shapes_render_as_the_oracle_renders_them"),
        inputs::SWEEP_CASES,
        |seed| {
            let (mut rng, roots) = draw(seed);
            let tree = templates::tree(&mut rng, &roots, inputs::WIDE_TEMPLATES);
            if gate {
                drawn.extend(templates::constructs(&tree));
            }
            let template = spacing::dressed(&mut rng, &tree);
            if gate {
                dressed.extend(spacing::constructs(&template));
            }
            oracle.compare(&template, &roots).map_err(Box::new)
        },
    )?;
    if gate {
        fixtures::assert_fell(&drawn, templates::CONSTRUCTS);
        fixtures::assert_fell(&dressed, spacing::CONSTRUCTS);
    }
    Ok(())
}
