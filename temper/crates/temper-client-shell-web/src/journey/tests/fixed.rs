use super::*;
use crate::{host::Report, internal::Internal};
use skein_lib::{Duration, FixedStart};

#[test]
fn a_fixed_hosts_facts_and_turns_replay_as_recorded() {
    let start = FixedStart::read(b"1767690000,7").expect("fixed start");
    let native = Time::from_nanos(9_000_000_000);
    let mut platform = skein_ui_web::platform::Facts {
        seed: 99,
        now: native,
        wall: Wall::EPOCH,
        offset: -300,
        languages: b"en".as_slice().into(),
        primary: Primary::Control,
    };
    let settings = Internal { fixed_start: Some(start), ..Internal::default() };
    let mut clock = settings.start(&mut platform).expect("fixed clock");
    let facts = Facts {
        fixed: true,
        seed: platform.seed,
        now: platform.now,
        wall: platform.wall,
        zone_minutes: platform.offset,
        // A lost link sign-in gives the host a retry deadline.
        path: b"/welcome".as_slice().into(),
        secret: Some(b"fake-link".as_slice().into()),
        ..facts()
    };
    let (mut host, mut page) = facts.start();
    let mut recorder = Recorder::new(&facts);
    let (now, wall) = clock.read(native);
    assert_eq!((now, wall), (FixedStart::NOW, start.wall));
    host.clocks(now, wall);
    recorder.turn(now, wall);
    let duties = recorder.report(&mut host, &page, Report::Start);
    let exchange = duties
        .asked
        .iter()
        .find_map(|ask| match ask {
            jig_client_shell_web::adapter::Asked::Exchange { id, .. } => Some(*id),
            jig_client_shell_web::adapter::Asked::OpenStream { .. }
            | jig_client_shell_web::adapter::Asked::CloseStream
            | jig_client_shell_web::adapter::Asked::Save { .. }
            | jig_client_shell_web::adapter::Asked::Erase { .. } => None,
        })
        .expect("link sign-in exchange");
    recorder.report(&mut host, &page, Report::Heard(Heard::Failed { exchange }));
    host.draw(&mut page, &mut Vec::new());
    let deadline = host.deadline().expect("lost sign-in retry");
    recorder.deadline(Some(deadline));
    let instant = clock.until(Some(deadline), native).expect("native deadline");
    let before = Time::from_nanos(instant.as_nanos() - 1);
    let mut expected_turns = vec![Line::Turn { now, wall }];
    for native in [before, instant.saturating_add(Duration::from_secs(1))] {
        let (now, wall) = clock.read(native);
        let expected = if native == before { FixedStart::NOW } else { deadline };
        assert_eq!(now, expected);
        assert_eq!(wall.as_nanos(), start.wall.as_nanos() + now.saturating_since(FixedStart::NOW).as_nanos());
        host.clocks(now, wall);
        recorder.turn(now, wall);
        recorder.report(&mut host, &page, Report::Due);
        host.draw(&mut page, &mut Vec::new());
        recorder.deadline(host.deadline());
        // Ask at every iteration, including the one before the unchanged deadline.
        let _instant = clock.until(host.deadline(), native);
        expected_turns.push(Line::Turn { now, wall });
    }
    let saved = recorder.save(&page);
    assert!(
        saved.starts_with(b"journey 2\nseed 7\nclocks 3600000000000 1767690000000000000\nfixed\nprimary control\n")
    );
    let records = lines(&saved).expect("fixed records");
    assert!(!records.iter().any(|line| matches!(line, Line::Zone(_))));
    assert_eq!(
        records.into_iter().filter(|line| matches!(line, Line::Turn { .. })).collect::<Vec<_>>(),
        expected_turns
    );
    prove(&saved).expect("fixed recording replays unchanged");
}

#[test]
fn fixed_is_an_optional_fact_only_after_clocks() {
    let facts = Facts { fixed: true, ..facts() };
    let (_, page) = facts.start();
    let records = lines(&Recorder::new(&facts).save(&page)).expect("fixed records");
    let (read, _) = replay::facts(&records).expect("fixed facts");
    assert!(read.fixed);
    assert_eq!(Line::read(b"fixed"), Ok(Line::Fixed));
    let mut written = Vec::new();
    Line::Fixed.write(&mut written);
    assert_eq!(written, b"fixed\n");
    let mut plain = records.clone();
    plain.remove(3);
    assert!(!replay::facts(&plain).expect("old facts still read").0.fixed);
    for at in [2, 4] {
        let mut misplaced = plain.clone();
        misplaced.insert(at, Line::Fixed);
        let Err(error) = replay::facts(&misplaced) else { panic!("misplaced fact at {at} accepted") };
        assert_eq!(error.line, u32::try_from(at + 1).expect("line number"));
        assert_eq!(error.why, "expected ordered start facts for journey 2");
    }
}

#[test]
fn a_fixed_key_mark_is_walked_without_a_moving_judgement() {
    let records = [
        turn(1000),
        Line::Mark(None),
        Line::Deadline(None),
        turn(1100),
        Line::Heard(Heard::StreamOpened),
        Line::Deadline(None),
    ];
    let plain = recording(&records);
    assert_eq!(walk(&plain), Err(Unwalkable::Moving { step: 1 }));
    let mut fixed = lines(&plain).expect("plain records");
    fixed.insert(3, Line::Fixed);
    assert_eq!(walk(&written(&fixed)), Ok(vec![Step::Mark, Step::End]));
}

#[test]
fn an_unfixed_start_keeps_native_facts_and_has_no_fixed_line() {
    let mut platform = skein_ui_web::platform::Facts {
        seed: 11,
        now: Time::from_nanos(123),
        wall: Wall::from_nanos(456),
        offset: 120,
        languages: Box::default(),
        primary: Primary::Meta,
    };
    assert!(Internal::default().start(&mut platform).is_none());
    assert_eq!(
        (platform.seed, platform.now, platform.wall, platform.offset),
        (11, Time::from_nanos(123), Wall::from_nanos(456), 120)
    );
    let facts = facts();
    let (_, page) = facts.start();
    let records = lines(&Recorder::new(&facts).save(&page)).expect("native records");
    assert!(!records.contains(&Line::Fixed));
    assert!(!replay::facts(&records).expect("native facts").0.fixed);
}
