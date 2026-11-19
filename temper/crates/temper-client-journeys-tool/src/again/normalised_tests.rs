//! Native saves and walker decisions across placeholders (client/journey-placeholders.md, 4.2).
use super::*;
use skein_lib::{List, Token};
use temper_client_journeys_kit::web::{Capture, managed_capture};

fn managed() -> Capture {
    managed_capture(Capture::unfixed())
}

#[test]
fn a_still_save_finds_a_control_across_placeholders() {
    let mut capture = managed();
    let save = capture.save();
    capture.act(Role::Button, b"Connect as its admin", Act::Press);
    let committed = normalised(&capture.save()).expect("normalised source");
    let step = walked("managed", &committed)
        .expect("source walk")
        .into_iter()
        .find(|step| matches!(step, Step::Act { act: Act::Press, .. }))
        .expect("press step");
    let Step::Act { act, event } = &step else { panic!("press step") };
    let placeholder = format!("{:?}", b"http://127.0.0.1:10001/web");
    assert!(
        event.windows(placeholder.len()).any(|part| part == placeholder.as_bytes()),
        "the recorded event holds the placeholder's decimal elements"
    );
    assert_eq!(journey::control(&save, act, event), Ok(None), "raw save cannot give the placeholder event");
    let mut walker = Walker::new("managed", vec![step], None);
    walker.phase = Phase::Still { save: save.into_vec() };
    let mut face = BrowserFace::new(Token::new(1), "http://127.0.0.1:41234");
    walker.progress(&mut face, Time::ZERO).expect("normalised still save");
    assert!(
        matches!(&walker.phase, Phase::Find { control } if *control == Control { role: Role::Button, name: b"Connect as its admin".as_slice().into(), order: 0 })
    );
}

fn walk_save(port: &str, act: &str, marked: bool) -> Vec<u8> {
    format!("journey 2\nseed 11\nclocks 0 0\nprimary control\nways \"\" \"http://127.0.0.1:{port}\"\npath \"/\"\nturn 0 0\nact {act} \"Copy http://127.0.0.1:{port}/web\" \"Copy\"\ndeadline none\nturn 1000000000 0\n{}end\n", if marked { "mark\ndeadline none\n" } else { "" }).into_bytes()
}
#[test]
fn the_walk_check_normalises_both_recordings_and_keeps_real_differences() {
    let walker = Walker::new("ports", vec![Step::End], None);
    let live = walk_save("41234", "press", true);
    let committed = normalised(&live).expect("committed placeholders");
    assert!(walker.check(&committed, &live).is_ok());
    // The old source is normalised too, even when it predates placeholders.
    assert!(walker.check(&walk_save("42345", "press", true), &live).is_ok());
    for (new, step) in [(walk_save("42345", "toggle", true), 1), (walk_save("42345", "press", false), 2)] {
        assert!(
            matches!(walker.check(&committed, &new), Err(Refusal::WalkDiffers { name, step: found }) if name == "ports" && found == step)
        );
    }
}

#[test]
fn a_refused_normalisation_is_an_unreadable_save_at_its_line() {
    let mut walker =
        Walker::new("ports", vec![Step::Mark, Step::Act { act: Act::Press, event: b"Event".as_slice().into() }], None);
    walker.at = 1;
    for (save, line) in [(b"unknown\n".as_slice(), 1), (b"journey 2\nheard piece \"data: !bad\\x0a\"\n", 2)] {
        walker.phase = Phase::Still { save: save.into() };
        let mut face = BrowserFace::new(Token::new(1), "http://127.0.0.1:41234");
        let refusal = walker.progress(&mut face, Time::ZERO).expect_err("refused save");
        assert!(
            matches!(refusal, Refusal::Save { name, step: 2, refusal: journey::Refusal::Unreadable(journey::Unreadable { line: found, .. }) } if name == "ports" && found == line)
        );
    }
}

#[test]
fn a_typed_placeholder_is_given_as_committed() {
    use skein_browser::boundary::{Event, Request, Seen, States};
    let value = b"http://127.0.0.1:10001/web";
    let mut walker = Walker::new(
        "ports",
        vec![
            Step::Act { act: Act::Input { value: value.as_slice().into() }, event: b"Event".as_slice().into() },
            Step::End,
        ],
        None,
    );
    walker.phase =
        Phase::Find { control: Control { role: Role::TextField, name: b"Address".as_slice().into(), order: 0 } };
    let mut face = BrowserFace::new(Token::new(1), "http://127.0.0.1:41234");
    walker.progress(&mut face, Time::ZERO).expect("find field");
    let Request::Find { op, .. } = face.requests().next().expect("field query") else { panic!("find request") };
    let mut seen = List::with_capacity(1);
    seen.push(Seen {
        node: 42,
        role: b"textbox".as_slice().into(),
        name: b"Address".as_slice().into(),
        value: b"http://127.0.0.1:41234/web".as_slice().into(),
        states: States {
            focused: false,
            disabled: false,
            checked: false,
            expanded: false,
            selected: false,
            pressed: false,
        },
        rect: None,
    })
    .expect("one field");
    face.event(&Event::Found { op, seen, more: 0 });
    let next = walker.progress(&mut face, Time::ZERO).expect("type committed value");
    assert!(matches!(next, Next::Do(Doing::Act { act: Act::Input { value: found }, .. }) if found.as_ref() == value));
}

#[test]
fn an_end_save_is_kept_raw_for_add() {
    let save = walk_save("41234", "press", true);
    let mut walker = Walker::new("ports", vec![Step::End], None);
    walker.phase = Phase::Still { save: save.clone() };
    let mut face = BrowserFace::new(Token::new(1), "http://127.0.0.1:41234");
    assert_eq!(walker.progress(&mut face, Time::ZERO).expect("end save"), Next::Done);
    assert_eq!(walker.recording, Some(save));
}

#[test]
fn a_fixed_walk_keeps_its_raw_end_save() {
    let mut capture = managed_capture(Capture::fixed());
    let before = capture.save();
    capture.act(Role::Button, b"Connect as its admin", Act::Press);
    let after = capture.save();
    let committed = normalised(&after).expect("normalised walk");
    let press = walked("fixed", &committed)
        .expect("walk")
        .into_iter()
        .find(|step| matches!(step, Step::Act { act: Act::Press, .. }))
        .expect("press");
    let mut walker = Walker::new("fixed", vec![press, Step::End], None);
    let mut face = BrowserFace::new(Token::new(1), "http://127.0.0.1:41234");
    walker.phase = Phase::Still { save: before.into_vec() };
    walker.progress(&mut face, Time::ZERO).expect("find press in still save");
    assert!(matches!(&walker.phase, Phase::Find { control } if control.name.as_ref() == b"Connect as its admin"));
    walker.advance();
    walker.phase = Phase::Still { save: after.to_vec() };
    assert_eq!(walker.progress(&mut face, Time::ZERO).expect("end walk"), Next::Done);
    let recording = walker.recording.take().expect("end recording");
    walker.check(&committed, &recording).expect("same walk");
    assert_eq!(recording, after.as_ref(), "the end recording is the raw capture after the press");
}
