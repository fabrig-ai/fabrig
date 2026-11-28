use hearth_protocol_template::expression;
use hearth_template_world::{check, fixtures, generate, reference};
use skein_world::{
    sweep::{cases, gate},
    this_test,
};
use std::collections::BTreeSet;

use super::{EXPRESSIONS, EXPRESSIONS_AT_LIMITS, SweepResult, offset_rng};

#[test]
fn cut_and_mutated_texts_compile_as_the_reference_compiles_them() -> SweepResult {
    let tally = gate();
    let mut fell = BTreeSet::new();
    let mut mutations = generate::MutationBatch::default();
    cases(this_test!("cut_and_mutated_texts_compile_as_the_reference_compiles_them"), 0..10_000, |seed| {
        let mut rng = offset_rng(seed, EXPRESSIONS);
        let (limits, valid) = (fixtures::limits(&mut rng), generate::text(&mut rng, fixtures::WIDE));
        let text = match rng.below(4) {
            0 => valid.into(),
            1 => generate::cut(&mut rng, &valid).into(),
            _ => generate::mutate(&mut rng, &valid),
        };
        mutations.judge(text, |text| {
            let (got, want) = (expression::compile(&text, &limits), reference::compile(&text, &limits));
            assert_eq!(got.as_ref().map_err(|e| e.at), want.as_ref().map_err(|at| *at), "{text:?}");
            match &got {
                Ok(e) => check::invariants(e, &text),
                Err(e) if tally => {
                    fell.insert(fixtures::kind(&e.kind));
                }
                Err(_) => {}
            }
        });
        Ok(())
    })?;
    if tally {
        fixtures::assert_fell(&fell, fixtures::EVERY_KIND);
    }
    mutations.finish();
    Ok(())
}

#[test]
fn texts_at_the_limits_cut_and_mutated_compile_as_the_reference_does() -> SweepResult {
    let texts = fixtures::at_the_limits();
    let mut mutations = generate::MutationBatch::default();
    cases(this_test!("texts_at_the_limits_cut_and_mutated_compile_as_the_reference_does"), 0..500, |seed| {
        let mut rng = offset_rng(seed, EXPRESSIONS_AT_LIMITS);
        let index = usize::try_from(rng.below(texts.len() as u64)).expect("fixture index");
        let (valid, what, _) = &texts[index];
        let text =
            if rng.below(2) == 0 { generate::cut(&mut rng, valid).into() } else { generate::mutate(&mut rng, valid) };
        mutations.judge(text, |text| {
            let got = expression::compile(&text, &fixtures::LIMITS);
            let want = reference::compile(&text, &fixtures::LIMITS);
            assert_eq!(got.as_ref().map_err(|error| error.at), want.as_ref().map_err(|at| *at), "{what}: {text:?}");
            if let Ok(expression) = got {
                check::invariants(&expression, &text);
            }
        });
        Ok(())
    })?;
    mutations.finish();
    Ok(())
}
