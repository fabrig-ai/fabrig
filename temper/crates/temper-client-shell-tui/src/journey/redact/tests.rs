use super::*;
use crate::{
    host::{Duties, Host, Report},
    journey::{Facts, Kind, Recorder, Replay, Sent, Streamed, lines, prove, scan, scan_bytes},
};
use skein_lib::{Time, Token, Wall};
use skein_ui_screen::Size;
use skein_ui_screen_terminal::Key;
use temper_client_tui::{BoxLine, Scheme};

pub(in crate::journey) fn printed_link() -> Box<[u8]> {
    format!("http://127.0.0.1:8367/jig/v1/sign-in/link?secret={}", "A".repeat(43)).into_bytes().into()
}

pub(in crate::journey) fn written(records: &[Line]) -> Box<[u8]> {
    let mut bytes = Vec::new();
    for record in records {
        record.write(&mut bytes);
    }
    bytes.into()
}

fn body(secret: &[u8]) -> Box<[u8]> {
    let mut bytes = u32::try_from(secret.len()).expect("fake secret length").to_be_bytes().to_vec();
    bytes.extend_from_slice(secret);
    bytes.into()
}

fn send(message: Sent, credential: Option<Box<[u8]>>) -> Line {
    Line::Send { exchange: 1, message, credential }
}

fn places(bytes: &[u8]) -> Vec<Line> {
    places_with(bytes, &body(bytes))
}

fn places_with(bytes: &[u8], body: &[u8]) -> Vec<Line> {
    let value = || Box::<[u8]>::from(bytes);
    let mut result = vec![
        Line::Reply { exchange: 1, bytes: body.into() },
        Line::Streamed(Streamed::Record(body.into())),
        Line::Stored { key: value(), bytes: value() },
        Line::Save { key: value(), bytes: value() },
        Line::Erase(value()),
        Line::Credential(value()),
        Line::Keep(Some(value())),
        send(Sent::SignOut, Some(value())),
        Line::Stream(Some(value())),
        Line::Key { key: Key::Paste(value()), event: Some(value()) },
        Line::Address(value()),
        Line::BoxLine(value()),
        Line::BoxStage(value()),
        Line::Signed(value()),
        Line::Browse(value()),
        Line::Mark(Some(value())),
        Line::End(Some(value())),
    ];
    for message in [
        Sent::Authorize,
        Sent::ApprovalAsk,
        Sent::ApprovalRead,
        Sent::ApprovalDecide,
        Sent::SignInApproval,
        Sent::Request,
        Sent::Read,
        Sent::Watch,
        Sent::Unwatch,
        Sent::StreamGone,
        Sent::SignInLink,
    ] {
        result.push(send(message(body.into()), None));
    }
    result
}

#[test]
fn each_original_is_replaced_and_scanned_in_each_field() {
    let approval = [7; 32];
    let credential = credential();
    let bearer = [9; 40];
    let link = vec![0; 32];
    let given = b"fake&given";
    let sources = [
        (
            "approval take",
            Kind::Approval,
            approval.as_slice(),
            vec![send(Sent::SignInApproval(body(&approval)), None)],
            vec![],
        ),
        ("credential start", Kind::Credential, credential.as_ref(), vec![Line::Credential(credential.clone())], vec![]),
        ("credential keep", Kind::Credential, credential.as_ref(), vec![Line::Keep(Some(credential.clone()))], vec![]),
        ("bearer send", Kind::Bearer, bearer.as_slice(), vec![send(Sent::SignOut, Some(bearer.into()))], vec![]),
        ("bearer stream", Kind::Bearer, bearer.as_slice(), vec![Line::Stream(Some(bearer.into()))], vec![]),
        ("link send", Kind::Link, link.as_slice(), vec![send(Sent::SignInLink(body(&link)), None)], vec![]),
        (
            "box-line link",
            Kind::Link,
            link.as_slice(),
            vec![Line::BoxLine([b"link ".as_slice(), &printed_link()].concat().into())],
            vec![],
        ),
        ("given text", Kind::Given, given.as_slice(), vec![], vec![Box::<[u8]>::from(given.as_slice())]),
    ];
    let mut failures = Vec::new();
    for (name, kind, secret, source, given) in sources {
        let originals = originals(&source, &given);
        if originals.count(kind) != 1 {
            failures.push(format!("{name}: original not collected"));
        }
        let hidden = vec![b'x'; secret.len()];
        for (line, expected) in places(secret).into_iter().zip(places(&hidden)) {
            let kind = line.kind();
            let redacted = redact(&written(&[source.clone(), vec![line.clone()]].concat()), &given).expect("lines");
            let after = lines(&redacted.recording).expect("redacted lines").pop().expect("target");
            if after != expected || redacted.places == 0 {
                failures.push(format!("{name} in {kind}: replacement"));
            }
            let fields = fields_mut(&mut line.clone()).len();
            if fields == 0 {
                failures.push(format!("{name} in {kind}: field omitted"));
            }
            for (field, _) in fields_mut(&mut line.clone()) {
                if scan_bytes(field, &[], &originals) != Some(crate::journey::What::Original) {
                    failures.push(format!("{name} in {kind}: original not scanned"));
                }
            }
            for (field, _) in fields_mut(&mut after.clone()) {
                if scan_bytes(field, &[], &originals).is_some() {
                    failures.push(format!("{name} in {kind}: replacement refused"));
                }
            }
        }
    }
    // An escaped form keeps its encoded size in a framed field and shrinks to
    // the original's placeholder in text.
    let given = [Box::<[u8]>::from(given.as_slice())];
    for (line, expected) in [
        (Line::Erase(b"fake%26given".as_slice().into()), Line::Erase(vec![b'x'; 12].into())),
        (Line::Address(b"fake%26given".as_slice().into()), Line::Address(vec![b'x'; 10].into())),
    ] {
        let redacted = redact(&written(&[line]), &given).expect("encoded form");
        if lines(&redacted.recording).expect("form lines") != vec![expected] {
            failures.push("form's framing".to_owned());
        }
    }
    pasted_forms(&mut failures);
    pasted_proof(&mut failures);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn pasted_proof(failures: &mut Vec<String>) {
    // The host decodes the pasted URL's base64 into the wire's 32-byte secret.
    // Its recorded Typed event is the Debug print of the URL's byte vector.
    let mut world = World::new();
    let link = format!("http://127.0.0.1:8367/jig/v1/sign-in/link?secret={}", "A".repeat(43));
    world.report(Report::Key(Key::Paste(link.into_bytes().into())));
    world.report(Report::Key(Key::Enter));
    let file = world.save();
    assert!(prove(&file).is_ok(), "pasted link's live proof");
    let redacted = redact(&file, &[]).expect("redact pasted link");
    assert_eq!(redacted.originals.count(Kind::Link), 1);
    let original = lines(&file).expect("pasted lines");
    let after = lines(&redacted.recording).expect("redacted pasted lines");
    for (before, after) in original.iter().zip(&after) {
        if let (
            Line::Key { key: Key::Paste(before), event: before_event },
            Line::Key { key: Key::Paste(after), event: after_event },
        ) = (before, after)
        {
            if before == after {
                failures.push("native pasted link: base64 secret unchanged".to_owned());
            }
            if before_event == after_event {
                failures.push("native pasted link: decimal-byte event unchanged".to_owned());
            }
            if scan_bytes(before, &[], &redacted.originals) != Some(crate::journey::What::Original) {
                failures.push("native pasted link: base64 secret not scanned".to_owned());
            }
        }
    }
    match prove(&redacted.recording) {
        Ok(proved) => {
            if scan(&proved, &redacted.originals).expect("pasted link scan").is_some() {
                failures.push("native pasted link: redacted replay refused by scan".to_owned());
            }
            let mut replay = Replay::new(&proved).expect("pasted replay");
            while replay.next().expect("pasted duties").is_some() {}
        }
        Err(_) => failures.push("native pasted link: redaction is stale".to_owned()),
    }
}

fn pasted_forms(failures: &mut Vec<String>) {
    let source = send(Sent::SignInLink(body(&[0; 32])), None);
    let encoded = b"A".repeat(43);
    let hidden = b"eHh4".repeat(10).into_iter().chain(b"eHg".iter().copied()).collect::<Vec<_>>();
    let decimal = |bytes: &[u8]| bytes.iter().map(u8::to_string).collect::<Vec<_>>().join(", ").into_bytes();
    for (name, before, after) in [
        ("paste base64", encoded.clone(), hidden.clone()),
        (
            "event decimal",
            [b"Typed { value: [".as_slice(), &decimal(&encoded), b"] }"].concat(),
            [b"Typed { value: [".as_slice(), &decimal(&hidden), b"] }"].concat(),
        ),
    ] {
        let originals = originals(std::slice::from_ref(&source), &[]);
        // The synthetic framed body's prefix stands even when decimal elements
        // change width; the native event carrying this form is text.
        let mut framed = u32::try_from(before.len()).expect("form length").to_be_bytes().to_vec();
        framed.extend_from_slice(&after);
        for (line, expected) in places(&before).into_iter().zip(places_with(&after, &framed)) {
            let kind = line.kind();
            let field_count = fields_mut(&mut line.clone()).len();
            let redacted = redact(&written(&[source.clone(), line.clone()]), &[]).expect("encoded link");
            let after = lines(&redacted.recording).expect("redacted link").pop().expect("target");
            if after != expected || redacted.places != u32::try_from(field_count + 1).expect("places") {
                failures.push(format!("{name} in {kind}: replacement or count"));
            }
            for (field, _) in fields_mut(&mut line.clone()) {
                if scan_bytes(field, &[], &originals) != Some(crate::journey::What::Original) {
                    failures.push(format!("{name} in {kind}: not scanned"));
                }
            }
            for (field, _) in fields_mut(&mut after.clone()) {
                if scan_bytes(field, &[], &originals).is_some() {
                    failures.push(format!("{name} in {kind}: replacement refused"));
                }
            }
        }
    }
    for (prefix, suffix, replace) in
        [("[", "]", true), ("[1, ", ", 2]", true), ("[1", "]", false), ("[", "0]", false), ("", "", false)]
    {
        let before = [prefix.as_bytes(), &decimal(&encoded), suffix.as_bytes()].concat();
        let expected =
            [prefix.as_bytes(), &decimal(if replace { &hidden } else { &encoded }), suffix.as_bytes()].concat();
        let redacted = redact(&written(&[source.clone(), Line::Signed(before.clone().into())]), &[]).expect("boundary");
        if lines(&redacted.recording).expect("boundary lines").pop() != Some(Line::Signed(expected.into()))
            || scan_bytes(&before, &[], &redacted.originals).is_some() != replace
        {
            failures.push("decimal element boundary".to_owned());
        }
    }
}

pub(in crate::journey) fn credential() -> Box<[u8]> {
    let mut bytes = 7_u64.to_be_bytes().to_vec();
    bytes.extend_from_slice(&[8; 32]);
    bytes.into()
}

pub(in crate::journey) struct World {
    pub host: Host,
    pub recorder: Recorder,
    now: u64,
}
impl World {
    pub fn new() -> Self {
        Self::at_address(b"127.0.0.1:8367")
    }
    pub fn at_address(address: &[u8]) -> Self {
        Self::with_front(address, false)
    }
    fn with_front(address: &[u8], front: bool) -> Self {
        let facts = Facts {
            seed: 7,
            now: Time::ZERO,
            wall: Wall::EPOCH,
            size: Size { rows: 16, columns: 100 },
            scheme: Some(Scheme::Dark),
            background: None,
            boxed: false,
            address: address.into(),
            front,
            local: front,
            note: None,
            kept: Vec::new(),
            credential: None,
        };
        let (host, start) = facts.start();
        let mut world = Self { host, recorder: Recorder::new(&facts), now: 1 };
        world.report(start);
        world
    }
    pub fn report(&mut self, report: Report) -> Duties {
        self.reports(vec![report]).pop().expect("one report")
    }
    pub fn reports(&mut self, reports: Vec<Report>) -> Vec<Duties> {
        let now = Time::from_nanos(self.now);
        let wall = Wall::from_nanos(self.now);
        self.now += 1;
        self.host.clocks(now, wall);
        self.recorder.turn(now, wall);
        let duties = reports.into_iter().map(|report| self.recorder.report(&mut self.host, report)).collect();
        self.host.frame();
        self.recorder.deadline(self.host.deadline());
        duties
    }
    pub fn save(&mut self) -> Box<[u8]> {
        self.recorder.save(&mut self.host)
    }
    pub fn exchange(&mut self, approval: bool) -> Token {
        lines(&self.save())
            .expect("recorded lines")
            .iter()
            .rev()
            .find_map(|line| {
                if let Line::Send { exchange, message, .. } = line {
                    let wanted = if approval {
                        matches!(message, Sent::SignInApproval(_))
                    } else {
                        matches!(message, Sent::ApprovalAsk(_))
                    };
                    if wanted {
                        return Some(Token::new(*exchange));
                    }
                }
                None
            })
            .expect("sign-in exchange")
    }
}

#[test]
fn redacted_approval_proves_and_replays_to_its_end() {
    let mut world = World::new();
    let exchange = world.exchange(false);
    let mut asked = vec![0];
    asked.extend_from_slice(&body(&[7; 32]));
    asked.extend_from_slice(b"WDJBMJHT");
    asked.extend_from_slice(&300_000_000_000_u64.to_be_bytes());
    world.report(Report::Heard(jig_client::Event::Reply { exchange, message: asked.into() }));
    world.report(Report::Opened { opened: false });
    let exchange = world.exchange(true);
    let mut answer = vec![0, 1];
    answer.extend_from_slice(&body(&credential()));
    answer.extend_from_slice(&7_u64.to_be_bytes());
    answer.extend_from_slice(&300_000_000_000_u64.to_be_bytes());
    world.report(Report::Heard(jig_client::Event::Reply { exchange, message: answer.into() }));
    world.report(Report::Kept { written: true });
    let original = world.save();
    assert!(prove(&original).is_ok(), "host's live proof");
    let redacted = redact(&original, &[]).expect("redact approval");
    assert_eq!(redacted.originals.count(Kind::Approval), 1);
    assert_eq!(redacted.originals.count(Kind::Credential), 1);
    assert_eq!(redacted.originals.count(Kind::Bearer), 1);
    assert_eq!(
        scan(&original, &redacted.originals).expect("scan original").expect("original").place,
        crate::journey::Place::Field
    );
    let proved = prove(&redacted.recording).expect("redacted live proof");
    assert_eq!(scan(&proved, &redacted.originals).expect("last scan"), None);
    let mut replay = Replay::new(&proved).expect("redacted replay");
    let mut marks = 0;
    while replay.next().expect("redacted duties").is_some() {
        marks += 1;
    }
    assert!(marks > 0, "replayed to end");
}

#[test]
fn found_links_are_redacted_even_when_never_sent() {
    let url = printed_link();
    let mapping = jig_client_protocol_http::Mapping::new(b"/jig").expect("mapping prefix");
    let mut failures = Vec::new();
    for (name, source, sent) in [
        ("found never sent", Line::Found(Some(url.clone())), false),
        ("found sent", Line::Found(Some(url.clone())), true),
        ("box-line never sent", Line::BoxLine([b"link ".as_slice(), &url].concat().into()), false),
    ] {
        let mut records = vec![source];
        if sent {
            records.push(send(Sent::SignInLink(body(&[0; 32])), None));
        }
        records.push(Line::Mark(Some([b"Welcome: ".as_slice(), &url].concat().into())));
        let redacted = redact(&written(&records), &[]).expect("found lines");
        if redacted.originals.count(Kind::Link) != 1 {
            failures.push(format!("{name}: link original absent"));
        }
        let after = lines(&redacted.recording).expect("redacted found");
        if after.first() == records.first() {
            failures.push(format!("{name}: link secret unchanged"));
        }
        if after.last() == records.last() {
            failures.push(format!("{name}: printed link in mark unchanged"));
        }
        if scan_bytes(&url, &[], &redacted.originals) != Some(crate::journey::What::Original) {
            failures.push(format!("{name}: link URL-base64 not scanned"));
        }
        let redacted_url = if let Some(Line::Found(Some(bytes))) = after.first() {
            Some(bytes.clone())
        } else if let Some(Line::BoxLine(bytes)) = after.first()
            && let Some(BoxLine::Link { url }) = BoxLine::read(bytes)
        {
            Some(url.into_exposed())
        } else {
            None
        };
        if redacted_url
            .as_deref()
            .and_then(|bytes| jig_client_protocol_http::printed(&mapping, bytes).ok())
            .is_none_or(|link| link.secret.expose() != [b'x'; 32])
        {
            failures.push(format!("{name}: redacted link unreadable or wrong secret"));
        }
        for line in after {
            for (field, _) in fields_mut(&mut line.clone()) {
                if scan_bytes(field, &[], &redacted.originals).is_some() {
                    failures.push(format!("{name}: redacted {} refused", line.kind()));
                }
            }
        }
    }
    for (name, bytes) in [
        ("log containing the printed link", [b"log ".as_slice(), &url].concat()),
        ("unknown box word", [b"other ".as_slice(), &url].concat()),
        ("link without URL", b"link ".to_vec()),
        ("unreadable link", b"link not-a-link".to_vec()),
        (
            "wrong prefix",
            [b"link http://127.0.0.1:8367/other/v1/sign-in/link?secret=".as_slice(), &b"A".repeat(43)].concat(),
        ),
        ("unreadable secret", b"link http://127.0.0.1:8367/jig/v1/sign-in/link?secret=!".to_vec()),
    ] {
        if !originals(&[Line::BoxLine(bytes.into())], &[]).sought().is_empty() {
            failures.push(format!("{name}: not a link original"));
        }
    }
    let mut world = World::with_front(b"127.0.0.1:8367", true);
    world.report(Report::Front(BoxLine::Link { url: skein_lib::Secret::new(url.clone()) }));
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
    let hidden_url = url_base64(&[b'x'; 32]);
    assert!(find_from(&frame.text(), &url, 0).is_none(), "welcome no longer draws the printed link's original");
    assert!(find_from(&frame.text(), &hidden_url, 0).is_some(), "welcome draws the redacted link");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
