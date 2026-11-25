#[test]
fn drawn_records_cross_unchanged_or_are_refused_at_derived_layouts() {
    for seed in 0..32 {
        let mut rng = skein_lib::Rng::new(seed);
        crate::layouts::exchange(seed, rng.below(2) == 0);
    }
}
