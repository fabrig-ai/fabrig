use super::*;
use crate::journey::{Framing, Replay};
use skein_ui_screen_terminal::Key;

const ADDRESS: &str = "127.0.0.1:34073";
const FIXED: &str = "127.0.0.1:10001";

fn written(lines: &[Line]) -> Box<[u8]> {
    let mut bytes = Vec::new();
    for line in lines {
        line.write(&mut bytes);
    }
    bytes.into()
}

#[test]
fn every_line_field_and_both_prints_are_normalised() {
    let mut source = concat!(
        "reply 1 \"@\"\nstreamed record \"@\"\nstored \"@\" \"@\"\nsave \"@\" \"@\"\nerase \"@\"\n",
        "credential \"@\"\nkeep \"@\"\nstream \"@\"\nkey paste \"@\" \"@\"\n",
        "send 1 sign-out \"\" \"@\"\nsend 1 sign-in-service \"\" \"@\"\n",
        "found \"@\"\nlend 1 \"@\"\nbox-log \"@\"\nbox-line \"@\"\nbox-stage \"@\"\n",
        "address \"@\"\nsigned \"@\"\nbrowse \"@\"\nmark \"@\"\nend \"@\"\n",
    )
    .to_owned();
    for kind in [
        "authorize",
        "approval-ask",
        "approval-read",
        "approval-decide",
        "sign-in-approval",
        "request",
        "read",
        "watch",
        "unwatch",
        "stream-gone",
        "sign-in-link",
    ] {
        source.push_str("send 1 ");
        source.push_str(kind);
        source.push_str(" \"@\" \"@\"\n");
    }
    let result = normalise(source.replace('@', ADDRESS).as_bytes()).expect("readable fields");
    let expected = source.replace('@', FIXED);
    assert!(result.recording.as_ref() == expected.as_bytes(), "every field takes the port placeholder");
    assert!(lines(&result.recording).expect("normalised lines") == lines(expected.as_bytes()).expect("expected lines"));
    assert_eq!((result.build, result.port, result.code, result.stood), (0, 1, 0, 0));
    assert_eq!(result.places, u32::try_from(source.matches('@').count()).expect("field count"));
}

#[test]
fn framed_ports_keep_their_length_or_stand_in_every_field() {
    for (port, replacement, changed, stood) in [("34073", "10001", 1, 0), ("8080", "8080", 0, 1)] {
        let input = format!(
            "address \"localhost:{port}\"\nstored \"key\" \"\\x00\\x00\\x00{}localhost:{port}\"\nreply 1 \"localhost:{port}\"\n",
            if port.len() == 5 { "\\x0F" } else { "\\x0E" }
        );
        let result = normalise(input.as_bytes()).expect("framed fields");
        assert_eq!(result.recording.as_ref(), input.replace(port, replacement).as_bytes());
        assert_eq!((result.port, result.places, result.stood), (changed, changed * 3, stood));
        let before = lines(input.as_bytes()).expect("before");
        let after = lines(&result.recording).expect("after");
        for (mut before, mut after) in before.into_iter().zip(after) {
            for ((before, framing), (after, _)) in fields_mut(&mut before).into_iter().zip(fields_mut(&mut after)) {
                if framing == Framing::Framed {
                    assert_eq!(before.len(), after.len());
                }
            }
        }
    }
}

#[test]
fn build_codes_and_pasted_address_elements_agree_everywhere() {
    let address = b"http://127.0.0.1:34073/jig/link?secret=xxxxxxxx";
    let fixed = b"http://127.0.0.1:10001/jig/link?secret=xxxxxxxx";
    let mut source = lines(b"version \"0.0.0 (aa699a65adfb-dirty)\"\naddress \"127.0.0.1:34073\"\nbrowse \"http://127.0.0.1:34073/jig?code=WDJB-MJHT\"\nreply 1 \"\\x00\\x00\\x00\\x08WDJBMJHT\"\nmark \"WDJB-MJHT\"\nend \"0.0.0 (aa699a65adfb-dirty) WDJB-MJHT\"\n").expect("text recording");
    source.push(Line::Key {
        key: Key::Paste(address.as_slice().into()),
        event: Some(format!("Typed {{ value: {address:?} }}").into_bytes().into()),
    });
    let result = normalise(&written(&source)).expect("normalises");
    let mut expected = lines(b"version \"0.0.0 (000000000000)\"\naddress \"127.0.0.1:10001\"\nbrowse \"http://127.0.0.1:10001/jig?code=BBBB-BBBB\"\nreply 1 \"\\x00\\x00\\x00\\x08BBBBBBBB\"\nmark \"BBBB-BBBB\"\nend \"0.0.0 (000000000000) BBBB-BBBB\"\n").expect("expected text");
    let decimal_address = fixed;
    expected.push(Line::Key {
        key: Key::Paste(fixed.as_slice().into()),
        event: Some(format!("Typed {{ value: {decimal_address:?} }}").into_bytes().into()),
    });
    assert_eq!(result.recording, written(&expected));
    assert!(lines(&result.recording).expect("result lines") == expected);
    assert_eq!((result.build, result.port, result.code, result.places, result.stood), (1, 1, 1, 10, 0));
    let again = normalise(&result.recording).expect("again");
    assert!(again.recording == result.recording, "normalisation is idempotent");
    assert_eq!((again.build, again.port, again.code, again.places, again.stood), (0, 0, 0, 0, 0));
    assert!(normalise(b"unknown\n").is_err(), "unreadable lines refused");
}

#[test]
fn decimal_ports_require_whole_byte_elements_and_honor_stood() {
    let decimal = |bytes: &[u8]| bytes.iter().map(u8::to_string).collect::<Vec<_>>().join(", ");
    for (prefix, suffix, replace) in
        [("[", "]", true), ("[1, ", ", 2]", true), ("1", "]", false), ("[", "0]", false), ("", "", false)]
    {
        let text = format!("{prefix}{}{suffix}", decimal(ADDRESS.as_bytes()));
        let result = normalise(&written(&[Line::Signed(text.clone().into_bytes().into())])).expect("decimal field");
        let expected = if replace { format!("{prefix}{}{suffix}", decimal(FIXED.as_bytes())) } else { text };
        assert_eq!(result.recording, written(&[Line::Signed(expected.into_bytes().into())]));
        assert_eq!(result.port, u32::from(replace));
    }
    let text = format!("Typed {{ value: [{}] }}", decimal(b"http://localhost:8080/x"));
    let source = [
        Line::Key {
            key: Key::Paste(b"http://localhost:8080/x".as_slice().into()),
            event: Some(text.into_bytes().into()),
        },
        Line::Reply { exchange: 1, bytes: b"localhost:8080".as_slice().into() },
    ];
    let input = written(&source);
    let result = normalise(&input).expect("stood decimal port");
    assert_eq!(result.recording, input);
    assert_eq!((result.port, result.places, result.stood), (0, 0, 1));
}

#[test]
fn an_approved_sign_in_normalises_and_replays_to_the_end() {
    let file = crate::journey::redact::tests::approval_recording(ADDRESS.as_bytes());
    let result = normalise(&file).expect("approval recording");
    assert_eq!((result.port, result.code, result.stood), (1, 1, 0));
    let again = normalise(&result.recording).expect("normalised recording");
    assert_eq!(result.recording, again.recording, "normalisation is idempotent");
    assert_eq!((again.build, again.port, again.code, again.places, again.stood), (0, 0, 0, 0, 0));
    let recorded = lines(&result.recording).expect("normalised lines");
    let mut replay = Replay::new(&result.recording).expect("normalised replay");
    let mut marks = 0;
    loop {
        match replay.next() {
            Ok(Some(_)) => marks += 1,
            Ok(None) => break,
            Err(stale) => {
                let kind =
                    recorded.get(usize::try_from(stale.line - 1).expect("line fits")).expect("stale line").kind();
                panic!("approved sign-in: turn {} kind {kind}", stale.turn);
            }
        }
    }
    assert_eq!(marks, recorded.iter().filter(|line| matches!(line, Line::Mark(_) | Line::End(_))).count());
}
