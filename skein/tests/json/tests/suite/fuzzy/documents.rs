//! The tokenizer's machine world, swept (testing-strategy.md, 8; data/json.md,
//! 6): many documents, generated and mutated, and the transcripts, cut and
//! mutated, under limits and neighbours drawn from each seed, each run
//! checked against the reference parser. This stands in for the fuzz
//! target, which waits for a nightly toolchain.
//!
//! A sweep asserts that what it injects fell (testing-strategy.md, 3): each
//! way a document can fail, a stream ending early, idle or failing, and a
//! close in each state.

use skein_json::tokenizer::Error;
use skein_json_world::generate::{self, Shape};
use skein_json_world::world::{self, Run, Settings};
use skein_json_world::{Outcome, transcript};
use skein_lib::Rng;
use skein_protocol_kit::seen::Seen;

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
    let mut tally = Seen::default();
    for round in 0..ROUNDS {
        let seed = 0x0050_0000 + round;
        let mut rng = Rng::new(seed);
        let shape = Shape {
            depth: u32::try_from(rng.between(2, 6)).unwrap(),
            width: u32::try_from(rng.between(2, 8)).unwrap(),
            string: u32::try_from(rng.between(0, 16)).unwrap(),
        };
        let tokens = generate::tokens(&mut rng, shape);
        let mut document = generate::render(&mut rng, &tokens);
        if rng.chance(600) {
            document = generate::mutate(&mut rng, &document);
        }
        let limits = world::limits(&mut rng);
        let settings = Settings::chaotic(&mut rng, limits, document.len());
        let run = world::check(&document, &settings, seed);
        record(&mut tally, &run, &settings);
    }
    tally.assert_fell(&EVERY_ERROR);
    tally.assert_fell(&EVERY_FAULT);
    tally.assert_fell(&EVERY_NEIGHBOUR);
}

#[test]
fn transcripts_cut_mutated_and_failed_read_as_the_reference_reads_them() {
    let (hostile, realistic): (Vec<_>, Vec<_>) =
        transcript::all().into_iter().partition(|transcript| transcript.name.starts_with("hostile-"));
    let mut tally = Seen::default();
    for round in 0..ROUNDS / 4 {
        let seed = 0x0051_0000 + round;
        let mut rng = Rng::new(seed);
        // Mostly the realistic ones, which are long enough to cut and
        // mutate in many places.
        let transcripts = if rng.chance(800) { &realistic } else { &hostile };
        let transcript = &transcripts[usize::try_from(rng.below(transcripts.len() as u64)).unwrap()];
        let mut document = transcript.document.clone();
        if rng.chance(500) {
            document = generate::mutate(&mut rng, &document);
        }
        let mut limits = transcript.limits;
        limits.chunk = u32::try_from(rng.between(1, 128)).unwrap();
        let settings = Settings::chaotic(&mut rng, limits, document.len());
        let run = world::check(&document, &settings, seed);
        record(&mut tally, &run, &settings);
    }
    tally.assert_fell(&EVERY_ERROR);
    tally.assert_fell(&EVERY_FAULT);
    tally.assert_fell(&EVERY_NEIGHBOUR);
}

#[test]
fn fuzzy_text_and_skip_demands_cover_each_disposition() {
    use skein_json::tokenizer::{Event, Limits};
    let limits = Limits { depth: 32, string: 4096, number: 128, chunk: 16, length: 1 << 20 };
    let mut token_count = 0;
    let mut longs = 0;
    let mut skipped = 0;
    for seed in 0..2_000 {
        let mut rng = Rng::new(seed);
        let tokens = generate::tokens(&mut rng, Shape { depth: 4, width: 4, string: 16 });
        let document = generate::render(&mut rng, &tokens);
        let settings = Settings::calm(&mut rng, limits);
        for event in world::check_demands(&document, &settings, seed).answers {
            match event {
                Event::Token(_) => token_count += 1,
                Event::Long(_) => longs += 1,
                Event::Skipped(_) => skipped += 1,
                Event::Done => {}
                event @ (Event::Failed(_) | Event::Closed) => panic!("unexpected {event:?}"),
            }
        }
    }
    assert!(token_count > 0 && longs > 0 && skipped > 0);
}
