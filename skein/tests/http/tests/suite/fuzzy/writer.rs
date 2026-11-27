//! The event stream writer's machine world, swept (`skein/docs/design/net/http/testing.md`, 6).
//! Each case reads back what was written with the reference reader and the
//! reader's world. The gate checks answer, refusal and neighbour reach
//! across its seeds (`skein/docs/design/testing/sweeps.md`, 4).

use std::convert::Infallible;

use skein_http_world::writer_world::{self, Run, Settings};
use skein_lib::Rng;
use skein_protocol_kit::seen::Seen;
use skein_world::sweep::{cases, gate};
use skein_world::this_test;

const ROUNDS: u64 = 5_000;

fn record(tally: &mut Seen, run: &Run) {
    for (_, answer) in &run.answered {
        match answer {
            Some(answer) => tally.note(format!("{answer:?}")),
            None => tally.note("closed before the answer".into()),
        }
    }
    if run.finished {
        tally.note("finished".into());
    }
    tally.note(format!("closed while waiting for {:?}", run.closed_while));
    let fell = run.fell;
    for (flag, what) in [
        (fell.pieces, "an event in pieces"),
        (fell.ended, "the stream ended"),
        (fell.room_after_end, "room after the end"),
        (fell.failed_idle, "a failure while idle"),
    ] {
        if flag {
            tally.note(what.into());
        }
    }
    if let Some(waiting) = fell.failed_while {
        tally.note(format!("failed while waiting for {waiting:?}"));
    }
}

#[test]
fn generated_events_are_written_and_read_back_as_written_whatever_the_neighbours() {
    let mut tally = Seen::default();
    let at_gate = gate();
    let base = 0x00a0_0000;
    let test = this_test!("generated_events_are_written_and_read_back_as_written_whatever_the_neighbours");
    cases::<_, Infallible, _>(test, base..base + ROUNDS, |seed| {
        let mut rng = Rng::new(seed);
        let items = writer_world::items(&mut rng);
        let limits = writer_world::limits(&mut rng);
        let settings = Settings::chaotic(&mut rng, limits, 400);
        let run = writer_world::check(&items, &settings, seed);
        if at_gate {
            record(&mut tally, &run);
        }
        Ok(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
    if !at_gate {
        return;
    }
    for what in [
        "Sent",
        "Refused(Name)",
        "Refused(Id)",
        "Refused(Comment)",
        "Refused(TooLong)",
        "Failed(Reset)",
        "Failed(Invalid)",
        "Failed(Other)",
        "closed before the answer",
        "finished",
        "closed while waiting for Above",
        "closed while waiting for Room",
        "closed while waiting for Close",
        "an event in pieces",
        "the stream ended",
        "room after the end",
        "a failure while idle",
        "failed while waiting for Above",
        "failed while waiting for Room",
        "failed while waiting for Close",
        "failed while waiting for Nothing",
    ] {
        assert!(tally.0.contains_key(what), "{what} fell: {:#?}", tally.0);
    }
}
