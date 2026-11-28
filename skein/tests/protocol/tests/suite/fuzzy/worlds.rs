//! Protocol cases draw a scenario, caps and a wire from their seed
//! (skein/docs/design/testing/sweeps.md, 4.2–4.5). In every mode the
//! referee holds each run to what the other end's top sent.
//!
//! At the gate the sweep asserts that what it aims at fell: a
//! writer held back by a slow reader, an upload held back by a slow
//! consumer, an upload stopped by a response that came first, a writer that
//! heard its stream fail, a reset, a connection that carried several calls,
//! events with an id or a reconnection time, and each outcome at each end.

use std::convert::Infallible;

use skein_json::Token;
use skein_lib::Rng;
use skein_protocol_kit::seen::Seen;
use skein_protocol_world::scenario;
use skein_protocol_world::server_end::framed_len;
use skein_protocol_world::world::{self, Hold, Run, Scenario, Settings};
use skein_world::sweep::{cases, gate};

const FIRST: u64 = 0x00b0_0000;
const ROUNDS: u64 = 300;

fn record(tally: &mut Seen, run: &Run) {
    let fell = run.fell;
    for (flag, what) in [
        (fell.writer_held, "a writer held back by a slow reader"),
        (fell.upload_held, "an upload held back by a slow consumer"),
        (fell.calls > 1, "a connection that carried several calls"),
        (fell.ids > 0, "an event's id or reconnection time checked"),
        (fell.upload_stopped, "an upload stopped by a response first"),
        (fell.writer_failed, "a writer that heard its stream fail"),
        (fell.reset, "a reset"),
    ] {
        if flag {
            tally.note(what.into());
        }
    }
    for outcome in &run.client.outcomes {
        tally.note(format!("client {outcome:?}"));
    }
    if run.client.outcomes.len() < run.client.calls || run.client.calls == 0 {
        tally.note("client closed first".into());
    }
    for outcome in &run.server.outcomes {
        tally.note(format!("server {outcome:?}"));
    }
}

fn drawn(seed: u64) -> (Scenario, Settings) {
    let mut rng = Rng::new(seed);
    // The slow reader and the slow consumer take a twentieth each.
    let scenario = match rng.below(20) {
        0 => scenario::slow_reader(&mut rng),
        1 => scenario::slow_consumer(&mut rng),
        2..=4 => scenario::early(&mut rng),
        5..=11 => scenario::partway(&mut rng),
        12..=15 => scenario::several(&mut rng),
        _ => scenario::stream(&mut rng),
    };
    (scenario, drawn_settings(&mut rng))
}

fn drawn_settings(rng: &mut Rng) -> Settings {
    let mut settings = Settings::drawn(rng);
    if rng.chance(500) {
        // The least each stack allows below it.
        settings.client_intake = skein_http::client::largest_read(&settings.client.client);
        settings.client_output = skein_http::client::largest_room(&settings.client.client);
        settings.server_intake = skein_http::server::largest_read(&settings.server.server);
        settings.server_output = skein_http::server::largest_room(&settings.server.server);
    }
    if rng.chance(100) {
        settings.reset = Some(rng.below(1_000));
    }
    settings
}

/// Qualify a fresh hold before running any machine. Ineligible holds keep
/// their scripted stall and the run's other goals.
fn qualify_hold(scenario: &mut Scenario, settings: &Settings) -> Option<Hold> {
    let hold = scenario.holds?;
    let stall = match hold {
        Hold::Writer => scenario.client.stall,
        Hold::Upload => scenario.server.stall,
    };
    // The referee requires time to fill the path twice and observe the hold
    // before the timed stall releases it. A cut precludes a hold-and-release.
    let fill = settings.in_flight() * 4000 / (u64::from(settings.carry) * (settings.piece as u64 + 1));
    if settings.reset.is_none()
        && stall.is_some_and(|(after, time)| time >= fill + 200 && unread_work(scenario, settings, hold, after))
    {
        return None;
    }
    scenario.holds.take()
}

fn unread_work(scenario: &Scenario, settings: &Settings, hold: Hold, after: usize) -> bool {
    match hold {
        Hold::Writer => {
            let bytes: u64 = scenario.server.answers[0].iter().skip(after).map(framed_len).sum();
            bytes > settings.in_flight()
        }
        Hold::Upload => {
            // After the stall token, raw string bytes are a lower bound on
            // the unread wire bytes: JSON escaping can only add bytes.
            let bytes: u64 = scenario.requests[0]
                .iter()
                .skip(after)
                .filter_map(|token| match token {
                    Token::String(text) => Some(text.len() as u64),
                    _ => None,
                })
                .sum();
            let capacity = u64::from(settings.client_output)
                + u64::from(settings.server_intake)
                + u64::from(settings.server.server.read)
                + u64::from(settings.server.json.chunk);
            bytes > capacity
        }
    }
}

#[test]
fn every_scenario_holds_whatever_the_caps_and_the_wire() {
    let test = skein_world::this_test!("every_scenario_holds_whatever_the_caps_and_the_wire");
    let at_gate = gate();
    let mut tally = Seen::default();
    let (mut writer_ineligible, mut upload_ineligible) = (0, 0);
    cases(test, FIRST..FIRST + ROUNDS, |seed| -> Result<(), Infallible> {
        let (mut scenario, settings) = drawn(seed);
        if !at_gate {
            match qualify_hold(&mut scenario, &settings) {
                Some(Hold::Writer) => writer_ineligible += 1,
                Some(Hold::Upload) => upload_ineligible += 1,
                None => {}
            }
        }
        let run = world::run(&scenario, &settings, seed);
        if at_gate {
            record(&mut tally, &run);
        }
        Ok(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    if !at_gate {
        eprintln!("fuzzy reach: {test} hold/upload-ineligible {upload_ineligible}");
        eprintln!("fuzzy reach: {test} hold/writer-ineligible {writer_ineligible}");
        return;
    }
    for what in [
        "a writer held back by a slow reader",
        "an upload held back by a slow consumer",
        "a connection that carried several calls",
        "an event's id or reconnection time checked",
        "an upload stopped by a response first",
        "a writer that heard its stream fail",
        "a reset",
        "client Done(Keep)",
        "client Done(Close)",
        "client closed first",
        "server Done(Keep)",
        "server Done(Close)",
        "server Ended",
        "server Failed(Truncated)",
        "server Failed(Stream(Reset))",
    ] {
        assert!(tally.0.contains_key(what), "{what} fell: {:#?}", tally.0);
    }
}

#[test]
fn full_width_request_draw_fits_the_json_budget() {
    // More messages than the sweep draws, to exercise the shared byte budget.
    let request = scenario::request(&mut Rng::new(u64::MAX), 120, 400);
    let limits = world::json_limits();
    let _body = skein_json_kit::write(&request, &limits).expect("the bounded conversation fits the document");
}

#[test]
fn full_width_seeds_draw_runnable_protocol_cases() {
    let seed = u64::MAX;
    let (mut scenario, settings) = drawn(seed);
    assert_eq!(qualify_hold(&mut scenario, &settings), None);
    let _ = world::run(&scenario, &settings, seed);

    for (hold, draw) in
        [(Hold::Writer, scenario::slow_reader as fn(&mut Rng) -> Scenario), (Hold::Upload, scenario::slow_consumer)]
    {
        let mut rng = Rng::new(seed);
        let mut scenario = draw(&mut rng);
        let mut settings = drawn_settings(&mut rng);
        // A fast, uncut wire isolates the work left when the user stalls.
        settings.reset = None;
        settings.piece = 256;
        settings.carry = 1000;
        settings.grant = 1000;
        settings.eagerness = 1000;
        assert_eq!(qualify_hold(&mut scenario, &settings), None);
        let run = world::run(&scenario, &settings, seed);
        match hold {
            Hold::Writer => assert!(run.fell.writer_held && run.client.resumed.is_some()),
            Hold::Upload => assert!(run.fell.upload_held && run.server.resumed.is_some()),
        }

        // Still reaches a timed stall but its last event or token cannot
        // fill the path. Only the hold goal is ineligible.
        let after = match hold {
            Hold::Writer => scenario.server.answers[0].len() - 1,
            Hold::Upload => scenario.requests[0].len() - 1,
        };
        let stall = Some((after, 600));
        match hold {
            Hold::Writer => scenario.client.stall = stall,
            Hold::Upload => scenario.server.stall = stall,
        }
        assert_eq!(qualify_hold(&mut scenario, &settings), Some(hold));
        let run = world::run(&scenario, &settings, seed);
        match hold {
            Hold::Writer => assert!(run.client.resumed.is_some()),
            Hold::Upload => assert!(run.server.resumed.is_some()),
        }
    }
}
