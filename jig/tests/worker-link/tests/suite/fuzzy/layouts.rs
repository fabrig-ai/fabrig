use core::convert::Infallible;
use skein_world::{sweep::cases, this_test};

#[test]
fn drawn_records_cross_unchanged_or_are_refused_at_derived_layouts() {
    cases(this_test!("drawn_records_cross_unchanged_or_are_refused_at_derived_layouts"), 0..32, |seed| {
        let mut rng = skein_lib::Rng::new(seed);
        crate::layouts::exchange(seed, rng.below(2) == 0);
        Ok::<(), Infallible>(())
    })
    .expect("layout checks assert");
}
