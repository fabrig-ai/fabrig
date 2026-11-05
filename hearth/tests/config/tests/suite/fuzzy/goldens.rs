use hearth_config_world::world::{self, Settings};
use hearth_test_fixtures::golden;
use skein_lib::Rng;
#[test]
fn golden_documents_cut_and_mutated_read_as_the_reference_reads_them() {
    let corpus = golden::corpus("json");
    for seed in golden::CASES {
        let mut rng = Rng::new(golden::SEED + seed);
        for (path, input) in &corpus {
            let limits = world::limits(&mut rng);
            for document in [input.clone(), hearth_test_fixtures::hostile::mutate(&mut rng, input)] {
                let settings = Settings::calm(&mut rng, limits);
                let run = world::check(&document, &settings, golden::SEED + seed);
                assert!(run.outcome.is_some(), "{}: seed {seed}", path.display());
            }
        }
    }
}
