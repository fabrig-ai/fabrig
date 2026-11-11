use super::*;
use jig_client_shell_web::adapter::Heard;

const RECORD: &[u8] = b"one record";
const CHANGED: &[u8] = b"new record";
const EVENT: &[u8] = b"data: b25lIHJlY29yZA==\n\n";

fn piece(bytes: &[u8]) -> Line {
    Line::Heard(Heard::Piece { bytes: bytes.into() })
}
fn split(bytes: &[u8], at: usize) -> Vec<Line> {
    vec![piece(&bytes[..at]), Line::Due, piece(&bytes[at..])]
}
fn written(lines: &[Line]) -> Vec<u8> {
    let mut out = Vec::new();
    for line in lines {
        line.write(&mut out);
    }
    out
}
fn check_record(streams: &Streams, record: &[u8], line: u32) {
    assert_eq!(streams.events().len(), 1, "one record");
    let event = streams.events().first().expect("one event");
    assert_eq!(event.line, line);
    assert!(event.record.as_ref() == record, "decoded record");
}

#[test]
fn an_event_split_at_each_byte_reads_the_same() {
    for cut in 1..EVENT.len() {
        let lines = split(EVENT, cut);
        let streams = Streams::read(&lines).expect("split event");
        check_record(&streams, RECORD, 1);
        assert!(streams.plain().is_empty(), "bare framing is not plain");
    }
}

#[test]
fn a_changed_record_keeps_its_pieces_lengths() {
    for cut in 1..EVENT.len() {
        let mut lines = split(EVENT, cut);
        let mut streams = Streams::read(&lines).expect("split event");
        streams.events_mut().first_mut().expect("one event").record.copy_from_slice(CHANGED);
        streams.write(&mut lines);
        for (line, length) in [(lines.first().expect("first"), cut), (lines.last().expect("last"), EVENT.len() - cut)] {
            let Line::Heard(Heard::Piece { bytes }) = line else { panic!("piece") };
            assert_eq!(bytes.len(), length);
        }
        check_record(&Streams::read(&lines).expect("changed stream"), CHANGED, 1);
    }
}

#[test]
fn three_events_and_a_comment_in_one_piece() {
    let mut lines = vec![piece(&[EVENT, b": comment\n", EVENT, EVENT].concat())];
    let mut streams = Streams::read(&lines).expect("three events");
    assert_eq!(streams.events().len(), 3);
    for event in streams.events() {
        assert!(event.record.as_ref() == RECORD, "decoded record");
        assert_eq!(event.line, 1);
    }
    assert_eq!(streams.plain().len(), 1);
    let plain = streams.plain_mut().first_mut().expect("comment");
    assert_eq!(plain.line, 1);
    assert!(plain.bytes.as_ref() == b": comment\n", "comment retained");
    plain.bytes.copy_from_slice(b": changed\n");
    streams.write(&mut lines);
    let changed = Streams::read(&lines).expect("changed comment");
    assert!(changed.plain().first().expect("comment").bytes.as_ref() == b": changed\n", "comment written");
}

#[test]
fn an_open_starts_a_new_stream() {
    for boundary in
        [Line::Open { piece: 64, path: b"/".as_slice().into(), headers: Vec::new() }, Line::Heard(Heard::StreamOpened)]
    {
        let mut lines = vec![piece(b"data: b25l"), boundary, piece(EVENT)];
        let before = written(&lines);
        let streams = Streams::read(&lines).expect("separate streams");
        assert_eq!(streams.events().len(), 2);
        assert!(streams.events().first().expect("cut event").record.as_ref() == b"one", "first stream");
        assert_eq!(streams.events().last().expect("new event").line, 3);
        streams.write(&mut lines);
        assert!(written(&lines) == before, "boundary preserves bytes");
    }
}

#[test]
fn a_bad_data_line_is_undecoded_by_its_first_piece() {
    for payload in [b"%%%%".as_slice(), b"YR==", b"YQ=", b"YQ", b"YQ==AAAA", b"__=="] {
        let bytes = [b": comment\ndata: ", payload, b"\n"].concat();
        for cut in 1..bytes.len() {
            let lines = split(&bytes, cut);
            let expected = if cut <= b": comment\n".len() { 3 } else { 1 };
            let error = Streams::read(&lines).err().expect("invalid base64 refused");
            assert_eq!(error.line, expected, "first piece of bad data line");
        }
    }
}

#[test]
fn a_last_line_cut_short_decodes_its_whole_groups() {
    for (bytes, tail) in [
        (b"data: b25lI".as_slice(), b"I".as_slice()),
        (b"data: b25lIH".as_slice(), b"IH".as_slice()),
        (b"data: b25lIHJ".as_slice(), b"IHJ".as_slice()),
    ] {
        let mut lines = split(bytes, bytes.len() - tail.len());
        let mut streams = Streams::read(&lines).expect("cut last line");
        check_record(&streams, b"one", 1);
        let plain = streams.plain_mut().first_mut().expect("incomplete group");
        assert_eq!(plain.line, 3);
        assert!(plain.bytes.as_ref() == tail, "only incomplete group is plain");
        plain.bytes.fill(b'x');
        streams.write(&mut lines);
        let Line::Heard(Heard::Piece { bytes }) = lines.last().expect("last piece") else { panic!("piece") };
        assert!(bytes.iter().all(|byte| *byte == b'x'), "plain tail written");
    }
}

#[test]
fn the_committed_recordings_read_and_write_unchanged() {
    let journeys = skein_scratch::paths::tree().join("docs/design/client/web/journeys");
    let mut count = 0;
    for entry in std::fs::read_dir(journeys).expect("journeys") {
        let path = entry.expect("journey").path().join("recording");
        if !path.is_file() {
            continue;
        }
        count += 1;
        let original = std::fs::read(path).expect("recording");
        let mut lines = crate::journey::lines(&original).expect("recording lines");
        Streams::read(&lines).expect("recording streams").write(&mut lines);
        assert!(written(&lines) == original, "recording is byte-identical");
    }
    assert!(count > 0, "committed recordings were checked");
}

#[test]
fn refused_bodies_and_partial_prefixes_are_plain() {
    for bytes in [b"refused\nbody".as_slice(), b"dat", b": partial comment"] {
        let mut lines = vec![piece(bytes)];
        let streams = Streams::read(&lines).expect("plain stream");
        assert!(streams.events().is_empty());
        assert!(
            streams.plain().iter().flat_map(|plain| plain.bytes.iter()).copied().eq(bytes.iter().copied()),
            "all plain bytes"
        );
        streams.write(&mut lines);
        let Line::Heard(Heard::Piece { bytes: restored }) = lines.first().expect("piece") else { panic!("piece") };
        assert!(restored.as_ref() == bytes, "plain stream roundtrip");
    }
}

#[test]
#[should_panic(expected = "11 10")]
fn a_record_length_change_panics_with_lengths_alone() {
    let mut lines = vec![piece(EVENT)];
    let mut streams = Streams::read(&lines).expect("event");
    streams.events_mut().first_mut().expect("event").record = b"long record".as_slice().into();
    streams.write(&mut lines);
}

#[test]
#[should_panic(expected = "11 10")]
fn a_plain_length_change_panics_with_lengths_alone() {
    let mut lines = vec![piece(b": comment\n")];
    let mut streams = Streams::read(&lines).expect("comment");
    streams.plain_mut().first_mut().expect("comment").bytes = b": comments\n".as_slice().into();
    streams.write(&mut lines);
}
