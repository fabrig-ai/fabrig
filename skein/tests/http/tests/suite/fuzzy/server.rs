//! The server's machine world, swept (`skein/docs/design/net/http/testing.md`, 6).
//! Each case checks generated or transcript requests against the reference
//! reader and the test's writer. The gate checks outcome, rejection, refusal
//! and neighbour reach across its seeds (`skein/docs/design/testing/sweeps.md`, 4).
//! The fixed prefix grid checks cuts directly and needs no fault tally.

use std::convert::Infallible;

use super::super::requests::{serve, settings};

use skein_http::server::{Body, Error};
use skein_http::{Method, Version};
use skein_http_world::server_world::{self, Outcome, Plan, Run, Seen as Served, Settings};
use skein_http_world::{generate, request_transcript, requests};
use skein_lib::Rng;
use skein_protocol_kit::seen::Seen;
use skein_world::sweep::{cases, gate};
use skein_world::this_test;

const ROUNDS: u64 = 10_000;

/// The transcripts, longer, are swept fewer times.
const TRANSCRIPT_ROUNDS: u64 = 1_000;

fn record(tally: &mut Seen, run: &Run, settings: &Settings) {
    for seen in &run.seen {
        match seen.outcome {
            Some(Outcome::Failed(Error::Stream(fault))) => {
                tally.note("failed Stream".into());
                tally.note(format!("failed Stream({fault:?})"));
            }
            Some(outcome) => tally.note(format!("{outcome:?}")),
            None => tally.note("closed before the outcome".into()),
        }
        note_reply(tally, seen);
        for (_, refusal) in &seen.refused {
            tally.note(format!("refused {refusal:?}"));
        }
        if seen.discarded {
            tally.note("a body discarded".into());
        }
        if seen.reply_failed.is_some() {
            tally.note("the reply failed".into());
        }
        if seen.body_failed.is_some() {
            tally.note(format!("the body failed with {:?}", seen.body_failed));
        }
        if seen.outcome == Some(Outcome::Failed(Error::BodyTooLong)) {
            let answered = seen.sent.starts_with(b"HTTP/1.1 413");
            tally.note(format!("a chunked body past the limit, answered: {answered}"));
        }
    }
    tally.note(format!("closed while waiting for {:?}", run.closed_while));
    let fell = run.fell;
    for (flag, what) in [
        (fell.idle_end, "an end with nothing demanded"),
        (fell.crossed_end, "a demand crossed the end"),
        (fell.withdrew, "a body demand withdrawn"),
        (fell.gave_up, "a body given up"),
        (fell.late_answer, "an answer after the withdrawal"),
        (fell.failed_after_end, "a failure after the end"),
        (fell.reused, "a connection reused"),
        (fell.pipelined, "requests pipelined"),
        (fell.continued, "a 100 (Continue)"),
        (fell.tired, "a client tired of waiting for a 100"),
        (fell.head_waited, "a head waited for a discard"),
        (fell.room_after_end, "room after the end"),
        (fell.reply_withdrawn, "a reply's demand withdrawn"),
        (fell.crossed_withdrawal, "a withdrawal crossed a piece"),
        (fell.untouched, "a body untouched, discarded late"),
        (fell.finished_over_grant, "a Finish in place of a grant's Send"),
    ] {
        if flag {
            tally.note(what.into());
        }
    }
    if let Some(waiting) = fell.failed_while {
        tally.note(format!("failed while waiting for {waiting:?}"));
    }
    if settings.cut.is_some() {
        tally.note("a stream ended early".into());
    }
}

/// What the reply came to: one to `HEAD`, one in chunks to HTTP/1.0 run
/// to the end of the stream, or a final response that went before a 100
/// (Continue) the client waited for.
fn note_reply(tally: &mut Seen, seen: &Served) {
    let (Some(call), Some((response, _)), Some(Outcome::Done(_))) = (&seen.call, &seen.response, seen.outcome) else {
        return;
    };
    if call.method == Method::Head {
        tally.note("a response to HEAD".into());
    }
    if call.version == Version::Http10
        && response.body == Body::Chunked
        && requests::sends_body(call.method, response)
        && seen.finished
    {
        tally.note("a reply in chunks to HTTP/1.0, to the end of the stream".into());
    }
    let expects = call.version == Version::Http11
        && call.body != Body::None
        && call.body != Body::Length(0)
        && call.headers.iter().any(|header| {
            header.is(b"expect") && header.value.to_ascii_lowercase().windows(12).any(|w| w == b"100-continue")
        });
    if expects && !seen.sent.starts_with(b"HTTP/1.1 100") {
        tally.note("a final response before the 100 that was due".into());
    }
}

/// `count` requests and the side above's plans, from `rng`.
fn scenario(rng: &mut Rng, count: usize) -> (Vec<u8>, Vec<Plan>) {
    let mut client = Vec::new();
    let mut plans = Vec::new();
    for _ in 0..count {
        client.extend(requests::request(rng));
        plans.push(server_world::plan(rng));
    }
    (client, plans)
}

#[test]
fn generated_mutated_and_corrupted_requests_are_served_as_the_reference_reads_them_whatever_the_neighbours() {
    let mut tally = Seen::default();
    let mut mutations = generate::MutationBatch::default();
    let at_gate = gate();
    let base = 0x0090_0000;
    let test = this_test!(
        "generated_mutated_and_corrupted_requests_are_served_as_the_reference_reads_them_whatever_the_neighbours"
    );
    cases::<_, Infallible, _>(test, base..base + ROUNDS, |seed| {
        let mut rng = Rng::new(seed);
        let count = usize::try_from(rng.between(1, 4)).unwrap();
        let (client, plans) = scenario(&mut rng, count);
        let client = if rng.chance(300) {
            generate::mutate(&mut rng, &client)
        } else if rng.chance(400) {
            requests::corrupt(&mut rng, &client)
        } else {
            client.into()
        };
        let limits = server_world::limits(&mut rng);
        let settings = Settings::chaotic(&mut rng, limits, client.input().len());
        let run = mutations.judge(client, |client| server_world::check(&client, &plans, &settings, seed));
        if at_gate {
            record(&mut tally, &run, &settings);
        }
        Ok(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    mutations.finish();
    if !at_gate {
        return;
    }
    let mut every: Vec<String> = [
        "Done(Keep)",
        "Done(Close)",
        "Ended",
        "Failed(Truncated)",
        "Failed(ChunkSize)",
        "Failed(Chunk)",
        "Failed(Trailer)",
        "Failed(Extensions)",
        "Failed(BodyTooLong)",
        "a chunked body past the limit, answered: true",
        "a chunked body past the limit, answered: false",
        "failed Stream",
        "failed Stream(Reset)",
        "failed Stream(Invalid)",
        "failed Stream(Other)",
        "closed before the outcome",
        "a response to HEAD",
        "a reply in chunks to HTTP/1.0, to the end of the stream",
        "a final response before the 100 that was due",
        "refused Status",
        "refused Name",
        "refused Value",
        "refused Reserved",
        "refused Body",
        "refused TooLong",
        "a body discarded",
        "the reply failed",
        "the body failed with Some(Other)",
        "the body failed with Some(Invalid)",
        "the body failed with Some(Reset)",
        "closed while waiting for Next",
        "closed while waiting for Room",
        "closed while waiting for Request",
        "closed while waiting for Body",
        "closed while waiting for Above",
        "closed while waiting for Close",
        "an end with nothing demanded",
        "a demand crossed the end",
        "a body demand withdrawn",
        "a body given up",
        "an answer after the withdrawal",
        "a failure after the end",
        "a connection reused",
        "requests pipelined",
        "a 100 (Continue)",
        "a client tired of waiting for a 100",
        "a head waited for a discard",
        "room after the end",
        "a reply's demand withdrawn",
        "a withdrawal crossed a piece",
        "a body untouched, discarded late",
        "a Finish in place of a grant's Send",
        "failed while waiting for Next",
        "failed while waiting for Room",
        "failed while waiting for Request",
        "failed while waiting for Body",
        "failed while waiting for Above",
        "failed while waiting for Close",
        "failed while waiting for Nothing",
        "a stream ended early",
    ]
    .iter()
    .map(|what| (*what).to_string())
    .collect();
    for rejection in server_world::rejections() {
        every.push(format!("Failed(Rejected({rejection:?}))"));
    }
    tally.assert_fell(&every);
}

#[test]
fn request_transcripts_cut_mutated_and_failed_are_served_as_the_reference_reads_them() {
    let transcripts = request_transcript::all();
    let mut tally = Seen::default();
    let mut mutations = generate::MutationBatch::default();
    let at_gate = gate();
    let base = 0x0091_0000;
    let test = this_test!("request_transcripts_cut_mutated_and_failed_are_served_as_the_reference_reads_them");
    cases::<_, Infallible, _>(test, base..base + TRANSCRIPT_ROUNDS, |seed| {
        let mut rng = Rng::new(seed);
        let transcript = &transcripts[usize::try_from(rng.below(transcripts.len() as u64)).unwrap()];
        let client = transcript.bytes.clone();
        let client = if rng.chance(500) { generate::mutate(&mut rng, &client) } else { client.into() };
        let plans: Vec<Plan> = (0..4).map(|_| server_world::plan(&mut rng)).collect();
        let settings = Settings::chaotic(&mut rng, transcript.limits, client.input().len());
        let run = mutations.judge(client, |client| server_world::check(&client, &plans, &settings, seed));
        if at_gate {
            record(&mut tally, &run, &settings);
        }
        Ok(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    mutations.finish();
    if at_gate {
        tally.assert_fell(&[
            "Done(Keep)",
            "Done(Close)",
            "Ended",
            "Failed(Truncated)",
            "failed Stream",
            "closed before the outcome",
        ]);
    }
}

#[test]
fn a_request_transcript_cut_short_anywhere_comes_to_what_the_reference_reads_of_its_prefix() {
    for transcript in request_transcript::all() {
        let len = transcript.bytes.len();
        let step = (len / 40).max(1);
        for cut in (0..=len).step_by(step) {
            let seed = cut as u64;
            let settings = Settings { cut: Some(cut), ..settings(&transcript, seed) };
            let _ = serve(&transcript, &settings, seed);
        }
    }
}
