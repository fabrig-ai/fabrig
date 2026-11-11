//! Real host and view evidence for redaction (temper/docs/design/client/web/journeys.md, 3.3 and 3.6–3.8 and 4).
use super::{walk, written};
use jig_client_protocol_wire as w;
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
    assert!(find(recording, value).is_none(), "original absent from html-free recording");
    for mut line in lines(recording).expect("html-free lines") {
        for (field, _) in fields_mut(&mut line) {
            assert!(find(field, value).is_none(), "original absent from byte field");
        }
    }
    let streams = Streams::read(&lines(recording).expect("html-free lines")).expect("stream records");
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
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("keyed size fits"));
    value.encode(&mut writer).expect("measured keyed request");
    writer.finish()
}
fn check_key_walk(byte_at_a_time: bool) {
    let key = key();
    let trimmed = &key[..key.len() - 1];
    let recording = walk::keyed(&key, byte_at_a_time);
    let records = lines(&recording).expect("key walk lines");
    let (index, body) = keyed_body(&records);
    let request = w::Keyed::decode(&w::CEILINGS, &mut Reader::new(body)).expect("keyed envelope");
    assert!(request.value().as_ref().is_some_and(|value| value.expose() == key), "carried value retains the newline");
    let ask = w::Ask::decode(&w::CEILINGS, &mut Reader::new(request.request())).expect("typed ask");
    let w::Ask::AccountChange(change) = ask else { panic!("account change") };
    let w::AccountChange::AddKey(change) = change.change() else { panic!("add key") };
    assert!(*change.digest() == digest(&key), "typed ask holds the whole value's raw sha-256");
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
        request
            .value()
            .as_ref()
            .is_some_and(|value| value.expose() == [vec![b'x'; trimmed.len()], vec![b'\n']].concat()),
        "placeholder carried once"
    );
    assert!(
        find(body, &digest(&[vec![b'x'; trimmed.len()], vec![b'\n']].concat())).is_some(),
        "placeholder's digest carried"
    );
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
            assert!(kept.record() == [b'x'; 16], "saved name is placeholder");
        }
        if let Line::Exchange { path, body, .. } = &line
            && path.as_ref() == b"/jig/v1/watch"
        {
            watches += 1;
            let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(body)).expect("watch envelope");
            assert!(open.stream() == &[b'x'; 16], "watch name is placeholder");
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
        before.iter().any(|line| matches!(line, Line::Exchange { path, body, .. }
        if path.as_ref() == b"/jig/v1/stream-gone" && find(body, &[7; 16]).is_some())),
        "start drops stored predecessor"
    );
    let result = redacted(&recording);
    let proved = proved(&result);
    absent(&proved, &[7; 16]);
    let records = lines(&proved).expect("proved stored lines");
    assert!(
        records.iter().any(|line| matches!(line, Line::Exchange { path, body, .. }
        if path.as_ref() == b"/jig/v1/stream-gone" && find(body, &[b'x'; 16]).is_some())),
        "stream-gone carries placeholder"
    );
}
#[test]
fn a_reshaped_value_is_stale_never_kept() {
    let mut records = lines(&walk::keyed(&key(), false)).expect("key walk lines");
    let (index, body) = keyed_body(&records);
    let mut request = w::Keyed::decode(&w::CEILINGS, &mut Reader::new(body)).expect("keyed envelope").into_parts();
    let mut value = request.value.expect("carried key").expose().to_vec();
    value.insert(value.len() / 2, b'\n');
    request.value = Some(Secret::new(value.into()));
    let request = w::Keyed::new(&w::CEILINGS, request).expect("reshaped envelope");
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
fn a_committed_journey_redacted_replays() {
    let welcome = include_bytes!("../../../../../docs/design/client/web/journeys/welcome-playground/recording");
    let mut chat = lines(include_bytes!("../../../../../docs/design/client/web/journeys/chat-draft/recording"))
        .expect("chat lines");
    // The full chat replay takes over 300 ms; keep the turn of its second open.
    let open = chat
        .iter()
        .enumerate()
        .filter(|(_, line)| matches!(line, Line::Open { .. }))
        .nth(1)
        .expect("chat's second open")
        .0;
    let deadline = chat
        .iter()
        .enumerate()
        .skip(open + 1)
        .find(|(_, line)| matches!(line, Line::Deadline(_)))
        .expect("second open's deadline")
        .0;
    chat.truncate(deadline + 1);
    chat.push(Line::End(None));
    assert_eq!(chat.iter().filter(|line| matches!(line, Line::Open { .. })).count(), 2);
    for recording in [welcome.as_slice(), &written(&chat)] {
        let result = redacted(recording);
        assert!(
            scan::lines(&lines(&result.recording).expect("committed lines"), &result.originals)
                .expect("decoded committed streams")
                .is_none()
        );
        let mut replay = Replay::new(&result.recording).expect("committed replay");
        let mut marks = 0;
        loop {
            match replay.next() {
                Ok(Some(mark)) => {
                    assert!(scan::page(&mark.page.html(), &result.originals).is_none(), "clean committed page");
                    marks += 1;
                }
                Ok(None) => break,
                Err(stale) => panic!("committed journey stale at line {}", stale.line),
            }
        }
        assert!(marks > 1, "committed journey reaches its end");
    }
}
