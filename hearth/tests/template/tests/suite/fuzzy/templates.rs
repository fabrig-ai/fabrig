use hearth_protocol_template::template;
use hearth_template_world::{check, fixtures, generate, reference};
use skein_world::{
    sweep::{cases, gate},
    this_test,
};
use std::collections::BTreeSet;

use super::{STRINGS, SweepResult, TEMPLATES, TEMPLATES_AT_LIMITS, offset_rng};

#[test]
fn cut_and_mutated_templates_compile_as_the_reference_compiles_them() -> SweepResult {
    let tally = gate();
    let mut fell = BTreeSet::new();
    let mut mutations = generate::MutationBatch::default();
    cases(this_test!("cut_and_mutated_templates_compile_as_the_reference_compiles_them"), 0..5_000, |seed| {
        let mut rng = offset_rng(seed, TEMPLATES);
        let (limits, valid) =
            (fixtures::limits(&mut rng), generate::template::text(&mut rng, fixtures::WIDE_TEMPLATES));
        let text = match rng.below(4) {
            0 => valid.into(),
            1 => generate::cut(&mut rng, &valid).into(),
            _ => generate::template::mutate(&mut rng, &valid),
        };
        mutations.judge(text, |text| {
            let (got, want) = (template::compile(&text, &limits), reference::template::compile(&text, &limits));
            assert_eq!(got.as_ref().map_err(|e| e.at), want.as_ref().map_err(|at| *at), "{text:?}");
            match &got {
                Ok(e) => check::template(e, &text),
                Err(e) if tally => {
                    fell.insert(fixtures::kind(&e.kind));
                }
                Err(_) => {}
            }
        });
        Ok(())
    })?;
    if tally {
        fixtures::assert_fell(&fell, fixtures::TEMPLATE_KINDS);
    }
    mutations.finish();
    Ok(())
}

#[test]
fn cut_and_mutated_strings_interpolate_as_the_reference_reads_them() -> SweepResult {
    let tally = gate();
    let mut fell = BTreeSet::new();
    let mut mutations = generate::MutationBatch::default();
    cases(this_test!("cut_and_mutated_strings_interpolate_as_the_reference_reads_them"), 0..2_000, |seed| {
        let mut rng = offset_rng(seed, STRINGS);
        let (limits, valid) = (fixtures::limits(&mut rng), generate::template::string(&mut rng, fixtures::WIDE));
        let text = match rng.below(4) {
            0 => valid.into(),
            1 => generate::cut(&mut rng, &valid).into(),
            _ => generate::template::mutate(&mut rng, &valid),
        };
        mutations.judge(text, |text| {
            let got = hearth_protocol_template::interpolation::compile(&text, &limits);
            let want = reference::template::interpolate(&text, &limits);
            assert_eq!(got.as_ref().map_err(|e| e.at), want.as_ref().map_err(|at| *at), "{text:?}");
            if tally && let Err(error) = got {
                fell.insert(fixtures::kind(&error.kind));
            }
        });
        Ok(())
    })?;
    if tally {
        fixtures::assert_fell(&fell, fixtures::STRING_KINDS);
    }
    mutations.finish();
    Ok(())
}

#[test]
fn templates_at_the_limits_cut_and_mutated_compile_as_the_reference_does() -> SweepResult {
    let mut mutations = generate::MutationBatch::default();
    cases(this_test!("templates_at_the_limits_cut_and_mutated_compile_as_the_reference_does"), 0..500, |seed| {
        let mut rng = offset_rng(seed, TEMPLATES_AT_LIMITS);
        let limits = hearth_protocol_template::limits::Limits {
            template: u32::try_from(rng.between(64, 4096)).expect("tiny text limit"),
            statements: u32::try_from(rng.between(1, 8)).expect("tiny statement limit"),
            ..fixtures::LIMITS
        };
        let cases = fixtures::templates_at_the_limits(&limits);
        let index = usize::try_from(rng.below(cases.len() as u64)).expect("fixture index");
        let (valid, what, _) = &cases[index];
        let text = if rng.below(2) == 0 {
            generate::cut(&mut rng, valid).into()
        } else {
            generate::template::mutate(&mut rng, valid)
        };
        mutations.judge(text, |text| {
            let got = template::compile(&text, &limits);
            let want = reference::template::compile(&text, &limits);
            assert_eq!(got.as_ref().map_err(|e| e.at), want.as_ref().map_err(|at| *at), "{what}");
            if let Ok(compiled) = got {
                check::template(&compiled, &text);
            }
        });
        Ok(())
    })?;
    mutations.finish();
    Ok(())
}
