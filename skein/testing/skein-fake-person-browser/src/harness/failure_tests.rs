//! Failed Keep replies name their own step (skein/docs/design/ui-web.md, 14.2).
use super::{Play, Script, Step, expected_trouble, step_failure, trouble_report, trouble_step};
use crate::tests::face;
use skein_browser::boundary::{Event, Key, Read, Refusal, Request, Trouble};
use skein_lib::{Time, Token};

#[test]
fn a_failed_keep_reports_its_own_step() {
    for (reply, why) in [
        (Ok(Read::NoElement), "no element"),
        (Ok(Read::NoAttribute), "no attribute"),
        (Err(Refusal::Protocol), "browser refused read: Protocol (DOM.getAttributes: malformed reply)"),
    ] {
        let mut script = Script::new(Box::new([
            Step::StopServer,
            Step::Keep { selector: b"#hidden".as_slice().into(), attribute: None, name: b"read".as_slice().into() },
        ]));
        let mut face = face();
        script.next(&mut face, Time::ZERO);
        script.next(&mut face, Time::ZERO);
        let Request::Attribute { op, .. } = face.requests().next().expect("Keep's read") else {
            panic!("Keep asks for an attribute")
        };
        face.event(&match reply {
            Ok(read) => Event::Attribute { op, read },
            Err(why) => Event::Refused { op, why, detail: b"DOM.getAttributes: malformed reply".as_slice().into() },
        });
        assert_eq!(
            step_failure(&mut script, &mut face, Time::ZERO),
            Some((1, format!("keep selector #hidden attribute value: {why}"))),
        );
    }
}

#[test]
fn unexpected_trouble_reports_its_url_when_present() {
    assert_eq!(
        trouble_report(Trouble::Log, b"bad", b"https://example.test/401"),
        "unexpected Log: bad (https://example.test/401)"
    );
    assert_eq!(trouble_report(Trouble::Log, b"bad", b""), "unexpected Log: bad");
}

#[test]
fn expected_trouble_reads_only_the_text() {
    let expected = [Box::<[u8]>::from(b"401".as_slice()), Box::default()];
    let text = b"Failed to load resource";
    let url = b"https://example.test/401";
    assert!(trouble_report(Trouble::Log, text, url).contains("401"));
    assert!(!expected_trouble(&expected, text));
    assert!(expected_trouble(&expected, b"status 401 (Unauthorized)"));
    assert!(!expected_trouble(&expected, b""));
}

#[test]
fn unexpected_trouble_names_its_persons_current_step() {
    let mut first = Script::new(Box::new([Step::StopServer]));
    let mut second = Script::new(Box::new([Step::StopServer, Step::StartServer, Step::Key(Key::Enter)]));
    let mut face = face();
    second.next(&mut face, Time::ZERO);
    second.next(&mut face, Time::ZERO);
    let people: [&mut dyn Play; 2] = [&mut first, &mut second];
    assert_eq!(trouble_step(Token::new(101), &people), (1, 2));
    assert_eq!(trouble_step(Token::new(100), &people), (0, 0));
}

#[test]
fn a_refused_act_reports_its_detail_when_present() {
    for (why, detail, expected) in [
        (
            Refusal::Protocol,
            b"DOM.getContentQuads: Could not compute content quads.".as_slice(),
            "browser refused act: Protocol (DOM.getContentQuads: Could not compute content quads.)",
        ),
        (Refusal::Gone, b"", "browser refused act: Gone"),
    ] {
        let mut face = face();
        face.key(Key::Enter, false);
        let Request::Key { op, .. } = face.requests().next().expect("key") else { panic!("key request") };
        face.event(&Event::Refused { op, why, detail: detail.into() });
        assert_eq!(face.failure().as_deref(), Some(expected));
        assert_eq!(face.refusal(), Some(why));
    }
}
