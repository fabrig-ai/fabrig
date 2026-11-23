use hearth_config_world::{
    fixtures::Seen,
    generate,
    world::{self, Settings},
};
use skein_lib::Rng;
const FIRST_SEED: u64 = 0xC0_0300;
const ROUNDS: u64 = 10000;
#[test]
fn generated_documents_mutated_and_cut_read_as_the_reference_reads_them() {
    let mut seen = Seen::default();
    let mut mutations = hearth_test_fixtures::hostile::MutationBatch::default();
    for seed in FIRST_SEED..FIRST_SEED + ROUNDS {
        let mut rng = Rng::new(seed);
        let limits = world::limits(&mut rng);
        let document = generate::document(&mut rng, &limits);
        let document =
            if rng.chance(700) { hearth_test_fixtures::hostile::mutate(&mut rng, &document) } else { document.into() };
        let settings = Settings::chaotic(&mut rng, limits, document.input().len());
        let run = mutations.judge(document, |document| world::check(&document, &settings, seed));
        seen.record(&run, &settings);
    }
    mutations.finish();
    seen.assert_fell(Seen::EVERY_FAULT_AND_ENDING);
}
