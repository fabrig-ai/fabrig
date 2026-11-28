//! Reviewed document cases (skein/docs/design/testing/sweeps.md, 4).

use hearth_config_world::world::{self, Settings};
use hearth_test_fixtures::golden;
use skein_lib::Rng;
use skein_world::{
    sweep::{Case, cases},
    this_test,
};
use std::convert::Infallible;

fn schedule_seed(seed: u64) -> u64 {
    golden::SEED.wrapping_add(seed)
}

#[test]
fn golden_document_schedules_accept_full_width_seeds() {
    assert_eq!(schedule_seed(u64::MAX), 0x0030_5fff);
}

#[test]
fn golden_documents_cut_and_mutated_read_as_the_reference_reads_them() -> Result<(), Case<Infallible>> {
    let corpus = golden::corpus("json");
    let mut mutations = hearth_test_fixtures::hostile::MutationBatch::default();
    let test = this_test!("golden_documents_cut_and_mutated_read_as_the_reference_reads_them");
    cases(test, golden::CASES, |seed| {
        let scheduled = schedule_seed(seed);
        let mut rng = Rng::new(scheduled);
        for (path, input) in &corpus {
            let limits = world::limits(&mut rng);
            for document in [input.clone().into(), hearth_test_fixtures::hostile::mutate(&mut rng, input)] {
                mutations.judge(document, |document| {
                    let settings = Settings::calm(&mut rng, limits);
                    let run = world::check(&document, &settings, scheduled);
                    assert!(run.outcome.is_some(), "{}: seed {seed}", path.display());
                });
            }
        }
        Ok(())
    })?;
    mutations.finish();
    Ok(())
}
