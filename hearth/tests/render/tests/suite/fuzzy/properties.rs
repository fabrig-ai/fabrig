use hearth_render_world::fixtures::laws;
use hearth_render_world::fixtures::properties as inputs;
use hearth_render_world::json;
use skein_lib::Rng;
use skein_world::{
    sweep::{Case, cases},
    this_test,
};
use std::convert::Infallible;

#[test]
fn patch_draws_wrap_the_full_width_case_seed() {
    // The draw starts at the seed just before the merge-law block.
    let mut rng = Rng::new(0x0030_2fff);
    let expected = (json::generate(&mut rng, laws::WIDE_DEPTH), json::generate(&mut rng, laws::WIDE_DEPTH));
    assert_eq!(laws::patches(u64::MAX, laws::WIDE_DEPTH), expected);
}

#[test]
fn generated_patches_obey_the_laws_and_the_reference() -> Result<(), Case<Infallible>> {
    cases(this_test!("generated_patches_obey_the_laws_and_the_reference"), laws::ROUNDS, |seed| {
        let (target, patch) = laws::patches(seed, laws::WIDE_DEPTH);
        laws::reference_agrees(&target, &patch, seed);
        laws::idempotent(&target, &patch, seed);
        let object = laws::object(target);
        assert_eq!(laws::merged(&[&object, &laws::EMPTY]), laws::merged(&[&object]), "seed {seed}");
        assert_eq!(laws::merged(&[&object, &patch]).as_ref().map(laws::has_null), Some(false), "seed {seed}");
        Ok(())
    })
}
#[test]
fn generated_repositories_and_edits_keep_every_property_0() {
    inputs::sweep(this_test!("generated_repositories_and_edits_keep_every_property_0"), 0);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_1() {
    inputs::sweep(this_test!("generated_repositories_and_edits_keep_every_property_1"), 1);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_2() {
    inputs::sweep(this_test!("generated_repositories_and_edits_keep_every_property_2"), 2);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_3() {
    inputs::sweep(this_test!("generated_repositories_and_edits_keep_every_property_3"), 3);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_4() {
    inputs::sweep(this_test!("generated_repositories_and_edits_keep_every_property_4"), 4);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_5() {
    inputs::sweep(this_test!("generated_repositories_and_edits_keep_every_property_5"), 5);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_6() {
    inputs::sweep(this_test!("generated_repositories_and_edits_keep_every_property_6"), 6);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_7() {
    inputs::sweep(this_test!("generated_repositories_and_edits_keep_every_property_7"), 7);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_8() {
    inputs::sweep(this_test!("generated_repositories_and_edits_keep_every_property_8"), 8);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_9() {
    inputs::sweep(this_test!("generated_repositories_and_edits_keep_every_property_9"), 9);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_10() {
    inputs::sweep(this_test!("generated_repositories_and_edits_keep_every_property_10"), 10);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_11() {
    inputs::sweep(this_test!("generated_repositories_and_edits_keep_every_property_11"), 11);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_12() {
    inputs::sweep(this_test!("generated_repositories_and_edits_keep_every_property_12"), 12);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_13() {
    inputs::sweep(this_test!("generated_repositories_and_edits_keep_every_property_13"), 13);
}
#[test]
fn generated_repositories_and_edits_keep_every_property_14() {
    inputs::sweep(this_test!("generated_repositories_and_edits_keep_every_property_14"), 14);
}
