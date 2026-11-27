use hearth_protocol_template::{limits::SHIPPED, template};
use hearth_template_world::{check, generate, reference};
use hearth_test_fixtures::golden;
use skein_world::{sweep::cases, this_test};

use super::{SweepResult, offset_rng};

#[test]
fn golden_templates_cut_and_mutated_compile_as_the_reference_compiles_them() -> SweepResult {
    let corpus = golden::corpus("j2");
    let mut mutations = generate::MutationBatch::default();
    cases(
        this_test!("golden_templates_cut_and_mutated_compile_as_the_reference_compiles_them"),
        golden::CASES,
        |seed| {
            let mut rng = offset_rng(seed, golden::SEED);
            for (path, input) in &corpus {
                for text in [
                    input.clone().into(),
                    generate::cut(&mut rng, input).into(),
                    generate::template::mutate(&mut rng, input),
                ] {
                    mutations.judge(text, |text| {
                        let compiled = template::compile(&text, &SHIPPED);
                        assert_eq!(
                            compiled.as_ref().map_err(|error| error.at),
                            reference::template::compile(&text, &SHIPPED).as_ref().map_err(|at| *at),
                            "{}",
                            path.display()
                        );
                        if let Ok(compiled) = compiled {
                            check::template(&compiled, &text);
                        }
                    });
                }
            }
            Ok(())
        },
    )?;
    mutations.finish();
    Ok(())
}
