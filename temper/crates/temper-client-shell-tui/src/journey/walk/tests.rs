use super::*;
use crate::host::{Duties, Host, Report};
use crate::journey::{Facts, Recorder};
use jig_client::{Message, Request};
use skein_lib::{Time, Token, Wall};
use skein_ui_screen::Size;
use temper_client_tui::Scheme;

const APPROVAL: &[u8] = b"walk 1\nsee \"then press Approve.\"\nmark\napprove\nsee \"/ for commands\"\nend\n";

#[test]
fn reads_each_walk_record_with_recording_fields() {
    let file = b"walk 1\nsee \"a\\x0A\\\"\\\\\"\nkey enter\nkey end\nkey char \"\\xC3\\xA9\"\nkey alt \"x\"\nkey control \"c\"\nkey paste \"\\x00 text\"\nmark\napprove\nend\n";
    assert_eq!(
        walk(file).expect("walk"),
        vec![
            WalkStep::See(b"a\n\"\\".as_slice().into()),
            WalkStep::Key(Key::Enter),
            WalkStep::Key(Key::End),
            WalkStep::Key(Key::Char('é')),
            WalkStep::Key(Key::Alt('x')),
            WalkStep::Key(Key::Control(b'c')),
            WalkStep::Key(Key::Paste(b"\0 text".as_slice().into())),
            WalkStep::Mark,
            WalkStep::Approve,
            WalkStep::End,
        ]
    );
    assert_eq!(walk(b"walk 1\nend\n").expect("empty walk"), vec![WalkStep::End]);
}

#[test]
fn unreadable_walks_name_the_line() {
    let rows: &[(&str, &[u8], u32)] = &[
        ("empty", b"", 1),
        ("another version", b"walk 2\nend\n", 1),
        ("leading zero", b"walk 01\nend\n", 1),
        ("version missing", b"walk\nend\n", 1),
        ("extra version field", b"walk 1 extra\nend\n", 1),
        ("header missing", b"mark\nend\n", 1),
        ("header repeated", b"walk 1\nwalk 1\nend\n", 2),
        ("end missing", b"walk 1\nmark\n", 3),
        ("second end", b"walk 1\nend\nend\n", 3),
        ("step after end", b"walk 1\nend\nmark\n", 3),
        ("unknown record", b"walk 1\nother\nend\n", 2),
        ("recording record", b"walk 1\ndue\nend\n", 2),
        ("blank line", b"walk 1\n\nend\n", 2),
        ("mark field", b"walk 1\nmark extra\nend\n", 2),
        ("approve field", b"walk 1\napprove extra\nend\n", 2),
        ("end field", b"walk 1\nend extra\n", 2),
        ("see field missing", b"walk 1\nsee\nend\n", 2),
        ("see not quoted", b"walk 1\nsee bare\nend\n", 2),
        ("see extra field", b"walk 1\nsee \"text\" extra\nend\n", 2),
        ("see bad escape", b"walk 1\nsee \"\\q\"\nend\n", 2),
        ("see raw byte", b"walk 1\nsee \"\xFF\"\nend\n", 2),
        ("key missing", b"walk 1\nkey\nend\n", 2),
        ("unknown key", b"walk 1\nkey other\nend\n", 2),
        ("key with event", b"walk 1\nkey enter none\nend\n", 2),
        ("key text missing", b"walk 1\nkey paste\nend\n", 2),
        ("key extra text", b"walk 1\nkey enter \"x\"\nend\n", 2),
        ("two characters", b"walk 1\nkey char \"ab\"\nend\n", 2),
        ("invalid character", b"walk 1\nkey alt \"\\xFF\"\nend\n", 2),
        ("control not letter", b"walk 1\nkey control \"0\"\nend\n", 2),
        ("doubled space", b"walk 1\nkey  enter\nend\n", 2),
        ("trailing space", b"walk 1\nmark \nend\n", 2),
        ("missing newline", b"walk 1\nend", 2),
    ];
    for (name, file, line) in rows {
        let error = walk(file).expect_err(name);
        assert_eq!(error.line, *line, "{name}");
        assert!(!error.why.is_empty(), "{name} gives a reason");
    }
}

struct World {
    host: Host,
    recorder: Recorder,
    now: u64,
}
impl World {
    fn new() -> (Self, Duties) {
        let facts = Facts {
            seed: 7,
            now: Time::ZERO,
            wall: Wall::EPOCH,
            size: Size { rows: 24, columns: 80 },
            scheme: Some(Scheme::Dark),
            background: None,
            boxed: false,
            address: b"127.0.0.1:8367".as_slice().into(),
            front: false,
            local: false,
            note: None,
            kept: Vec::new(),
            credential: None,
        };
        let (host, start) = facts.start();
        let mut world = Self { host, recorder: Recorder::new(&facts), now: 1 };
        let duties = world.report(start);
        (world, duties)
    }
    fn turn(&mut self) {
        self.host.clocks(Time::from_nanos(self.now), Wall::from_nanos(self.now));
        self.recorder.turn(Time::from_nanos(self.now), Wall::from_nanos(self.now));
        self.now += 1;
    }
    fn report(&mut self, report: Report) -> Duties {
        self.turn();
        let duties = self.recorder.report(&mut self.host, report);
        self.finish();
        duties
    }
    fn finish(&mut self) {
        self.recorder.deadline(self.host.deadline());
        self.host.frame();
    }
    fn mark(&mut self) {
        self.turn();
        self.recorder.mark(&mut self.host);
        self.finish();
    }
    fn save(&mut self) -> Box<[u8]> {
        self.recorder.save(&mut self.host)
    }
}

fn exchange(duties: &Duties, approval_ask: bool) -> Token {
    duties
        .asked
        .iter()
        .find_map(|request| match request {
            Request::Exchange { exchange, message: Message::ApprovalAsk(_), .. } if approval_ask => Some(*exchange),
            Request::Exchange { exchange, message: Message::SignInApproval(_), .. } if !approval_ask => Some(*exchange),
            Request::Exchange { .. }
            | Request::OpenStream { .. }
            | Request::CloseStream
            | Request::Save { .. }
            | Request::Erase { .. }
            | Request::Told { .. }
            | Request::Authorized(_)
            | Request::Approval(_)
            | Request::ServiceCredential { .. } => None,
        })
        .expect("approval exchange")
}

fn approval_saves() -> [Box<[u8]>; 3] {
    let (mut world, duties) = World::new();
    let initial = world.save();
    // Jig's v20 ApprovalAskAnswer::Asked: tag, secret length and bytes, code, expiry.
    let mut asked = vec![0];
    asked.extend_from_slice(&32_u32.to_be_bytes());
    asked.extend_from_slice(&[7; 32]);
    asked.extend_from_slice(b"BCDFGHJK");
    asked.extend_from_slice(&300_000_000_000_u64.to_be_bytes());
    let duties = world
        .report(Report::Heard(jig_client::Event::Reply { exchange: exchange(&duties, true), message: asked.into() }));
    let take = exchange(&duties, false);
    world.report(Report::Opened { opened: true });
    world.mark();
    let waiting = world.save();
    // SignedIn: tag, present credential, length, sign-in id and fake secret, party, expiry.
    let mut signed_in = vec![0, 1];
    signed_in.extend_from_slice(&40_u32.to_be_bytes());
    signed_in.extend_from_slice(&7_u64.to_be_bytes());
    signed_in.extend_from_slice(&[7; 32]);
    signed_in.extend_from_slice(&7_u64.to_be_bytes());
    signed_in.extend_from_slice(&300_000_000_000_u64.to_be_bytes());
    world.report(Report::Heard(jig_client::Event::Reply { exchange: take, message: signed_in.into() }));
    world.report(Report::Kept { written: true });
    connect(&mut world);
    [initial, waiting, world.save()]
}

fn connect(world: &mut World) {
    let duties = world.report(Report::Heard(jig_client::Event::Streamed {
        message: jig_client::Streamed::Record(crate::host::tests::hello_bytes()),
    }));
    world.report(Report::Heard(jig_client::Event::Streamed {
        message: jig_client::Streamed::Record(crate::host::tests::me_bytes(&duties)),
    }));
}

fn written(records: &[Line]) -> Box<[u8]> {
    let mut bytes = Vec::new();
    for record in records {
        record.write(&mut bytes);
    }
    bytes.into()
}
fn key(key: Key) -> Line {
    Line::Key { key, event: None }
}

#[test]
fn follows_keys_in_order_and_browse_at_each_approval() {
    let [initial, _, approved] = approval_saves();
    let steps = walk(APPROVAL).expect("approval walk");
    let original = lines(&approved).expect("approval recording");
    assert_eq!(follows(&steps, &original), None);
    assert_eq!(follows(&steps, &lines(&initial).expect("initial save")), Some(3));
    let mut added = original.clone();
    added.insert(added.len() - 1, key(Key::Enter));
    assert_eq!(follows(&steps, &added), Some(5), "an added key differs at end");
    let missing = walk(b"walk 1\nsee \"ready\"\nkey enter\nend\n").expect("key walk");
    assert_eq!(follows(&missing, &original), Some(2), "a missing key differs at its step");
    let browse = Line::Browse(b"http://localhost/approve".as_slice().into());
    let steps = walk(b"walk 1\nkey tab\napprove\nkey enter\napprove\nend\n").expect("two approvals");
    for (name, records, differs) in [
        ("ordered", vec![key(Key::Tab), browse.clone(), key(Key::Enter), browse.clone()], None),
        ("reordered", vec![key(Key::Enter), browse.clone(), key(Key::Tab), browse.clone()], Some(1)),
        ("browse before preceding key", vec![browse.clone(), key(Key::Tab), key(Key::Enter), browse.clone()], Some(2)),
        ("browse beyond next key", vec![key(Key::Tab), key(Key::Enter), browse.clone()], Some(2)),
        ("earlier browse is not reused after key", vec![key(Key::Tab), browse.clone(), key(Key::Enter)], Some(4)),
        ("missing later key", vec![key(Key::Tab), browse.clone()], Some(3)),
        ("extra trailing key", vec![key(Key::Tab), browse.clone(), key(Key::Enter), browse, key(Key::Tab)], Some(5)),
    ] {
        assert_eq!(follows(&steps, &records), differs, "{name}");
    }
}

#[test]
fn saves_show_only_the_final_replayed_text_and_last_browse() {
    let [initial, waiting, approved] = approval_saves();
    for (save, waiting_shown, launchpad_shown, address) in [
        (&initial, false, false, None),
        (&waiting, true, false, Some(b"http://127.0.0.1:8367/terminal?code=BCDF-GHJK".as_slice())),
        (&approved, false, true, Some(b"http://127.0.0.1:8367/terminal?code=BCDF-GHJK".as_slice())),
    ] {
        assert_eq!(shows(save, b"then press Approve."), Ok(waiting_shown));
        assert_eq!(shows(save, b"/ for commands"), Ok(launchpad_shown));
        assert_eq!(browsed(save).expect("save's last browse").as_deref(), address);
    }
    assert_eq!(shows(&initial, b""), Ok(true));
    assert_eq!(shows(&initial, &[b'x'; 2_000]), Ok(false));
    let mut records = lines(&approved).expect("records");
    // The last deadline follows the waiting frame's mark: shows must finish replay.
    let deadline = records.iter().rposition(|line| matches!(line, Line::Deadline(_))).expect("last deadline");
    records[deadline] = Line::Deadline(Some(Time::from_nanos(u64::MAX)));
    assert!(
        matches!(shows(&written(&records), b"then press Approve."), Err(Refusal::Stale(stale)) if stale.line == u32::try_from(deadline + 1).expect("line"))
    );
    assert!(matches!(shows(b"end\n", b""), Err(Refusal::Unreadable(_))));
    let mut malformed = approved.to_vec();
    malformed.extend_from_slice(b"unknown\n");
    assert!(matches!(shows(&malformed, b""), Err(Refusal::Unreadable(_))));
    assert!(browsed(&malformed).is_err(), "the whole save must be readable");
    let mut records = lines(&approved).expect("records");
    let last = b"http://localhost/approve?code=JKLM-NPQR".as_slice();
    records.insert(records.len() - 1, Line::Browse(last.into()));
    assert_eq!(browsed(&written(&records)).expect("last browse").as_deref(), Some(last));
}
