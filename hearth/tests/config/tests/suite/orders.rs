//! Orders beside the decoder (hearth/docs/design/domain/render/README.md, 5.5).
use hearth_config_world::{
    generate,
    world::{self, Step},
};
use hearth_protocol_config::{json::Event, limits::Limits};
use skein_lib::Rng;

// A duplicate key, a fraction, trailing bytes, a cut and something other than an object.
const REFUSED: &[&[u8]] = &[br#"{"a":1,"a":2}"#, br#"{"a":1.2}"#, br#"{"a":1} x"#, br#"{"a":[1,"#, b"[]"];

fn documents(mut check: impl FnMut(&[Step])) {
    let limits = Limits { document: 4096, depth: 16, keys: 32, settings: 16 };
    let mut run = |document: &[u8], seed, decoded| {
        let steps = world::steps(document, &limits, seed);
        let outcome = steps.iter().flat_map(|step| &step.above).find(|event| !matches!(event, Event::Closed));
        assert!(
            matches!((decoded, outcome), (true, Some(Event::Decoded(_))) | (false, Some(Event::Failed(_)))),
            "expected outcome, seed {seed}, document {document:?}: {steps:?}"
        );
        check(&steps);
    };
    for seed in 0..16 {
        let document = generate::document(&mut Rng::new(seed), &limits);
        run(&document, seed, true);
    }
    for document in REFUSED {
        for seed in 0..4 {
            run(document, seed, false);
        }
    }
}

#[test]
fn a_decoder_has_one_read_out_whatever_the_cuts() {
    documents(|steps| {
        for step in steps {
            assert!(step.below.len() <= 1, "at most one read out: {step:?}");
        }
    });
}

#[test]
fn a_delivery_gives_a_decoder_s_caller_at_most_one_event() {
    documents(|steps| {
        for step in steps {
            assert!(step.above.len() <= 1, "at most one event a delivery: {step:?}");
        }
    });
}

#[test]
fn a_decoder_s_outcome_comes_once_and_closed_last() {
    documents(|steps| {
        let items: Vec<_> = steps.iter().flat_map(|step| &step.above).collect();
        assert!(
            matches!(items.as_slice(), [Event::Decoded(_) | Event::Failed(_), Event::Closed]),
            "one outcome then one Closed: {steps:?}"
        );
        for step in steps {
            if step.above.iter().any(|event| matches!(event, Event::Decoded(_) | Event::Failed(_))) {
                assert!(step.below.is_empty(), "no read out with the outcome: {step:?}");
            }
        }
        for step in steps.iter().rev().take(3) {
            assert!(step.above.is_empty() && step.below.is_empty(), "strays emit nothing: {step:?}");
        }
    });
}
