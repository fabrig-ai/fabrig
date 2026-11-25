//! The event stream reader's machine world, swept (testing-strategy.md, 8;
//! http.md, 6): many streams, generated and mutated, and the bodies of the
//! transcripts that hold events, cut and mutated, under limits and
//! neighbours drawn from each seed, each run checked against the reference
//! reader. This stands in for the fuzz target, which waits for a nightly
//! toolchain.
//!
//! A sweep asserts that what it injects fell (testing-strategy.md, 3): each
//! way a stream can fail, each way it can end or fail below, a close in
//! each state, a CR's LF delivered alone, and a line read in pieces.

use skein_http::sse::{self, Error, Event, Limits, Reader, Request, Waiting};
use skein_http_world::reference::Ending;
use skein_http_world::sse_world::{self, Run, Settings};
use skein_http_world::{generate, transcript};
use skein_lib::stream::{Delimiter, Down, Fault, Read, Up};
use skein_lib::{Env, Queue, Rng, Time, Wall, usize_of};
use skein_protocol_kit::random::draw;
use skein_protocol_kit::seen::Seen;

const ROUNDS: u64 = 12_000;

fn record(tally: &mut Seen, run: &Run, settings: &Settings) {
    match run.outcome {
        Some(Ending::Failed(Error::Stream(_))) => tally.note("failed Stream".into()),
        Some(outcome) => tally.note(format!("{outcome:?}")),
        None => tally.note("closed before the outcome".into()),
    }
    tally.note(format!("closed while waiting for {:?}", run.closed_while));
    let fell = run.fell;
    for (flag, what) in [
        (fell.idle_end, "an end with nothing demanded"),
        (fell.crossed_end, "a demand crossed the end"),
        (fell.late_delivery, "a delivery after the close"),
        (fell.failed_after_end, "a failure after the end"),
        (fell.lone_lf, "a lone LF after a CR"),
        (fell.long_line, "a line longer than a chunk"),
    ] {
        if flag {
            tally.note(what.into());
        }
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

const EVERY: [&str; 26] = [
    "Ended",
    "Failed(LineTooLong)",
    "Failed(EventTooLong)",
    "Failed(FieldTooLong)",
    "failed Stream",
    "closed before the outcome",
    "closed while waiting for Next",
    "closed while waiting for Bytes",
    "closed while waiting for Above",
    "closed while waiting for Close",
    "an end with nothing demanded",
    "a demand crossed the end",
    "a delivery after the close",
    "a failure after the end",
    "a lone LF after a CR",
    "a line longer than a chunk",
    "failed while waiting for Next",
    "failed while waiting for Bytes",
    "failed while waiting for Above",
    "failed while waiting for Close",
    "failed while waiting for Nothing",
    "failed with Reset",
    "failed with Invalid",
    "failed with Other",
    "a stream ended early",
    "the stream below filled while the side above stopped",
];

#[test]
fn generated_and_mutated_streams_read_as_the_reference_reads_them_whatever_the_neighbours() {
    let mut tally = Seen::default();
    let mut mutations = generate::MutationBatch::default();
    for round in 0..ROUNDS {
        let seed = 0x0080_0000 + round;
        let mut rng = Rng::new(seed);
        let stream = generate::events(&mut rng);
        let stream = if rng.chance(500) { generate::mutate(&mut rng, &stream) } else { stream.into() };
        let limits = sse_world::limits(&mut rng);
        let settings = Settings::chaotic(&mut rng, limits, stream.input().len());
        let run = mutations.judge(stream, |stream| sse_world::check(&stream, &settings, seed));
        record(&mut tally, &run, &settings);
    }
    mutations.finish();
    tally.assert_fell(&EVERY);
}

#[test]
fn the_transcripts_events_cut_mutated_and_failed_read_as_the_reference_reads_them() {
    let mut bodies = Vec::new();
    for transcript in transcript::all() {
        let decoded = transcript::read(&transcript);
        if let (Some(_), Some(head)) = (decoded.events, decoded.head) {
            let response =
                skein_http_world::reference::response(&transcript.bytes, transcript.method, false, &transcript.limits);
            assert!(head.status > 0);
            bodies.push((response.body, transcript.sse));
        }
    }
    assert!(bodies.len() >= 4, "the transcripts hold event streams");
    let mut tally = Seen::default();
    let mut mutations = generate::MutationBatch::default();
    for round in 0..ROUNDS / 6 {
        let seed = 0x0081_0000 + round;
        let mut rng = Rng::new(seed);
        let (body, limits) = &bodies[usize::try_from(rng.below(bodies.len() as u64)).unwrap()];
        let stream = body.clone();
        let stream = if rng.chance(500) { generate::mutate(&mut rng, &stream) } else { stream.into() };
        let mut limits = *limits;
        limits.chunk = u32::try_from(rng.between(1, 128)).unwrap();
        let settings = Settings::chaotic(&mut rng, limits, stream.input().len());
        let run = mutations.judge(stream, |stream| sse_world::check(&stream, &settings, seed));
        record(&mut tally, &run, &settings);
    }
    mutations.finish();
    tally.assert_fell(&["Ended", "failed Stream", "closed before the outcome", "a stream ended early"]);
}

/// Cuts in the bytes already delivered to the reader, independent of the
/// transport's line-scan scheduling. Line ends and chunk caps also cut.
#[derive(Clone, Copy, Debug)]
enum Pieces {
    Bytes,
    Split(usize),
    Mask(u64),
}

impl Pieces {
    fn end(self, start: usize, cap: usize) -> usize {
        match self {
            Pieces::Bytes => start + 1,
            Pieces::Split(at) if at > start => at.min(cap),
            Pieces::Split(_) => cap,
            Pieces::Mask(mask) => (start + 1..cap).find(|&at| mask & (1 << (at - 1)) != 0).unwrap_or(cap),
        }
    }
}

struct SplitCase {
    input: Vec<u8>,
    limits: Limits,
    read: Read,
    fault: Option<Fault>,
}

#[derive(PartialEq, Eq, Debug)]
struct Trace {
    events: Vec<Event>,
    retry: Option<u64>,
    last_id: Vec<u8>,
}

fn split_trace(case: &SplitCase, pieces: Pieces) -> Trace {
    let SplitCase { input, limits, read, fault } = case;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut reader = Reader::new(limits);
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    let mut events = Vec::new();
    let chunk = usize_of(limits.chunk);
    let mut at = 0;
    let mut terminal = Some(fault.map_or(Up::End, Up::Failed));
    // Each body delivery and each data answer consumes at least one byte,
    // with a Next, end, dispatch or close taking only a step at its boundary.
    for _ in 0..4 * (input.len() + 1) + 4 {
        let request = match reader.waiting() {
            Waiting::Next => Some(Request::Next),
            Waiting::Above => Some(Request::Data(Down::Demand { read: *read, room: 0 })),
            Waiting::Close => Some(Request::Close),
            Waiting::Nothing => {
                assert!(below.is_empty());
                return Trace { events, retry: reader.retry(), last_id: reader.last_event_id().to_vec() };
            }
            Waiting::Bytes => {
                assert_eq!(below.pop(), Some(Down::Demand { read: Read::Line { max: limits.chunk }, room: 0 }));
                let delivery = if at == input.len() {
                    terminal.take().expect("one body terminal")
                } else {
                    let cap = input.len().min(at + chunk);
                    let end = pieces.end(at, cap);
                    let line = &input[at..end];
                    let count =
                        line.iter().position(|byte| matches!(byte, b'\r' | b'\n')).map_or(line.len(), |i| i + 1);
                    at += count;
                    Up::Bytes(line[..count].into())
                };
                sse::up(&mut reader, &env, delivery, &mut above, &mut below);
                None
            }
        };
        if let Some(request) = request {
            assert!(below.is_empty());
            sse::down(&mut reader, &env, request, &mut above, &mut below);
        }
        while let Some(event) = above.pop() {
            events.push(event);
        }
    }
    panic!("a finite stream settles under {pieces:?}");
}

fn split_case(seed: u64, short: bool) -> SplitCase {
    const VALUE: &[u8] = b"aba :\0\xff";
    let mut rng = Rng::new(seed);
    let mut input = b"data:".to_vec();
    if rng.chance(500) {
        input.push(b' ');
    }
    let size = if short { rng.between(2, 4) } else { rng.between(8, 20) };
    for _ in 0..size {
        let at = usize::try_from(rng.below(VALUE.len() as u64)).expect("an index into the alphabet");
        input.push(VALUE[at]);
    }
    let end: &[u8] = match rng.below(3) {
        0 => b"\n",
        1 => b"\r",
        _ => b"\r\n",
    };
    input.extend_from_slice(end);
    if !short {
        for line in [b"data".as_slice(), b"data: ababa", b"id: gen", b"event: named", b"retry: 27"] {
            input.extend_from_slice(line);
            input.extend_from_slice(end);
        }
    }
    input.extend_from_slice(end);
    let mut limits = Limits { line: 32, event: 128, field: 16, chunk: draw(&mut rng, 4, 32) };
    match rng.below(8) {
        0 => limits.line = 9, // Inside the first long data value.
        1 => limits.event = 10,
        2 => limits.field = 2,
        3 => {
            limits.line = 9;
            limits.event = 9; // Both limits can fail on the same byte.
        }
        _ => {}
    }
    let max = draw(&mut rng, 3, limits.chunk);
    let read = match rng.below(6) {
        0 => Read::Fill(max),
        1 => Read::Line { max },
        2 => Read::Arrived { max },
        3 => Read::Scan { until: Delimiter::new(b"aba").expect("fits in a scan"), max },
        4 => Read::Scan { until: Delimiter::new(b"\nab").expect("fits in a scan"), max },
        _ => Read::Nothing,
    };
    let fault = if rng.chance(150) {
        let at = usize::try_from(rng.below(input.len() as u64)).expect("a cut within the input");
        input.truncate(at);
        Some(Fault::Reset)
    } else {
        None
    };
    SplitCase { input, limits, read, fault }
}

#[test]
fn every_split_of_generated_deliveries_matches_single_byte_deliveries() {
    const BASE: u64 = 0x0082_0000;
    const SHORT: u64 = 32;
    const CASES: u64 = 128;
    let mut outcomes = Seen::default();
    for round in 0..CASES {
        let seed = BASE + round;
        let short = round < SHORT;
        let case = split_case(seed, short);
        // Single-byte deliveries cannot use the run path.
        let bytewise = split_trace(&case, Pieces::Bytes);
        outcomes.note(format!("{:?}", bytewise.events[bytewise.events.len() - 2]));
        let check = |pieces| {
            assert_eq!(split_trace(&case, pieces), bytewise, "seed {seed:#x}, {pieces:?}, read {:?}", case.read);
        };
        if short {
            // Enumerate every subset of optional cuts. Ends of lines and
            // the end of the delivery are always cut, so their bits add no case.
            let optional = case.input[..case.input.len().saturating_sub(1)]
                .iter()
                .enumerate()
                .filter(|(_, byte)| !matches!(byte, b'\r' | b'\n'))
                .fold(0, |mask, (at, _)| mask | (1 << at));
            let mut mask = optional;
            loop {
                check(Pieces::Mask(mask));
                if mask == 0 {
                    break;
                }
                mask = (mask - 1) & optional;
            }
        } else {
            for at in 0..=case.input.len() {
                check(Pieces::Split(at));
            }
        }
    }
    outcomes.assert_fell(&[
        "Ended",
        "Failed(LineTooLong)",
        "Failed(EventTooLong)",
        "Failed(FieldTooLong)",
        "Failed(Stream(Reset))",
    ]);
}
