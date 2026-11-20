//! A recording through the shipped host, replayed and proved to its end (client/tui/journeys.md, 4).
use temper_client_shell_tui::journey::{Facts, Recorder, Replay, lines, prove};
mod fixture;
use fixture::{linked_walk_duties, replay_stale, same_frames, written};
use temper_client_shell_tui::{host::Report, journey::Line};

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

#[test]
fn local_reports_replay_and_compare_the_new_duties() {
    let mut walk = linked_walk_duties(false).0;
    let original = lines(&walk.save()).expect("local records");
    for kind in ["local", "noted", "found", "read", "note"] {
        assert!(original.iter().any(|line| line.kind() == kind), "missing {kind}");
    }
    for (kind, replacement) in
        [("read", Line::Browse(b"changed".as_slice().into())), ("note", Line::Note(temper_client_tui::Note::SignedOut))]
    {
        let mut records = original.clone();
        let at = records.iter().position(|line| line.kind() == kind).expect("local duty");
        records[at] = replacement;
        assert_eq!(records.iter().zip(&original).filter(|(a, b)| a != b).count(), 1);
        let stale = replay_stale(&written(&records));
        assert_eq!(stale.line, u32::try_from(at + 1).expect("line"), "changed {kind}");
        let mut expected = Vec::new();
        original[at].write(&mut expected);
        expected.pop();
        assert_eq!(stale.asked.as_ref(), expected);
    }
    same_frames(walk);
}

#[test]
fn lending_reports_and_duties_replay_through_the_host() {
    use fixture::{delivery_report, terminal_document, watch};
    use jig_client::contract::LoginExit;
    use jig_client_protocol_wire as w;
    let (mut walk, duties) = linked_walk_duties(true);
    let duties = walk.admit(duties);
    let subject = w::Subject::Terminal(
        w::SubjectTerminal::new(&w::CEILINGS, w::SubjectTerminalParts { terminal: skein_lib::Rng::new(7).next_u64() })
            .unwrap(),
    );
    let terminal = watch(&duties, &subject);
    let duties = walk.reports(vec![delivery_report(terminal, &terminal_document(Some((1, false))))]);
    assert!(duties.iter().any(|duties| duties.lend.as_ref().is_some_and(|lend| lend.number == 1)));
    walk.mark();
    let duties = walk.reports(vec![delivery_report(terminal, &terminal_document(Some((1, true))))]);
    assert!(duties.iter().any(|duties| duties.unlend));
    let duties = walk.reports(vec![Report::Lent { number: 1, exit: LoginExit::Killed { signal: 15 } }]);
    walk.admit(duties.into_iter().next().unwrap());
    walk.mark();
    let records = lines(&walk.save()).unwrap();
    assert!(records.contains(&Line::Lend { number: 1, name: b"work".as_slice().into() }));
    assert!(records.contains(&Line::Unlend));
    assert!(records.contains(&Line::Lent { number: 1, exit: LoginExit::Killed { signal: 15 } }));
    same_frames(walk);
}

#[test]
fn found_links_redact_prove_and_draw_the_replacement() {
    use skein_lib::{Secret, Time, Wall, bytes::find_from};
    use temper_client_shell_tui::journey::{Sent, redact, scan};
    use temper_client_tui::BoxLine;
    let url = fixture::printed_link();
    let mut world = fixture::World::with_front(b"127.0.0.1:8367", true);
    world.report(Report::Front(BoxLine::Link { url: Secret::new(url.clone()) }));
    let now = Time::from_nanos(world.now);
    let wall = Wall::from_nanos(world.now);
    world.host.clocks(now, wall);
    world.recorder.turn(now, wall);
    world.recorder.mark(&mut world.host);
    world.recorder.deadline(world.host.deadline());
    let recording = world.save();
    let records = lines(&recording).expect("welcome records");
    assert!(
        !records.iter().any(|line| matches!(line, Line::Send { message: Sent::SignInLink(_), .. })),
        "the printed link was never sent"
    );
    assert!(
        records.iter().any(|line| {
            if let Line::Mark(Some(print)) = line {
                return find_from(print, &url, 0).is_some();
            }
            false
        }),
        "the mark drew the printed link whole"
    );
    let redacted = redact(&recording, &[]).expect("redact welcome");
    let proved = prove(&redacted.recording).expect("redacted welcome proof");
    assert!(scan(&proved, &redacted.originals).expect("welcome scan").is_none(), "redacted welcome scan");
    let mut replay = Replay::new(&proved).expect("welcome replay");
    let frame = replay.next().expect("welcome duties").expect("welcome mark").frame;
    let hidden_url = fixture::url_base64(&[b'x'; 32]);
    assert!(find_from(&frame.text(), &url, 0).is_none(), "welcome no longer draws the printed link's original");
    assert!(find_from(&frame.text(), &hidden_url, 0).is_some(), "welcome draws the redacted link");
}
