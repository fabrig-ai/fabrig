//! Real host and view evidence for redaction (temper/docs/design/client/web/journeys-recordings.md, 1.3 and temper/docs/design/client/web/journey-redaction.md and temper/docs/design/client/web/journeys-recordings.md, 1.7–1.8 and 2).
use super::{walk, written};
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{Reader, Secret, Writer, bytes::find, sha256::digest};
use skein_ui::Act;
use temper_client_shell_web::journey::{
    Kind, Line, Redacted, Refusal, Replay, SECRET_EVENT, Streams, fields_mut, lines, prove_drawn, redact, scan,
};

fn key() -> Vec<u8> {
    ["s", "k", "-", "0123456789abcdefghijklmnop", "\n"].concat().into_bytes()
}
fn redacted(recording: &[u8]) -> Redacted {
    redact(recording, &[]).expect("readable host walk redacts")
}
fn proved(result: &Redacted) -> Box<[u8]> {
    let records = lines(&result.recording).expect("redacted lines");
    assert!(scan::lines(&records, &result.originals).expect("decoded streams").is_none(), "clean redacted lines");
    let mut turns = 0;
    let proof = prove_drawn(&result.recording, &mut |_, page| {
        turns += 1;
        assert!(scan::page(&page.html(), &result.originals).is_none(), "clean drawn page");
    });
    let recording = match proof {
        Ok(recording) => recording,
        Err(Refusal::Stale(stale)) => {
            let kind = records.get(usize::try_from(stale.line - 1).expect("line fits")).expect("stale line").kind();
            panic!("redacted walk stale at line {} ({kind})", stale.line);
        }
        Err(Refusal::Unreadable(_)) => panic!("redacted walk unreadable"),
        Err(Refusal::Unproved { mark }) => panic!("unproved mark {mark}"),
        Err(Refusal::Differs { mark, byte }) => panic!("redacted print differs at mark {mark}, byte {byte}"),
    };
    assert_eq!(turns, records.iter().filter(|line| matches!(line, Line::Deadline(_))).count());
    assert!(turns > 0, "visited drawn pages");
    let records = lines(&recording).expect("html-free lines");
    assert!(records.iter().all(|line| !matches!(line, Line::Mark(Some(_)) | Line::End(Some(_)))));
    assert!(scan::lines(&records, &result.originals).expect("decoded html-free streams").is_none());
    recording
}
fn absent(recording: &[u8], value: &[u8]) {
    let mut records = lines(recording).expect("html-free lines");
    let streams = Streams::read(&records).expect("stream records");
    absent_lines(recording, &mut records, &streams, value);
}
fn absent_lines(recording: &[u8], records: &mut [Line], streams: &Streams, value: &[u8]) {
    assert!(find(recording, value).is_none(), "original absent from html-free recording");
    for line in records {
        for (field, _) in fields_mut(line) {
            assert!(find(field, value).is_none(), "original absent from byte field");
        }
    }
    for event in streams.events() {
        assert!(find(&event.record, value).is_none(), "original absent from stream record");
    }
}
fn keyed_body(records: &[Line]) -> (usize, &[u8]) {
    records
        .iter()
        .enumerate()
        .find_map(|(index, line)| {
            if let Line::Exchange { path, body, .. } = line {
                (path.as_ref() == b"/jig/v1/request").then_some((index, body.as_ref()))
            } else {
                None
            }
        })
        .expect("keyed exchange")
}
fn encode(value: &w::Keyed) -> Box<[u8]> {
    let mut writer = Writer::new(
        usize::try_from(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("keyed size fits"),
    );
    value.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("measured keyed request");
    writer.finish()
}
fn check_key_walk(byte_at_a_time: bool) {
    let key = key();
    let trimmed = &key[..key.len() - 1];
    let recording = walk::keyed(&key, byte_at_a_time);
    let records = lines(&recording).expect("key walk lines");
    let (index, body) = keyed_body(&records);
    let request = w::Keyed::decode(&w::CEILINGS, &mut Reader::new(body)).expect("keyed envelope");
    assert!(request.value.as_ref().is_some_and(|value| value.expose() == key), "carried value retains the newline");
    let ask = w::Ask::decode(&w::CEILINGS, &mut Reader::new(request.request.as_ref())).expect("typed ask");
    let w::Ask::AccountChange(change) = ask else { panic!("account change") };
    let w::AccountChange::AddKey(change) = &change.change else { panic!("add key") };
    assert!(change.digest == digest(&key), "typed ask holds the whole value's raw sha-256");
    assert!(find(body, trimmed).is_some(), "value carried in keyed exchange");
    assert!(find(body, &digest(&key)).is_some(), "digest carried in keyed exchange");
    let Line::Exchange { id, .. } = &records[index] else { panic!("keyed exchange") };
    assert!(
        records
            .iter()
            .any(|line| matches!(line, Line::Heard(jig_client_shell_web::adapter::Heard::Answered { exchange, .. })
        if exchange == id)),
        "key exchange answered"
    );
    let result = redacted(&recording);
    assert_eq!(result.originals.count(Kind::Typed), 1);
    let proved = proved(&result);
    absent(&proved, &key);
    absent(&proved, trimmed);
    absent(&proved, &digest(&key));
    absent(&proved, &digest(trimmed));
    let after = lines(&proved).expect("proved key lines");
    let inputs: Vec<_> = after
        .iter()
        .filter_map(|line| {
            if let Line::Act { act: Act::Input { value }, event, .. } = line { Some((value, event)) } else { None }
        })
        .collect();
    assert_eq!(inputs.len(), if byte_at_a_time { key.len() } else { 1 });
    for (index, (value, event)) in inputs.into_iter().enumerate() {
        let length = if byte_at_a_time { index + 1 } else { key.len() };
        assert_eq!(value.len(), length);
        assert!(
            value.iter().enumerate().all(|(index, byte)| *byte == if key[index] == b'\n' { b'\n' } else { b'x' }),
            "typed bytes become placeholders"
        );
        let marker = format!("Secret {{ len: {length} }}");
        assert!(find(event, marker.as_bytes()).is_some(), "secret event's length stands");
    }
    let (_, body) = keyed_body(&after);
    let request = w::Keyed::decode(&w::CEILINGS, &mut Reader::new(body)).expect("redacted keyed envelope");
    assert!(
        request.value.as_ref().is_some_and(|value| value.expose() == [vec![b'x'; trimmed.len()], vec![b'\n']].concat()),
        "placeholder carried once"
    );
    assert!(
        find(body, &digest(&[vec![b'x'; trimmed.len()], vec![b'\n']].concat())).is_some(),
        "placeholder's digest carried"
    );
}

#[test]
fn saved_sign_in_keeps_real_walks_provable_and_typed_keys_intact() {
    use temper_client_shell_web::journey::{prove_lines, redact_lines, saved_lines, typed};
    let record = skein_ui_web::store::Record {
        key: jig_client::STREAM_KEY.into(),
        value: {
            let kept = w::Kept { store_version: 1, record: vec![8; 16].into() };
            let mut writer = Writer::new(
                usize::try_from(kept.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
                    .expect("kept length"),
            );
            kept.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("measured kept");
            writer.finish()
        },
    };
    let key = key();
    let walks = [walk::walked(), walk::stored_stream(record), walk::keyed(&key, false), walk::departed()];
    let mut typed_acts = 0;
    for recording in walks {
        let before = lines(&recording).expect("kept lines");
        prove_lines(before.clone()).expect("kept walk proves");
        let mut after = saved_lines(before.clone());
        let output = written(&after);
        prove_lines(after.clone()).expect("saved walk still proves");
        let (_, added, _) = redact_lines(after.clone(), &[]).expect("saved walk redacts");
        assert_eq!(added.count(Kind::Link), 0);
        assert_eq!(added.count(Kind::Stored), 0);
        for (before, after) in before.iter().zip(&after) {
            if typed(before) {
                typed_acts += 1;
                assert_eq!(after, before, "saving leaves real typed keys for add");
            }
        }
        let streams = Streams::read(&after).expect("stream records");
        absent_lines(&output, &mut after, &streams, &[7; 16]);
        absent_lines(&output, &mut after, &streams, &[8; 16]);
    }
    assert!(typed_acts > 0, "the key walk includes a secret input");
}

#[test]
fn the_key_fields_event_shows_the_secret() {
    let key = key();
    let event = walk::key_event(&key);
    assert!(find(&event, SECRET_EVENT).is_some(), "real key event declares a secret");
    assert!(
        find(&event, format!("Secret {{ len: {} }}", key.len()).as_bytes()).is_some(),
        "real key event gives byte length"
    );
    assert!(find(&event, &key[..key.len() - 1]).is_none(), "event hides the key");
}
#[test]
fn a_pasted_key_goes_and_the_replay_holds() {
    check_key_walk(false);
}
#[test]
fn a_key_typed_a_byte_at_a_time_replays() {
    check_key_walk(true);
}
#[test]
fn the_streams_name_goes() {
    let recording = walk::walked();
    let before = lines(&recording).expect("walk lines");
    let streams = Streams::read(&before).expect("stream records");
    assert!(streams.events().iter().any(|event| find(&event.record, &[7; 16]).is_some()), "hello carries stream name");
    let result = redacted(&recording);
    let proved = proved(&result);
    absent(&proved, &[7; 16]);
    let mut saves = 0;
    let mut watches = 0;
    for line in lines(&proved).expect("proved lines") {
        if let Line::Save { key, value } = &line
            && key.as_ref() == jig_client::STREAM_KEY
        {
            saves += 1;
            let kept = w::Kept::decode(&w::CEILINGS, &mut Reader::new(value)).expect("stream kept");
            assert!(kept.record.as_ref() == [b'x'; 16], "saved name is placeholder");
        }
        if let Line::Exchange { path, body, .. } = &line {
            if path.as_ref() == b"/jig/v1/watch" {
                watches += 1;
                let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(body)).expect("watch envelope");
                assert!(open.stream == [b'x'; 16], "watch name is placeholder");
            } else if path.as_ref() == b"/jig/v1/batch" {
                let batch = w::Batch::decode(&w::CEILINGS, &mut Reader::new(body)).expect("batch envelope");
                for member in &batch.members {
                    if let w::Member::Open(open) = member {
                        watches += 1;
                        assert_eq!((&open.stream), &[b'x'; 16], "batch watch name is placeholder");
                    }
                }
            }
        }
    }
    assert!(saves > 0 && watches > 0, "saved stream and watches checked");
}
#[test]
fn a_stored_stream_replays() {
    let record = lines(&walk::walked())
        .expect("walk lines")
        .into_iter()
        .find_map(|line| {
            if let Line::Save { key, value } = line {
                (key.as_ref() == jig_client::STREAM_KEY).then_some(skein_ui_web::store::Record { key, value })
            } else {
                None
            }
        })
        .expect("real stream kept record");
    let recording = walk::stored_stream(record);
    let before = lines(&recording).expect("stored walk lines");
    assert!(
        before.iter().any(|line| matches!(line, Line::Open { path, .. }
        if path.as_ref() == format!("/jig/v1/stream?version={}&replaces=07070707070707070707070707070707", w::VERSION).as_bytes())),
        "start names the stored predecessor"
    );
    let result = redacted(&recording);
    let proved = proved(&result);
    absent(&proved, &[7; 16]);
    let records = lines(&proved).expect("proved stored lines");
    assert!(
        records.iter().any(|line| matches!(line, Line::Open { path, .. }
        if path.as_ref() == format!("/jig/v1/stream?version={}&replaces=78787878787878787878787878787878", w::VERSION).as_bytes())),
        "the replacement query carries the placeholder's lowercase hex"
    );
}
#[test]
fn a_reshaped_value_is_stale_never_kept() {
    let mut records = lines(&walk::keyed(&key(), false)).expect("key walk lines");
    let (index, body) = keyed_body(&records);
    let mut request = w::Keyed::decode(&w::CEILINGS, &mut Reader::new(body)).expect("keyed envelope");
    let mut value = request.value.expect("carried key").expose().to_vec();
    value.insert(value.len() / 2, b'\n');
    request.value = Some(Secret::new(value.into()));
    let request = request;
    let Line::Exchange { body, .. } = &mut records[index] else { panic!("keyed exchange") };
    *body = encode(&request);
    let result = redacted(&written(&records));
    match prove_drawn(&result.recording, &mut |_, _| {}) {
        Err(Refusal::Stale(stale)) => {
            assert_eq!(usize::try_from(stale.line).expect("line fits"), index + 1);
            assert!(
                stale.recorded.starts_with("exchange ") && stale.asked.starts_with("exchange "),
                "exchange differs"
            );
            assert!(stale.recorded != stale.asked, "reshaped carry refuses proof");
        }
        Ok(_) | Err(Refusal::Unreadable(_) | Refusal::Unproved { .. } | Refusal::Differs { .. }) => {
            panic!("reshaped value must be stale at its exchange")
        }
    }
}
#[test]
fn a_sign_in_walks_streams_read_and_write_unchanged() {
    let recording = walk::walked();
    let mut records = lines(&recording).expect("sign-in lines");
    let streams = Streams::read(&records).expect("sign-in streams");
    assert!(!streams.events().is_empty(), "host walk carries stream records");
    streams.write(&mut records);
    assert!(written(&records) == recording, "stream and line round trip is byte-identical");
}

#[test]
fn a_sign_in_walk_and_its_pages_pass_the_last_scan() {
    let recording = redacted(&walk::walked()).recording;
    let originals = temper_client_shell_web::journey::originals(&[], &[]);
    let mut recordings = 0;
    let mut pages = 0;
    let records = lines(&recording).expect("sign-in lines");
    assert!(scan::lines(&records, &originals).expect("decoded sign-in streams").is_none(), "clean recording");
    recordings += 1;
    let mut replay = Replay::new(&recording).expect("sign-in replay");
    while let Some(mark) = replay.next().expect("sign-in walk holds") {
        assert!(scan::page(&mark.page.html(), &originals).is_none(), "clean printed page");
        pages += 1;
    }
    assert!(recordings > 0 && pages > 0, "recording and pages scanned");
}

#[test]
fn a_sign_in_walk_redacted_replays() {
    let recording = walk::walked();
    let (records, originals, _) =
        temper_client_shell_web::journey::redact_lines(lines(&recording).expect("sign-in lines"), &[])
            .expect("sign-in recording redacts");
    assert!(scan::lines(&records, &originals).expect("decoded sign-in streams").is_none());
    let mut replay = Replay::from_lines(records).expect("sign-in replay");
    let mut marks = 0;
    let mut ended = false;
    loop {
        match replay.next() {
            Ok(Some(mark)) => {
                assert!(scan::page(&mark.page.html(), &originals).is_none(), "clean sign-in page");
                marks += 1;
                ended = mark.end;
            }
            Ok(None) => break,
            Err(stale) => panic!("sign-in walk stale at line {}", stale.line),
        }
    }
    assert!(marks > 1 && ended, "several sign-in marks replayed to the end");
}
