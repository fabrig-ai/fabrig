//! A recording through the shipped host, replayed and proved to its end (client/tui/journeys.md, 4).
use temper_client_shell_tui::journey::{Facts, Recorder, Replay, lines, prove};
mod fixture;

#[test]
fn setup_and_log_recordings_replay_and_prove_to_the_end() {
    for (set_up, log) in fixture::SETUPS {
        let recording = recorded(&fixture::setup(set_up, log));
        let mut replay = Replay::new(&recording).expect("read recording");
        while replay.next().expect("replay facts").is_some() {}
        assert!(prove(&recording).is_ok(), "new facts preserve recorded prints");
        if !set_up && log.is_empty() {
            // Journey, seed, clocks, screen and address precede box and front.
            const BOX_AT: usize = 5;
            const FRONT_AT: usize = 6;
            let mut legacy = lines(&recording).expect("start records");
            legacy.swap(BOX_AT, FRONT_AT);
            let out = fixture::written(&legacy);
            assert!(prove(&out).is_ok(), "recordings with front then box replay unchanged");
        }
    }
}

fn recorded(given: &Facts) -> Box<[u8]> {
    let (mut host, start) = given.start();
    let mut recorder = Recorder::new(given);
    // The original walk's start turn and following mark turn.
    let now = skein_lib::Time::from_nanos(1);
    let wall = skein_lib::Wall::from_nanos(1);
    host.clocks(now, wall);
    recorder.turn(now, wall);
    recorder.report(&mut host, start);
    recorder.deadline(host.deadline());
    host.frame();
    let now = skein_lib::Time::from_nanos(2);
    let wall = skein_lib::Wall::from_nanos(2);
    host.clocks(now, wall);
    recorder.turn(now, wall);
    recorder.mark(&mut host);
    recorder.deadline(host.deadline());
    recorder.save(&mut host)
}
