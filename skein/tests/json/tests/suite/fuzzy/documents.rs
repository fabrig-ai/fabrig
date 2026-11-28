//! The tokenizer's machine world (skein/docs/design/data/json.md, 6): each
//! case checks a generated or transcript document against the reference
//! under drawn limits and neighbours. The gate seeds also check error,
//! fault and neighbour reach (skein/docs/design/testing/sweeps.md, 4).

use skein_json::tokenizer::Error;
use skein_json_world::generate::{self, Shape};
use skein_json_world::world::{self, Run, Settings};
use skein_json_world::{Outcome, transcript};
use skein_lib::Rng;
use skein_protocol_kit::seen::Seen;
use skein_world::sweep::{cases, gate};
use skein_world::this_test;
use std::convert::Infallible;

const ROUNDS: u64 = 20_000;

fn record(tally: &mut Seen, run: &Run, settings: &Settings) {
    match run.outcome {
        Some(Outcome::Failed(Error::Stream(_))) => tally.note("failed Stream".into()),
        Some(outcome) => tally.note(format!("{outcome:?}")),
        None => tally.note("closed before the outcome".into()),
    }
    tally.note(format!("closed while waiting for {:?}", run.closed_while));
    let fell = run.fell;
    if fell.idle_end {
        tally.note("an end with nothing demanded".into());
    }
    if fell.crossed_end {
        tally.note("a demand crossed the end".into());
    }
    if fell.byte_then_end {
        tally.note("an end after a number's last byte".into());
    }
    if fell.late_delivery {
        tally.note("a delivery after the close".into());
    }
    if fell.failed_after_end {
        tally.note("a failure after the end".into());
    }
    if let Some(waiting) = fell.failed_while {
        tally.note(format!("failed while waiting for {waiting:?}"));
    }
    if let Some(fault) = run.failed {
        tally.note(format!("failed with {fault:?}"));
    }
    if settings.cut.is_some() {
        tally.note("a stream ended early".into());
    }
    if run.held_back == settings.cap {
        tally.note("the stream below filled while the side above stopped".into());
    }
}

const EVERY_ERROR: [&str; 13] = [
    "Failed(Unexpected)",
    "Failed(Trailing)",
    "Failed(TooLong)",
    "Failed(TooDeep)",
    "Failed(StringTooLong)",
    "Failed(NumberTooLong)",
    "Failed(Number)",
    "Failed(Escape)",
    "Failed(Surrogate)",
    "Failed(Utf8)",
    "Failed(Control)",
    "Failed(Truncated)",
    "failed Stream",
];

const EVERY_FAULT: [&str; 10] = [
    "an end after a number's last byte",
    "a delivery after the close",
    "a failure after the end",
    "failed while waiting for Next",
    "failed while waiting for Bytes",
    "failed while waiting for Close",
    "failed while waiting for Nothing",
    "failed with Reset",
    "failed with Invalid",
    "failed with Other",
];

const EVERY_NEIGHBOUR: [&str; 9] = [
    "Done",
    "closed before the outcome",
    "closed while waiting for Next",
    "closed while waiting for Bytes",
    "closed while waiting for Close",
    "an end with nothing demanded",
    "a demand crossed the end",
    "a stream ended early",
    "the stream below filled while the side above stopped",
];

#[test]
fn generated_and_mutated_documents_read_as_the_reference_reads_them_whatever_the_neighbours() {
    let at_gate = gate();
    let mut tally = Seen::default();
    let mut mutations = generate::MutationBatch::default();
    cases(
        this_test!("generated_and_mutated_documents_read_as_the_reference_reads_them_whatever_the_neighbours"),
        0x0050_0000..0x0050_0000 + ROUNDS,
        |seed| {
            let mut rng = Rng::new(seed);
            let shape = Shape {
                depth: u32::try_from(rng.between(2, 6)).unwrap(),
                width: u32::try_from(rng.between(2, 8)).unwrap(),
                string: u32::try_from(rng.between(0, 16)).unwrap(),
            };
            let tokens = generate::tokens(&mut rng, shape);
            let document = generate::render(&mut rng, &tokens);
            let document = if rng.chance(600) { generate::mutate(&mut rng, &document) } else { document.into() };
            let limits = world::limits(&mut rng);
            let settings = Settings::chaotic(&mut rng, limits, document.input().len());
            let run = mutations.judge(document, |document| world::check(&document, &settings, seed));
            if at_gate {
                record(&mut tally, &run, &settings);
            }
            Ok::<(), Infallible>(())
        },
    )
    .unwrap_or_else(|case| panic!("{case}"));
    mutations.finish();
    if at_gate {
        tally.assert_fell(&EVERY_ERROR);
        tally.assert_fell(&EVERY_FAULT);
        tally.assert_fell(&EVERY_NEIGHBOUR);
    }
}

#[test]
fn transcripts_cut_mutated_and_failed_read_as_the_reference_reads_them() {
    let (hostile, realistic): (Vec<_>, Vec<_>) =
        transcript::all().into_iter().partition(|transcript| transcript.name.starts_with("hostile-"));
    let at_gate = gate();
    let mut tally = Seen::default();
    let mut mutations = generate::MutationBatch::default();
    cases(
        this_test!("transcripts_cut_mutated_and_failed_read_as_the_reference_reads_them"),
        0x0051_0000..0x0051_0000 + ROUNDS / 4,
        |seed| {
            let mut rng = Rng::new(seed);
            // Mostly the realistic ones, which are long enough to cut and
            // mutate in many places.
            let transcripts = if rng.chance(800) { &realistic } else { &hostile };
            let transcript = &transcripts[usize::try_from(rng.below(transcripts.len() as u64)).unwrap()];
            let document = transcript.document.clone();
            let document = if rng.chance(500) { generate::mutate(&mut rng, &document) } else { document.into() };
            let mut limits = transcript.limits;
            limits.chunk = u32::try_from(rng.between(1, 128)).unwrap();
            let settings = Settings::chaotic(&mut rng, limits, document.input().len());
            let run = mutations.judge(document, |document| world::check(&document, &settings, seed));
            if at_gate {
                record(&mut tally, &run, &settings);
            }
            Ok::<(), Infallible>(())
        },
    )
    .unwrap_or_else(|case| panic!("{case}"));
    mutations.finish();
    if at_gate {
        tally.assert_fell(&EVERY_ERROR);
        tally.assert_fell(&EVERY_FAULT);
        tally.assert_fell(&EVERY_NEIGHBOUR);
    }
}

#[test]
fn fuzzy_text_and_skip_demands_cover_each_disposition() {
    use skein_json::tokenizer::{Event, Limits};
    let limits = Limits { depth: 32, string: 4096, number: 128, chunk: 16, length: 1 << 20 };
    let at_gate = gate();
    let mut token_count = 0;
    let mut longs = 0;
    let mut skipped = 0;
    cases(this_test!("fuzzy_text_and_skip_demands_cover_each_disposition"), 0..2_000, |seed| {
        let mut rng = Rng::new(seed);
        let tokens = generate::tokens(&mut rng, Shape { depth: 4, width: 4, string: 16 });
        let document = generate::render(&mut rng, &tokens);
        let settings = Settings::calm(&mut rng, limits);
        for event in world::check_demands(&document, &settings, seed).answers {
            match event {
                Event::Token(_) if at_gate => token_count += 1,
                Event::Long(_) if at_gate => longs += 1,
                Event::Skipped(_) if at_gate => skipped += 1,
                Event::Token(_) | Event::Long(_) | Event::Skipped(_) | Event::Done => {}
                event @ (Event::Failed(_) | Event::Closed) => panic!("unexpected {event:?}"),
            }
        }
        Ok::<(), Infallible>(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    if at_gate {
        assert!(token_count > 0 && longs > 0 && skipped > 0);
    }
}
