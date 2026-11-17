use super::*;
use crate::journey::{Line, Replay, lines, redact};
use jig_client_shell_web::adapter::Heard;
use skein_lib::{
    Writer,
    base64::{self, Alphabet},
};

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
    let source = concat!(
        "ways \"@\" \"@\"\nforge \"@\"\npath \"@\"\nsecret \"@\"\nstored \"@\" \"@\"\n",
        "tab-open \"@\"\ntab-point \"@\"\nplace \"@\"\nreturned \"@\"\ncopy 1 \"@\"\n",
        "heard answered 1 200 \"@\"\nwritten \"@\"\nunwritten \"@\"\n",
        "act press \"@\" \"@\"\nact input \"@\" \"@\" \"@\"\nact drawing \"@\" \"@\" \"@\"\n",
        "exchange 1 64 POST \"@\" \"@\" \"@\" \"@\"\nopen 64 \"@\" \"@\" \"@\"\n",
        "save \"@\" \"@\"\nerase \"@\"\nnavigate \"@\"\nreplace \"@\"\nshow \"@\" push\n",
        "mark \"<p>@</p>\"\nend \"<p>@</p>\"\n",
    );
    let result = normalise(source.replace('@', ADDRESS).as_bytes()).expect("readable fields");
    let expected = source.replace('@', FIXED);
    assert_eq!(result.recording.as_ref(), expected.as_bytes());
    assert_eq!(
        lines(&result.recording).expect("normalised lines"),
        lines(expected.as_bytes()).expect("expected lines")
    );
    assert_eq!((result.build, result.port, result.code, result.stood), (0, 1, 0, 0));
    assert_eq!(result.places, u32::try_from(source.matches('@').count()).expect("field count"));
    let again = normalise(&result.recording).expect("normalised recording");
    assert!(again.recording == result.recording, "normalisation is idempotent");
    assert_eq!((again.port, again.places, again.stood), (0, 0, 0));
}

#[test]
fn framed_ports_keep_their_length_or_stand_in_every_field() {
    for (port, replacement, changed, stood) in [("34073", "10001", 1, 0), ("8080", "8080", 0, 1)] {
        let input = format!(
            "path \"localhost:{port}\"\nstored \"key\" \"\\x00\\x00\\x00{}localhost:{port}\"\nexchange 1 64 POST \"/\" \"localhost:{port}\"\n",
            if port.len() == 5 { "\\x0f" } else { "\\x0e" }
        );
        let result = normalise(input.as_bytes()).expect("framed fields");
        let expected = input.replace(port, replacement);
        assert_eq!(result.recording.as_ref(), expected.as_bytes());
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

fn event(record: &[u8]) -> Box<[u8]> {
    let length = base64::encoded_len(Alphabet::Standard, u32::try_from(record.len()).expect("record length"))
        .expect("encoding length");
    let mut writer = Writer::new(usize::try_from(length).expect("encoding length"));
    base64::encode(Alphabet::Standard, record, &mut writer).expect("measured encoding");
    [b"data: ".as_slice(), &writer.finish(), b"\n\n"].concat().into()
}
fn piece(bytes: &[u8]) -> Line {
    Line::Heard(Heard::Piece { bytes: bytes.into() })
}

#[test]
fn joined_streams_number_records_and_plain_runs_at_their_first_piece() {
    let bytes = [b": localhost:42000\n".as_slice(), &event(b"localhost:41000")].concat();
    let expected = [b": localhost:10001\n".as_slice(), &event(b"localhost:10002")].concat();
    // Every possible split includes ones inside the host, port and base64 group.
    for cut in 0..=bytes.len() {
        let source =
            written(&[piece(&bytes[..cut]), Line::Path(b"localhost:43000".as_slice().into()), piece(&bytes[cut..])]);
        let result = normalise(&source).expect("stream decodes");
        let after = lines(&result.recording).expect("stream lines");
        let streams = Streams::read(&after).expect("normalised stream");
        let plain = streams.plain().first().expect("comment");
        let record = streams.events().first().expect("event");
        // A record starting on the later piece comes after the intervening field.
        let starts_later = cut <= b": localhost:42000\n".len();
        let first_later = cut == 0;
        assert_eq!(plain.bytes.as_ref(), if first_later { b": localhost:10002\n" } else { b": localhost:10001\n" });
        assert_eq!(record.record.as_ref(), if starts_later { b"localhost:10003" } else { b"localhost:10002" });
        assert_eq!((result.port, result.places, result.stood), (3, 3, 0));
        let pieces: Vec<_> = after
            .iter()
            .filter_map(|line| if let Line::Heard(Heard::Piece { bytes }) = line { Some(bytes.len()) } else { None })
            .collect();
        assert_eq!(pieces, vec![cut, bytes.len() - cut]);
    }
    let result = normalise(&written(&[piece(&bytes)])).expect("one piece");
    assert_eq!(result.recording, written(&[piece(&expected)]));
}

#[test]
fn only_ports_are_sought_and_redactions_refusals_are_preserved() {
    let input = b"mark \"0.0.0 (aa699a65adfb-dirty) WDJBMJHT WDJB-MJHT\"\n";
    let result = normalise(input).expect("text");
    assert_eq!(result.recording.as_ref(), input);
    assert_eq!((result.build, result.port, result.code, result.places, result.stood), (0, 0, 0, 0, 0));
    for bad in [b"unknown\n".as_slice(), b"heard piece \"data: !bad\\x0A\"\n"] {
        let expected = redact(bad, &[]).err().expect("redact refuses");
        assert_eq!(normalise(bad).err(), Some(expected));
    }
}

#[test]
fn committed_welcome_playground_normalises_and_replays_to_the_end() {
    let file = include_bytes!("../../../../../docs/design/client/web/journeys/welcome-playground/recording");
    let result = normalise(file).expect("committed recording");
    eprintln!(
        "welcome-playground: build={} port={} code={} places={} stood={}",
        result.build, result.port, result.code, result.places, result.stood
    );
    assert_eq!((result.build, result.port, result.code, result.places, result.stood), (0, 2, 0, 8, 0));
    let again = normalise(&result.recording).expect("idempotent committed recording");
    assert!(again.recording == result.recording, "normalisation is idempotent");
    assert_eq!((again.build, again.port, again.code, again.places, again.stood), (0, 0, 0, 0, 0));
    let recorded = lines(&result.recording).expect("normalised lines");
    assert!(
        recorded
            .iter()
            .any(|line| matches!(line, Line::Ways { origin, .. } if origin.as_ref() == b"http://127.0.0.1:10001")),
        "the committed origin takes its first port placeholder"
    );
    let mut replay = Replay::new(&result.recording).expect("normalised replay");
    let mut ended = false;
    loop {
        match replay.next() {
            Ok(Some(mark)) => ended = mark.end,
            Ok(None) => break,
            Err(stale) => {
                let kind =
                    recorded.get(usize::try_from(stale.line - 1).expect("line fits")).expect("stale line").kind();
                panic!("welcome-playground: turn {} kind {kind}", stale.turn);
            }
        }
    }
    assert!(ended, "replayed the final end");
}
