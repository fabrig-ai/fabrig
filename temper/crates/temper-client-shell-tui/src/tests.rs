use crate::{
    host::{Duties, Host, Report, Settings, Start},
    limits,
};
use jig_client::{Message, Request};
use jig_client_protocol_wire::v19 as w;
use jig_client_test_records as records;
use skein_lib::{Duration, List, Reader, Secret, Time, Wall, Writer};
use skein_ui_screen::{Rgb, Size};
use skein_ui_screen_terminal::Key;
use temper_client_domain::{
    self as root, Address, Place,
    pages::{
        Act,
        terminal::{self, Command, Font, Theme},
    },
};
use temper_client_tui::{Event, Local, Note, Own, Scheme, Told, WalkStep};

fn host() -> Host {
    Host::new(&Settings { limits: limits::tui() }, 7, Time::ZERO, Wall::EPOCH)
}

fn start(host: &mut Host, credential: Option<Secret>) -> Duties {
    start_look(host, credential, None, false)
}

fn start_look(host: &mut Host, credential: Option<Secret>, background: Option<Rgb>, boxed: bool) -> Duties {
    host.report(Report::Start(Start {
        front: false,
        size: Size { columns: 80, rows: 24 },
        scheme: Scheme::Dark,
        background,
        boxed,
        address: b"127.0.0.1:8367".as_slice().into(),
        local: false,
        note: None,
        kept: Vec::new(),
        credential,
    }))
}

fn link(address: &str) -> Vec<Key> {
    vec![
        Key::Paste(format!("http://{address}/jig/v1/sign-in/link?secret={}", "A".repeat(43)).into_bytes().into()),
        Key::Enter,
    ]
}

fn text(host: &mut Host) -> String {
    String::from_utf8(host.frame().text().into_vec()).expect("UTF-8 frame")
}

fn empty(duties: &Duties) {
    assert!(duties.asked.is_empty());
    assert!(duties.keep.is_none());
    assert!(duties.note.is_none());
    assert!(!duties.local);
    assert!(duties.address.is_none());
    assert!(duties.open.is_none());
    assert!(!duties.close);
}

#[test]
fn a_box_line_report_is_told_to_the_tui() {
    use temper_client_tui::{BoxLine, BoxState};

    let mut host = host();
    start(&mut host, None);
    let step = b"forgejo-3".as_slice();
    let line = b"forge is restarting".as_slice();
    empty(&host.report(Report::Front(BoxLine::Restarting { step: step.into() })));
    empty(&host.report(Report::Front(BoxLine::Log { line: line.into() })));

    let state = host.state.as_ref().expect("started TUI");
    assert_eq!(state.box_state(), BoxState::Restarting { step });
    assert_eq!(state.box_log().collect::<Vec<_>>(), vec![line]);
}

#[test]
fn a_pasted_link_is_checked_before_it_is_sent() {
    let rows = [
        (vec![Key::Paste(b"not a link".as_slice().into()), Key::Enter], false, "That is not a sign-in link"),
        (link("127.0.0.1:9000"), false, "That link is for 127.0.0.1:9000"),
        (link("127.0.0.1:8367"), true, "Signing in…"),
    ];
    for (keys, sends, line) in rows {
        let mut host = host();
        ask(&start(&mut host, None));
        host.frame();
        let mut asked = Vec::new();
        for key in keys {
            let duties = host.report(Report::Key(key));
            assert!(duties.keep.is_none());
            asked.extend(duties.asked);
        }
        if sends {
            assert_eq!(asked.len(), 1);
            assert!(matches!(
                asked.first(),
                Some(Request::Exchange { credential: None, message: Message::SignInLink(_), .. })
            ));
            assert!(!text(&mut host).contains(&"A".repeat(43)));
        } else {
            assert!(asked.is_empty(), "{line}: {asked:?}");
        }
        assert!(text(&mut host).contains(line), "{line}: {}", text(&mut host));
    }
}

#[test]
fn the_credential_is_kept_and_erased_as_the_link_says() {
    for (held, signed_out) in [(false, false), (true, false), (true, true)] {
        let mut host = host();
        let credential = held.then(|| Secret::new(vec![7; 40].into_boxed_slice()));
        let duties = start(&mut host, credential.clone());
        assert!(duties.keep.is_none());
        if held {
            assert!(duties.asked.iter().any(|request| matches!(request,
                Request::OpenStream { credential: offered } if offered == &credential
            )));
        } else {
            ask(&duties);
        }
        if signed_out {
            let duties =
                host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::SignedOut }));
            assert_eq!(duties.keep, Some(None));
            assert!(text(&mut host).contains("Signed out."));
            assert!(host.report(Report::Due).keep.is_none());
        }
    }
}

#[test]
#[should_panic(expected = "a report before the TUI's start")]
fn a_report_before_start_panics() {
    host().report(Report::Due);
}

#[test]
fn reports_keep_their_order() {
    let mut host = host();
    assert_eq!(host.frame().size(), Size { columns: 0, rows: 0 });
    ask(&start(&mut host, None));
    host.frame();
    assert_eq!(host.event_of(&Key::Enter), Some(Event::SignIn));
    assert_eq!(host.event_of(&Key::Char('h')), Some(Event::Typed { value: b"h".as_slice().into() }));
    assert_eq!(host.event_of(&Key::Control(b'c')), None);
    // Reading an event leaves the field and focus unchanged.
    assert_eq!(host.event_of(&Key::Char('h')), Some(Event::Typed { value: b"h".as_slice().into() }));
    for key in link("127.0.0.1:8367") {
        let duties = host.report(Report::Key(key));
        for request in duties.asked {
            if let Request::Exchange { exchange, .. } = request {
                host.report(Report::Heard(jig_client::Event::Lost { exchange }));
            }
        }
    }
    assert!(text(&mut host).contains("no answer from the engine"));
    let size = Size { columns: 40, rows: 12 };
    host.report(Report::Resize(size));
    assert_eq!(host.frame().size(), size);
    assert!(host.report(Report::Key(Key::Control(b'c'))).close);
    assert_eq!(host.deadline(), None);
    empty(&host.report(Report::Key(Key::Enter)));
    empty(&host.report(Report::Resize(Size { columns: 1, rows: 1 })));
    empty(&host.report(Report::Due));
    assert_eq!(host.frame().size(), size);
    host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::SignedOut }));
    host.report(Report::Kept { written: false });
}

#[test]
fn the_shipped_limits_are_valid() {
    let limits = limits::tui();
    assert!(limits::valid(&limits));
    let _ = Host::new(&Settings { limits: limits.clone() }, 7, Time::ZERO, Wall::EPOCH);
    let mut too_small = limits;
    too_small.view.nodes = 0;
    assert!(!limits::valid(&too_small));
}

#[test]
fn the_run_is_made_from_the_shipped_limits() {
    use skein_ui_screen_terminal::{Depth, Terminal};
    let limits = limits::tui();
    assert!(limits::valid(&limits));
    let settings = limits::terminal(&limits, Depth::Indexed, true);
    assert_eq!(settings.interval, limits.interval);
    assert_eq!(settings.depth, Depth::Indexed);
    assert_eq!(settings.most, limits.most);
    assert_eq!(settings.paste, limits.paste);
    assert!(Terminal::operations(&settings).is_some());
    let mut too_large = limits;
    too_large.most = Size { rows: u16::MAX, columns: u16::MAX };
    assert!(!limits::valid(&too_large));
}

fn ask(duties: &Duties) -> skein_lib::Token {
    let exchanges = duties
        .asked
        .iter()
        .filter_map(|request| {
            if let Request::Exchange { exchange, message: Message::ApprovalAsk(_), credential: None } = request {
                Some(*exchange)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(exchanges.len(), 1, "one approval ask");
    exchanges[0]
}

fn reply(host: &mut Host, exchange: skein_lib::Token, message: Box<[u8]>) -> Duties {
    host.report(Report::Heard(jig_client::Event::Reply { exchange, message }))
}

fn approval_bytes(code: &[u8], tail: &[u8]) -> Box<[u8]> {
    use jig_client_protocol_wire::v19 as w;
    let record = w::ApprovalAskAnswer::Asked(
        w::ApprovalAsked::new(
            &w::CEILINGS,
            w::ApprovalAskedParts {
                secret: Secret::new(Box::new([7; 32])),
                code: *b"WDJBMJHT",
                expires: 600_000_000_000,
            },
        )
        .expect("test approval"),
    );
    let mut writer = skein_lib::Writer::new(usize::try_from(record.measure()).expect("test length"));
    record.encode(&mut writer).expect("test record");
    let mut bytes = writer.finish().into_vec();
    let offset = bytes.windows(8).position(|part| part == b"WDJBMJHT").expect("code");
    bytes.splice(offset..offset + 8, code.iter().copied().filter(|byte| *byte != b'-'));
    bytes.extend_from_slice(tail);
    bytes.into_boxed_slice()
}

#[test]
fn approval_is_asked_once_per_opening() {
    for held in [false, true] {
        let mut host = host();
        let duties = start(&mut host, held.then(|| Secret::new(Box::new([7; 40]))));
        let exchange = if held {
            assert!(
                !duties
                    .asked
                    .iter()
                    .any(|request| matches!(request, Request::Exchange { message: Message::ApprovalAsk(_), .. }))
            );
            let duties =
                host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::SignedOut }));
            ask(&duties)
        } else {
            ask(&duties)
        };
        assert!(host.report(Report::Resize(Size { columns: 80, rows: 24 })).asked.is_empty());
        host.frame();
        assert!(host.report(Report::Key(Key::Enter)).asked.is_empty(), "no second ask while asking");
        reply(&mut host, exchange, approval_bytes(b"WDJBMJHT", b""));
        host.tell(temper_client_tui::Told::Link { link: jig_client::Link::Live });
        // A fresh screen replaces an ask the root still holds from the earlier opening.
        let duties =
            host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::SignedOut }));
        let exchange = ask(&duties);
        assert!(host.report(Report::Key(Key::Control(b'c'))).close);
        assert!(reply(&mut host, exchange, approval_bytes(b"BCDFGHJK", b"")).open.is_none());
        assert!(host.report(Report::Due).asked.is_empty());
    }
}

#[test]
fn approval_addresses_use_only_the_tuis_authority_and_a_valid_code() {
    for (address, code, tail, opens) in [
        ("127.0.0.1:8367", "WDJB-MJHT", "", true),
        ("127.0.0.1:9000", "WDJB-MJHT", "", true),
        ("127.0.0.1:8367", "wdjb-mjht", "", false),
        ("127.0.0.1:8367", "WDJB-MJH0", "", false),
        ("127.0.0.1:8367", "WDJB-MJHT&next=http://x", "", false),
        ("127.0.0.1:8367", "WDJB-MJHT", "http://evil.example/", false),
    ] {
        let mut host = host();
        let duties = host.report(Report::Start(Start {
            front: false,
            size: Size { columns: 80, rows: 24 },
            scheme: Scheme::Dark,
            background: None,
            boxed: false,
            address: address.as_bytes().into(),
            local: false,
            note: None,
            kept: Vec::new(),
            credential: None,
        }));
        let exchange = ask(&duties);
        let duties = reply(&mut host, exchange, approval_bytes(code.as_bytes(), tail.as_bytes()));
        let expected =
            opens.then(|| format!("http://{address}/terminal?code=WDJB-MJHT").into_bytes().into_boxed_slice());
        assert_eq!(duties.open, expected, "{address} {code} {tail}");
        let repeated = host.report(Report::Resize(Size { columns: 80, rows: 24 }));
        assert!(repeated.open.is_none());
        assert!(repeated.asked.is_empty(), "no ask until Enter or a new opening");
    }
}

#[test]
fn approval_enter_replaces_only_an_ended_or_pasted_take() {
    use jig_client_protocol_wire::v19 as w;
    for answer in [
        w::ApprovalSignInAnswer::Refused,
        w::ApprovalSignInAnswer::Expired,
        w::ApprovalSignInAnswer::Used,
        w::ApprovalSignInAnswer::Unknown,
        w::ApprovalSignInAnswer::Full(
            w::ApprovalFull::new(&w::CEILINGS, w::ApprovalFullParts { sign_ins: 2, earliest: 400, until: 300 })
                .expect("full"),
        ),
        w::ApprovalSignInAnswer::Waiting,
    ] {
        let waiting = answer == w::ApprovalSignInAnswer::Waiting;
        let mut host = host();
        let exchange = ask(&start(&mut host, None));
        let duties = reply(&mut host, exchange, approval_bytes(b"WDJBMJHT", b""));
        let take = duties
            .asked
            .iter()
            .find_map(|request| {
                if let Request::Exchange { exchange, message: Message::SignInApproval(_), .. } = request {
                    Some(*exchange)
                } else {
                    None
                }
            })
            .expect("approval take");
        let mut writer = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("test length"));
        answer.encode(&mut writer).expect("take answer");
        reply(&mut host, take, writer.finish());
        host.frame();
        let duties = host.report(Report::Key(Key::Enter));
        if waiting {
            assert!(duties.asked.is_empty());
            assert_eq!(
                host.state().signing(),
                &temper_client_tui::Signing::Refused(temper_client_tui::Refusal::NotALink)
            );
            for key in link("127.0.0.1:8367") {
                host.report(Report::Key(key));
            }
            assert!(host.state().pasted());
            ask(&host.report(Report::Key(Key::Enter)));
        } else {
            let exchange = ask(&duties);
            let duties = reply(&mut host, exchange, approval_bytes(b"BCDFGHJK", b""));
            assert_eq!(duties.open.as_deref(), Some(b"http://127.0.0.1:8367/terminal?code=BCDF-GHJK".as_slice()));
            assert!(!host.state().pasted());
        }
    }
    for answer in [
        Some(w::ApprovalAskAnswer::Refused),
        Some(w::ApprovalAskAnswer::NotReady),
        Some(w::ApprovalAskAnswer::Busy),
        None,
    ] {
        let mut host = host();
        let exchange = ask(&start(&mut host, None));
        if let Some(answer) = answer {
            let mut writer = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("length"));
            answer.encode(&mut writer).expect("answer");
            reply(&mut host, exchange, writer.finish());
        } else {
            host.report(Report::Heard(jig_client::Event::Lost { exchange }));
        }
        host.frame();
        ask(&host.report(Report::Key(Key::Enter)));
    }
}

#[test]
fn approval_deadline_includes_the_views_tick() {
    use jig_client_protocol_wire::v19 as w;
    let mut host = host();
    let exchange = ask(&start(&mut host, None));
    let duties = reply(&mut host, exchange, approval_bytes(b"WDJBMJHT", b""));
    let take = duties
        .asked
        .iter()
        .find_map(|request| {
            if let Request::Exchange { exchange, message: Message::SignInApproval(_), .. } = request {
                Some(*exchange)
            } else {
                None
            }
        })
        .expect("take");
    host.clocks(Time::from_nanos(11), Wall::from_nanos(59_999_999_999));
    let answer = w::ApprovalSignInAnswer::Waiting;
    let mut writer = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("length"));
    answer.encode(&mut writer).expect("answer");
    reply(&mut host, take, writer.finish());
    let tick = host.view.tick().expect("expiry tick");
    let delta = tick.as_nanos().saturating_sub(host.env.wall.as_nanos());
    let expected = host.env.now.saturating_add(skein_lib::Duration::from_nanos(delta));
    assert!(expected < temper_client_domain::next_deadline(&host.root).expect("take deadline"));
    assert_eq!(host.deadline(), Some(expected));
}

#[test]
fn approval_opened_reports_reach_the_tui() {
    let mut host = host();
    let exchange = ask(&start(&mut host, None));
    reply(&mut host, exchange, approval_bytes(b"WDJBMJHT", b""));
    assert_eq!(host.state().browser(), Some(temper_client_tui::Browser::Opening));
    for opened in [false, true] {
        empty(&host.report(Report::Opened { opened }));
        assert_eq!(
            host.state().browser(),
            Some(if opened { temper_client_tui::Browser::Opened } else { temper_client_tui::Browser::None })
        );
    }
    let page = b"http://127.0.0.1:8367/projects/3".as_slice();
    let other = b"http://127.0.0.1:8367/chats/9".as_slice();
    for (approval, pages, results) in [
        (false, vec![page], vec![(false, Some(page))]),
        (false, vec![page], vec![(true, None)]),
        (false, vec![page, other], vec![(false, Some(page)), (false, Some(other))]),
        (true, vec![page], vec![(false, None), (false, Some(page))]),
    ] {
        let mut host = if approval {
            let mut host = self::host();
            let exchange = ask(&start(&mut host, None));
            reply(&mut host, exchange, approval_bytes(b"WDJBMJHT", b""));
            host
        } else {
            signed_host(true).0
        };
        for address in pages {
            let duties = give(&mut host, Event::Open { address: address.into() });
            assert_eq!(duties.open.as_deref(), Some(address));
        }
        for (opened, note) in results {
            host.report(Report::Opened { opened });
            assert_eq!(host.state().not_opened(), note, "approval={approval}, opened={opened}");
        }
    }
}

#[test]
fn the_opener_gets_one_argument_and_no_terminal_descriptors() {
    use crate::opener::{self, Opener};
    use skein_io::kernel::{Fd, Pipe, Way};
    for (program, address, env) in [
        ("/usr/bin/xdg-open", "http://127.0.0.1:8367/terminal?code=WDJB-MJHT", vec!["DISPLAY=:1", "PATH=/usr/bin"]),
        ("/usr/bin/open", "http://127.0.0.1:9000/terminal?code=BCDF-GHJK", vec!["HOME=/person"]),
        ("/usr/bin/xdg-open", "http://127.0.0.1:8367/projects/3", vec!["DISPLAY=:1"]),
    ] {
        let opener = Opener {
            program: program.as_bytes().into(),
            env: env.iter().map(|value| Box::from(value.as_bytes())).collect(),
        };
        let root = Fd::new(91);
        let spawn = opener::spawn(&opener, address.as_bytes(), root);
        assert_eq!(spawn.program, opener.program);
        assert_eq!(spawn.args.as_ref(), &[Box::<[u8]>::from(address.as_bytes())]);
        assert_eq!(spawn.env, opener.env);
        assert_eq!(spawn.root, root);
        assert_eq!(spawn.dir.as_ref(), b".");
        assert_eq!(
            spawn.pipes.as_ref(),
            &[
                Pipe { child: 0, way: Way::Null, parent: None },
                Pipe { child: 1, way: Way::Null, parent: None },
                Pipe { child: 2, way: Way::Null, parent: None },
            ]
        );
    }
}

// The answer's expiry is five minutes; the host's slack is one minute.
const UNTIL: u64 = 300_000_000_000;
const SECOND: u64 = 1_000_000_000;
const LOCAL_SECRET: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const FRESH_SECRET: &str = "AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQE";

pub(crate) fn sign_in_bytes(answer: &w::SignInAnswer) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("answer length"));
    answer.encode(&mut writer).expect("answer fits");
    writer.finish()
}

pub(crate) fn signed_in_answer() -> w::SignInAnswer {
    w::SignInAnswer::SignedIn(
        w::SignedIn::new(
            &w::CEILINGS,
            w::SignedInParts { token: Some(Secret::new(Box::new([7; 40]))), party: 7, expires: UNTIL },
        )
        .expect("signed in"),
    )
}

fn local_link(address: &str, secret: &str) -> Secret {
    Secret::new(format!("http://{address}/jig/v1/sign-in/link?secret={secret}").into_bytes().into())
}

enum LocalAct {
    Found(Option<Secret>),
    Answer(w::SignInAnswer),
    Lost,
    Due(u64),
    End(u64),
    Paste,
    Approve,
    Connect,
}
#[derive(Clone, Copy)]
enum LocalWant {
    Read,
    Ask,
    Send,
    SendHeld,
    Connecting,
    Wait,
    Keep,
    Pasted,
    Taking,
    Until,
    Ended,
    Quiet,
}
struct LocalRow {
    front: bool,
    name: &'static str,
    local: bool,
    note: Option<Note>,
    held: bool,
    first: LocalWant,
    steps: Vec<(LocalAct, LocalWant)>,
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table holds the engine-link outcomes and sign-out judgements")]
fn the_engines_link_signs_in_and_falls_back() {
    use LocalAct::{Answer, Approve, Connect, Due, End, Found, Lost, Paste};
    use LocalWant::{Ask, Connecting, Ended, Keep, Pasted, Quiet, Read, Send, SendHeld, Taking, Until, Wait};
    let found = || Found(Some(local_link("127.0.0.1:8367", LOCAL_SECRET)));
    let fresh = || Found(Some(local_link("127.0.0.1:8367", FRESH_SECRET)));
    let row = |name, steps| LocalRow { front: false, name, local: true, note: None, held: false, first: Read, steps };
    let mut rows = vec![
        row("fresh", vec![]),
        LocalRow { front: false, name: "disabled", local: false, note: None, held: false, first: Ask, steps: vec![] },
        LocalRow {
            front: false,
            name: "signed out note",
            local: true,
            note: Some(Note::SignedOut),
            held: false,
            first: Ask,
            steps: vec![],
        },
        row("no file", vec![(Found(None), Ask)]),
        row("garbage", vec![(Found(Some(Secret::new(b"garbage".as_slice().into()))), Ask)]),
        row("other address", vec![(Found(Some(local_link("127.0.0.1:9000", LOCAL_SECRET))), Ask)]),
        row("signed in", vec![(found(), Send), (Answer(signed_in_answer()), Keep), (Connect, Until)]),
        row(
            "used then same then fresh",
            vec![
                (found(), Send),
                (Answer(w::SignInAnswer::LinkUsed), Wait),
                (Due(SECOND - 1), Quiet),
                (Due(SECOND), Read),
                (found(), Wait),
                (Due(2 * SECOND), Read),
                (fresh(), Send),
                (Answer(signed_in_answer()), Keep),
                (Connect, Until),
            ],
        ),
        row(
            "resend after fallback",
            vec![
                (found(), Send),
                (Answer(w::SignInAnswer::NotReady), Ask),
                (Due(SECOND), Send),
                (Answer(signed_in_answer()), Keep),
                (Connect, Until),
            ],
        ),
        row(
            "three used",
            vec![
                (found(), Send),
                (Answer(w::SignInAnswer::LinkUsed), Wait),
                (Due(SECOND), Read),
                (fresh(), Send),
                (Answer(w::SignInAnswer::LinkUsed), Wait),
                (Due(2 * SECOND), Read),
                (found(), Send),
                (Answer(w::SignInAnswer::LinkUsed), Ask),
            ],
        ),
        row(
            "same reads spent",
            vec![
                (found(), Send),
                (Answer(w::SignInAnswer::LinkExpired), Wait),
                (Due(SECOND), Read),
                (found(), Wait),
                (Due(2 * SECOND), Read),
                (found(), Ask),
            ],
        ),
    ];
    for (name, answer) in [
        ("unknown", w::SignInAnswer::LinkUnknown),
        ("refused", w::SignInAnswer::Refused),
        ("not ready", w::SignInAnswer::NotReady),
        ("busy", w::SignInAnswer::Busy),
        (
            "full",
            w::SignInAnswer::Full(
                w::SignInsFull::new(&w::CEILINGS, w::SignInsFullParts { sign_ins: 2, earliest: 400, link_until: 300 })
                    .expect("full"),
            ),
        ),
    ] {
        rows.push(row(name, vec![(found(), Send), (Answer(answer), Ask)]));
    }
    rows.push(row("lost", vec![(found(), Send), (Lost, Ask)]));
    for (name, answer) in [("front not ready", w::SignInAnswer::NotReady), ("front busy", w::SignInAnswer::Busy)] {
        rows.push(LocalRow {
            front: true,
            name,
            local: true,
            note: None,
            held: false,
            first: Read,
            steps: vec![
                (found(), Send),
                (Answer(answer), Connecting),
                (Due(SECOND), Send),
                (Answer(signed_in_answer()), Keep),
                (Connect, Until),
            ],
        });
    }
    for (name, wall, want) in [
        ("before slack", 239_999_999_999, Ended),
        ("at slack", 240_000_000_000, Read),
        ("at expiry", UNTIL, Read),
        ("after expiry", UNTIL + 1, Read),
    ] {
        rows.push(LocalRow {
            front: false,
            name,
            local: true,
            note: Some(Note::Until(Wall::from_nanos(UNTIL))),
            held: true,
            first: Quiet,
            steps: vec![(End(wall), want), (Due(SECOND), Quiet)],
        });
    }
    for (name, wall, want) in
        [("signed in then ended", 239_999_999_999, Ended), ("signed in then expired", 240_000_000_000, Read)]
    {
        rows.push(row(
            name,
            vec![
                (found(), Send),
                (Answer(signed_in_answer()), Keep),
                (Connect, Until),
                (End(wall), want),
                (Due(SECOND), Quiet),
            ],
        ));
    }
    rows.push(LocalRow {
        front: false,
        name: "no held note",
        local: true,
        note: None,
        held: true,
        first: Quiet,
        steps: vec![(End(UNTIL), Read)],
    });
    rows.push(LocalRow {
        front: false,
        name: "held signed out",
        local: true,
        note: Some(Note::SignedOut),
        held: true,
        first: Quiet,
        steps: vec![(End(UNTIL), Ask)],
    });
    rows.push(LocalRow {
        front: false,
        name: "paste after sign out",
        local: true,
        note: Some(Note::SignedOut),
        held: false,
        first: Ask,
        steps: vec![(Paste, Pasted), (Answer(signed_in_answer()), Keep), (Connect, Until)],
    });
    rows.push(LocalRow {
        front: false,
        name: "approval after sign out",
        local: true,
        note: Some(Note::SignedOut),
        held: false,
        first: Ask,
        steps: vec![(Approve, Taking), (Answer(signed_in_answer()), Keep), (Connect, Until), (Due(SECOND), Quiet)],
    });
    rows.push(LocalRow {
        front: false,
        name: "same expiry already noted",
        local: true,
        note: Some(Note::Until(Wall::from_nanos(UNTIL))),
        held: false,
        first: Read,
        steps: vec![(found(), Send), (Answer(signed_in_answer()), Keep), (Connect, Quiet)],
    });
    rows.push(LocalRow {
        front: false,
        name: "held has no expiry",
        local: true,
        note: None,
        held: true,
        first: Quiet,
        steps: vec![(Connect, Quiet)],
    });
    rows.extend([
        LocalRow {
            front: true,
            name: "front fresh",
            local: true,
            note: None,
            held: false,
            first: Read,
            steps: vec![(found(), Send), (Answer(signed_in_answer()), Keep)],
        },
        LocalRow {
            front: true,
            name: "front held",
            local: true,
            note: None,
            held: true,
            first: Read,
            steps: vec![
                (found(), SendHeld),
                (
                    Answer(w::SignInAnswer::SignedIn(
                        w::SignedIn::new(&w::CEILINGS, w::SignedInParts { token: None, party: 7, expires: UNTIL })
                            .expect("held sign-in"),
                    )),
                    Quiet,
                ),
            ],
        },
        LocalRow {
            front: true,
            name: "front held reread",
            local: true,
            note: None,
            held: true,
            first: Read,
            steps: vec![
                (found(), SendHeld),
                (Answer(w::SignInAnswer::LinkUsed), Wait),
                (Due(SECOND), Read),
                (fresh(), SendHeld),
            ],
        },
        LocalRow {
            front: true,
            name: "front held then expiry",
            local: true,
            note: None,
            held: true,
            first: Read,
            steps: vec![
                (found(), SendHeld),
                (
                    Answer(w::SignInAnswer::SignedIn(
                        w::SignedIn::new(&w::CEILINGS, w::SignedInParts { token: None, party: 7, expires: UNTIL })
                            .expect("held sign-in"),
                    )),
                    Quiet,
                ),
                (Connect, Until),
                (End(UNTIL), Read),
                (fresh(), Send),
            ],
        },
        LocalRow {
            front: true,
            name: "front signed-out note",
            local: true,
            note: Some(Note::SignedOut),
            held: false,
            first: Read,
            steps: vec![],
        },
    ]);
    let mut failures = Vec::new();
    for row in rows {
        local_row(row, &mut failures);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn local_row(row: LocalRow, failures: &mut Vec<String>) {
    let mut host = host();
    let duties = host.report(Report::Start(Start {
        front: row.front,
        size: Size { columns: 80, rows: 24 },
        scheme: Scheme::Dark,
        background: None,
        boxed: false,
        address: b"127.0.0.1:8367".as_slice().into(),
        local: row.local,
        kept: Vec::new(),
        credential: row.held.then(|| Secret::new(Box::new([7; 40]))),
        note: row.note,
    }));
    if host.state().front() != row.front {
        failures.push(format!("{}: front state", row.name));
    }
    local_check(&mut host, &duties, row.first, row.name, failures);
    let mut exchange = local_exchange(&duties);
    for (act, want) in row.steps {
        let duties = match act {
            LocalAct::Found(link) => host.report(Report::Local { link }),
            LocalAct::Answer(answer) => reply(&mut host, exchange, sign_in_bytes(&answer)),
            LocalAct::Lost => host.report(Report::Heard(jig_client::Event::Lost { exchange })),
            LocalAct::Due(now) => {
                host.clocks(Time::from_nanos(now), Wall::from_nanos(now));
                host.report(Report::Due)
            }
            LocalAct::End(wall) => {
                host.clocks(Time::ZERO, Wall::from_nanos(wall));
                host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::SignedOut }))
            }
            LocalAct::Approve => reply(&mut host, exchange, approval_bytes(b"WDJBMJHT", b"")),
            LocalAct::Paste => {
                host.frame();
                host.report(Report::Key(Key::Paste(local_link("127.0.0.1:8367", LOCAL_SECRET).into_exposed())));
                host.report(Report::Key(Key::Enter))
            }
            LocalAct::Connect => connect(&mut host),
        };
        local_check(&mut host, &duties, want, row.name, failures);
        if duties.asked.iter().any(|request| {
            matches!(
                request,
                Request::Exchange {
                    message: Message::SignInLink(_) | Message::SignInApproval(_) | Message::ApprovalAsk(_),
                    ..
                }
            )
        }) {
            exchange = local_exchange(&duties);
        }
    }
}
fn local_exchange(duties: &Duties) -> skein_lib::Token {
    duties
        .asked
        .iter()
        .find_map(|request| {
            if let Request::Exchange {
                exchange,
                message: Message::SignInLink(_) | Message::SignInApproval(_) | Message::ApprovalAsk(_),
                ..
            } = request
            {
                Some(*exchange)
            } else {
                None
            }
        })
        .unwrap_or(skein_lib::Token::new(u64::MAX))
}
fn local_check(host: &mut Host, duties: &Duties, want: LocalWant, name: &str, failures: &mut Vec<String>) {
    let asks = duties
        .asked
        .iter()
        .filter(|request| matches!(request, Request::Exchange { message: Message::ApprovalAsk(_), .. }))
        .count();
    let sends = duties
        .asked
        .iter()
        .filter(|request| {
            matches!(request, Request::Exchange { message: Message::SignInLink(_), credential, .. }
                if if matches!(want, LocalWant::SendHeld) {
                    credential.as_ref().is_some_and(|held| held.expose() == [7; 40])
                } else { credential.is_none() })
        })
        .count();
    let (read, ask, send, note, keep) = match want {
        LocalWant::Read => (true, 0, 0, None, None),
        LocalWant::Ask => (false, 1, 0, None, None),
        LocalWant::Send | LocalWant::SendHeld | LocalWant::Pasted => (false, 0, 1, None, None),
        LocalWant::Connecting | LocalWant::Wait | LocalWant::Quiet | LocalWant::Taking => (false, 0, 0, None, None),
        LocalWant::Keep => (false, 0, 0, None, Some(Some(Secret::new(Box::new([7; 40]))))),
        LocalWant::Until => (false, 0, 0, Some(Note::Until(Wall::from_nanos(UNTIL))), None),
        LocalWant::Ended => (false, 1, 0, Some(Note::SignedOut), Some(None)),
    };
    if (duties.local, asks, sends, duties.note) != (read, ask, send, note) {
        failures.push(format!(
            "{name}: local/approval/sign-in/note expected {read}/{ask}/{send}/{note:?}, got {}/{asks}/{sends}/{:?}",
            duties.local, duties.note
        ));
    }
    if matches!(want, LocalWant::Taking)
        && !duties.asked.iter().any(|request| {
            matches!(request, Request::Exchange { message: Message::SignInApproval(_), credential: None, .. })
        })
    {
        failures.push(format!("{name}: approval take absent"));
    }
    if (keep.is_some() || matches!(want, LocalWant::Quiet | LocalWant::SendHeld)) && duties.keep != keep {
        failures.push(format!("{name}: credential duty"));
    }
    if matches!(want, LocalWant::Read | LocalWant::Wait | LocalWant::Connecting)
        && host.state().local() != Local::Signing
    {
        failures.push(format!("{name}: local signing screen"));
    }
    let frame = host.frame();
    let print = frame.print();
    let text = String::from_utf8(frame.text().into_vec()).expect("UTF-8 frame");
    if matches!(want, LocalWant::Connecting) && !text.contains("Connecting") {
        failures.push(format!("{name}: connecting text"));
    }
    if matches!(want, LocalWant::Read) && !host.state().front() && !text.contains("Connecting to 127.0.0.1:8367") {
        failures.push(format!("{name}: connecting text"));
    }
    if matches!(want, LocalWant::Wait)
        && host.deadline() != Some(host.env.now.saturating_add(skein_lib::Duration::from_secs(1)))
    {
        failures.push(format!("{name}: one-second reread deadline"));
    }
    if !matches!(want, LocalWant::Pasted) && (text.contains("Signing in…") || text.contains("This link was refused"))
    {
        failures.push(format!("{name}: file's sign-in or refusal drawn"));
    }
    if !matches!(want, LocalWant::Pasted) && host.state().signing() != &temper_client_tui::Signing::Idle {
        failures.push(format!("{name}: file's exchange told the paste UI"));
    }
    for secret in [LOCAL_SECRET, FRESH_SECRET] {
        if text.contains(secret) || print.windows(secret.len()).any(|part| part == secret.as_bytes()) {
            failures.push(format!("{name}: file's secret drawn"));
        }
    }
}

pub(crate) fn stream_bytes(event: &w::StreamEvent) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(usize::try_from(event.measure()).expect("stream size"));
    event.encode(&mut writer).expect("stream fits");
    writer.finish()
}
pub(crate) fn hello_bytes() -> Box<[u8]> {
    let limits = &w::CEILINGS;
    let mut served = List::with_capacity(1);
    served
        .push(
            w::Served::new(
                limits,
                w::ServedParts {
                    owner: w::Owner::Jig,
                    first: jig_client_protocol_wire::VERSION,
                    last: jig_client_protocol_wire::VERSION,
                },
            )
            .expect("Jig version"),
        )
        .expect("one owner");
    let hello = w::StreamEvent::Hello(
        w::Hello::new(limits, w::HelloParts { stream: [7; 16], served, heartbeat: Duration::from_secs(30) })
            .expect("hello"),
    );
    stream_bytes(&hello)
}
pub(crate) fn me_bytes(duties: &Duties) -> Box<[u8]> {
    let limits = &w::CEILINGS;
    let watch = duties
        .asked
        .iter()
        .find_map(|request| {
            if let Request::Exchange { message: Message::Watch(bytes), .. } = request {
                let open = w::Open::decode(limits, &mut Reader::new(bytes)).expect("watch request");
                if open.subject().owner() == &w::Owner::Jig
                    && w::Subject::decode(limits, &mut Reader::new(open.subject().record())).expect("Jig subject")
                        == w::Subject::Me
                {
                    return Some(open.watch());
                }
            }
            None
        })
        .expect("Me watch");
    let text = |bytes: &[u8]| {
        w::DocumentText::new(limits, w::DocumentTextParts { bytes: bytes.into(), cut: false }).expect("text")
    };
    let me = w::Me::new(
        limits,
        w::MeParts {
            party: w::MeParty::new(
                limits,
                w::MePartyParts { number: 7, kind: w::PartyKind::Person, login: text(b"owner"), name: text(b"Owner") },
            )
            .expect("party"),
            projects: Box::new([]),
            needs: w::Needs::new(limits, w::NeedsParts { questions: 0, proposals: 0, escalations: 0, person_tasks: 0 })
                .expect("needs"),
            needs_total: 0,
            updates: w::Count::new(limits, w::CountParts { value: 0, limited: false }).expect("count"),
            results_seen: 0,
            cut: false,
            retention: Duration::from_secs(1000),
            deployment_role: None,
            providers: Box::new([]),
            providers_cut: false,
            key_bytes: 256,
            redirect_bytes: 256,
            words_bytes: 8192,
        },
    )
    .expect("Me");
    let value = w::DeliveryValue::Me(me);
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("Me size"));
    value.encode(&mut writer).expect("Me fits");
    let document = w::Owned::new(
        limits,
        w::OwnedParts { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: writer.finish() },
    )
    .expect("Me document");
    let delivery =
        w::StreamEvent::Delivery(w::Delivery::new(limits, w::DeliveryParts { watch, document }).expect("delivery"));
    stream_bytes(&delivery)
}
fn connect(host: &mut Host) -> Duties {
    if host.root.client().link().credential.is_none() {
        return host.report(Report::Due);
    }
    let duties = host
        .report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::Record(hello_bytes()) }));
    host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::Record(me_bytes(&duties)) }))
}

fn pieces(items: &[Box<[u8]>]) -> Box<[u8]> {
    let size = items.iter().map(|item| item.len() + 4).sum();
    let mut writer = Writer::new(size);
    for item in items {
        skein_codec::piece::write(&mut writer, item).expect("fixture piece");
    }
    writer.finish()
}

pub(crate) fn launchpad_me(projects: bool) -> w::Me {
    let mut me = records::frame_record().into_parts();
    me.retention = Duration::from_secs(1000);
    if projects {
        let template = w::Template::new(
            &w::CEILINGS,
            w::TemplateParts {
                number: 5,
                name: records::wire_text(b"Ask temper"),
                executor: w::TemplateExecutor::Charter(
                    w::TemplateCharter::new(&w::CEILINGS, w::TemplateCharterParts { value: 5 }).expect("charter"),
                ),
                contract: w::TaskContractDocument::Report(
                    w::TaskContractDocumentReport::new(&w::CEILINGS, w::TaskContractDocumentReportParts { words: 32 })
                        .expect("report"),
                ),
                shallower: 1,
                budget: w::TemplateBudget::AsAllowed(
                    w::TemplateAsAllowed::new(&w::CEILINGS, w::TemplateAsAllowedParts { cap: None }).expect("budget"),
                ),
                deadline: w::TemplateDeadline::None,
                tracked: None,
                flags: w::TaskFlags::new(
                    &w::CEILINGS,
                    w::TaskFlagsParts {
                        ended_by: w::EndedBy::Result,
                        conversation: true,
                        listed: true,
                        retention: w::Retention::Kept,
                    },
                )
                .expect("flags"),
                label: 7,
                grace: None,
                llm: None,
                resources: List::with_capacity(0),
            },
        )
        .expect("template");
        let mut writer = Writer::new(usize::try_from(template.measure()).expect("template size"));
        template.encode(&mut writer).expect("template fits");
        let asking = writer.finish();
        let mut chat = template.into_parts();
        chat.number = jig_client::shapes::CHAT;
        chat.name = records::wire_text(b"Chat");
        chat.label = 8;
        let chat = w::Template::new(&w::CEILINGS, chat).expect("chat template");
        let mut writer = Writer::new(usize::try_from(chat.measure()).expect("chat template size"));
        chat.encode(&mut writer).expect("chat template fits");
        let templates = pieces(&[asking, writer.finish()]);
        let mut charters = List::with_capacity(1);
        charters.push(5).expect("one charter");
        let creating = w::CreatingBounds::new(
            &w::CEILINGS,
            w::CreatingBoundsParts {
                within: w::BoundsCreate::new(
                    &w::CEILINGS,
                    w::BoundsCreateParts {
                        authority: records::wire_owner_authority_document(),
                        pool_left: 100,
                        charters,
                        procedures: List::with_capacity(0),
                        latest: None,
                        budget: 100,
                        maximum_budget: 100,
                        maximum_deadline: None,
                    },
                )
                .expect("bounds"),
                beyond: w::Beyond::Refused(
                    w::BeyondRefused::new(&w::CEILINGS, w::BeyondRefusedParts { value: w::Refusal::NotOffered })
                        .expect("beyond"),
                ),
            },
        )
        .expect("creating");
        let mut projects = Vec::new();
        for project in [3, 5] {
            let project = w::MeProject::new(
                &w::CEILINGS,
                w::MeProjectParts {
                    left_out: 0,
                    project,
                    name: records::wire_text(b"Project"),
                    role: w::HeldRole::Owner,
                    pool: None,
                    templates: templates.clone(),
                    creating: Some(creating.clone()),
                    models: List::with_capacity(0),
                    defaults: List::with_capacity(0),
                    public: None,
                },
            )
            .expect("project");
            projects.push(project_bytes(&project));
        }
        me.projects = pieces(&projects);
    }
    w::Me::new(&w::CEILINGS, me).expect("Me")
}

fn project_bytes(project: &w::MeProject) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(project.measure()).expect("project size"));
    project.encode(&mut writer).expect("project fits");
    writer.finish()
}

pub(crate) fn watch(duties: &Duties, subject: &w::Subject) -> u64 {
    duties
        .asked
        .iter()
        .rev()
        .find_map(|request| {
            if let Request::Exchange { message: Message::Watch(bytes), .. } = request {
                let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("watch");
                if open.subject().owner() == &w::Owner::Jig
                    && w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject().record())).expect("subject")
                        == *subject
                {
                    return Some(open.watch());
                }
            }
            None
        })
        .unwrap_or_else(|| panic!("requested watch {subject:?}: {:?}", duties.asked))
}

fn delivery(host: &mut Host, watch: u64, value: &w::DeliveryValue) -> Duties {
    host.report(delivery_report(watch, value))
}

pub(crate) fn delivery_report(watch: u64, value: &w::DeliveryValue) -> Report {
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("delivery size"));
    value.encode(&mut writer).expect("delivery fits");
    let document = w::Owned::new(
        &w::CEILINGS,
        w::OwnedParts { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: writer.finish() },
    )
    .expect("document");
    let event = w::StreamEvent::Delivery(
        w::Delivery::new(&w::CEILINGS, w::DeliveryParts { watch, document }).expect("delivery"),
    );
    Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::Record(stream_bytes(&event)) })
}

fn signed_host(projects: bool) -> (Host, u64, Duties) {
    signed_host_look(projects, None, false)
}

fn signed_host_look(projects: bool, background: Option<Rgb>, boxed: bool) -> (Host, u64, Duties) {
    let mut host = host();
    start_look(&mut host, Some(Secret::new(Box::new([7; 40]))), background, boxed);
    let hello = hello_for(&host);
    let duties =
        host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::Record(hello) }));
    let duties = admit(&mut host, duties);
    let me = watch(&duties, &w::Subject::Me);
    let duties = delivery(&mut host, me, &w::DeliveryValue::Me(launchpad_me(projects)));
    let duties = admit(&mut host, duties);
    (host, me, duties)
}

const NEWEST: u64 = 9;
const OLDER: u64 = 8;
const BACKGROUND: Rgb = Rgb { red: 0x19, green: 0x18, blue: 0x16 };
const BAND: Rgb = Rgb { red: 0x2d, green: 0x2c, blue: 0x2a };
const DIM: Rgb = Rgb { red: 0x9a, green: 0x93, blue: 0x88 };

fn chat_chip(task: u64, title: &[u8]) -> w::Chip {
    chat_chip_stage(task, title, w::RunStage::Idle)
}

fn chat_chip_stage(task: u64, title: &[u8], stage: w::RunStage) -> w::Chip {
    let working = stage == w::RunStage::Due;
    let chip = records::owner_chip_record(
        w::ChipPhase::Active(
            w::ChipPhaseActive::new(&w::CEILINGS, w::ChipPhaseActiveParts { value: stage }).expect("stage"),
        ),
        if working { w::Phase::Due } else { w::Phase::Idle },
    );
    let mut reader = Reader::new(chip.rows());
    let mut rows = Vec::new();
    while let Some(bytes) = skein_codec::piece::split(&mut reader).expect("chip piece") {
        let mut fact = w::Fact::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("chip fact");
        if let w::Fact::Chip(row) = fact {
            let mut row = row.into_parts();
            let mut fields = row.value.into_parts();
            fields.task = task;
            fields.title = records::wire_text(title);
            row.value = w::ChipFields::new(&w::CEILINGS, fields).expect("chat chip");
            fact = w::Fact::Chip(w::ChipRow::new(&w::CEILINGS, row).expect("chat row"));
        } else if working && let w::Fact::ChipDetails(row) = fact {
            let mut row = row.into_parts();
            let mut fields = row.value.into_parts();
            fields.live_delegates = 0;
            row.value = w::ChipDetailsFields::new(&w::CEILINGS, fields).expect("working details");
            fact = w::Fact::ChipDetails(Box::new(w::ChipDetailsRow::new(&w::CEILINGS, row).expect("details row")));
        }
        rows.push(fact);
    }
    records::chip_from_rows(&rows)
}

fn left_off_host() -> Host {
    home_host(false)
}

fn person_chat(host: &mut Host) {
    let duties = give(host, Event::Act { act: Act::Go { address: Address::Chat { task: NEWEST } } });
    let task_watch = watch(
        &duties,
        &w::Subject::Task(
            w::SubjectTask::new(&w::CEILINGS, w::SubjectTaskParts { task: NEWEST }).expect("task subject"),
        ),
    );
    let activity = read_exchange(&duties, "activity");
    let mut task = records::task_record().into_parts();
    task.project = 3;
    task.chip = chat_chip(NEWEST, b"Newest");
    task.activity = 1;
    task.result = None;
    delivery(host, task_watch, &w::DeliveryValue::Task(Box::new(w::Task::new(&w::CEILINGS, task).expect("chat"))));
    let source = w::ActivityLine::new(
        &w::CEILINGS,
        w::ActivityLineParts {
            task: NEWEST,
            position: 1,
            at: 0,
            line: w::Activity::Words(
                w::ActivityWords::new(
                    &w::CEILINGS,
                    w::ActivityWordsParts {
                        message: 1,
                        by: w::Requester::Person(
                            w::RequesterPerson::new(&w::CEILINGS, w::RequesterPersonParts { value: 7 })
                                .expect("person"),
                        ),
                        kind: w::MessageKind::Words,
                        words: b"Person words".as_slice().into(),
                    },
                )
                .expect("words"),
            ),
        },
    )
    .expect("activity");
    let row =
        w::DecodedActivityLine::new(&w::CEILINGS, w::DecodedActivityLineParts { source, decoded: None }).expect("line");
    read_reply(host, activity, &w::ReadValue::Activity(records::activity_rows(&[row])));
}

fn give(host: &mut Host, event: Event) -> Duties {
    let mut duties = Duties::default();
    host.event(event, &mut duties);
    let more = host.report(Report::Due);
    duties.asked.extend(more.asked);
    duties.close |= more.close;
    stored(host, duties)
}

fn stored(host: &mut Host, mut duties: Duties) -> Duties {
    let mut pending = std::collections::VecDeque::from(core::mem::take(&mut duties.asked));
    while let Some(request) = pending.pop_front() {
        if matches!(&request, Request::Save { .. } | Request::Erase { .. }) {
            pending.extend(host.report(Report::Heard(jig_client::Event::Stored)).asked);
        }
        duties.asked.push(request);
    }
    duties
}

fn typed(host: &mut Host, words: &[u8]) {
    give(host, Event::Typed { value: words.into() });
}

fn key_words(host: &mut Host, words: &str) {
    let mut echoed = String::from_utf8(host.words().to_vec()).expect("UTF-8 composer");
    for character in words.chars() {
        host.frame();
        empty(&host.report(Report::Key(Key::Char(character))));
        echoed.push(character);
        assert!(text(host).contains(&format!("❯ {echoed}")), "composer must echo {echoed:?}");
    }
}

#[test]
fn no_project_composer_accepts_commands() {
    let (mut host, _, _) = signed_host(false);
    key_words(&mut host, "/");
    let frame = text(&mut host);
    for command in ["/theme", "/help", "/exit", "Close temper"] {
        assert!(frame.contains(command), "offered command list: {command}\n{frame}");
    }
    key_words(&mut host, "exit");
    assert!(text(&mut host).contains("❯ /exit"));
    assert!(host.report(Report::Key(Key::Enter)).close, "/exit must close from the launchpad");
    assert!(host.words().is_empty(), "closing clears the command composer");
    for words in ["/theme", "/help", "/zz", "hello", "q", "/"] {
        no_project_keys(words);
    }
    no_project_lifecycle();
}

fn no_project_lifecycle() {
    for signing_out in [false, true] {
        let (mut host, me, _) = signed_host(false);
        let revision = host.state().revision();
        let written = host.state().box_written();
        key_words(&mut host, "/he");
        assert!(host.state().revision() > revision, "typing redraws the TUI-owned composer");
        assert_eq!(host.state().box_written(), written, "typing preserves the editor's write count");
        assert!(host.state().field().is_empty(), "commands stay separate from sign-in text");
        let revision = host.state().revision();
        empty(&host.report(Report::Due));
        assert_eq!(host.state().revision(), revision, "an unchanged live link preserves the composer");
        assert_eq!(host.words(), b"/he");
        if signing_out {
            host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::SignedOut }));
            host.frame();
            empty(&host.report(Report::Key(Key::Char('q'))));
            assert_eq!(host.state().field(), b"q", "sign-in typing uses its own field");
        } else {
            delivery(&mut host, me, &w::DeliveryValue::Me(launchpad_me(true)));
            assert_eq!(host.root.pages().address(), &Address::Home { project: 3 });
            assert!(host.words().is_empty(), "navigation opens an empty project composer");
            assert!(!text(&mut host).contains("❯ /he"));
        }
        assert!(host.state().command_words().is_empty(), "screen transitions clear command words");
        assert!(host.state().box_written() > written);
    }
}

fn no_project_keys(words: &str) {
    let (mut host, _, _) = signed_host(false);
    key_words(&mut host, words);
    let written = host.state().box_written();
    let duties = host.report(Report::Key(if words == "/" { Key::Escape } else { Key::Enter }));
    empty(&duties);
    match words {
        "/theme" => assert_eq!(host.root.pages().look().theme, Theme::Light),
        "/help" => {
            assert_eq!(host.root.pages().notes().last(), Some(&terminal::Note::Help));
            assert!(text(&mut host).contains("/exit"), "help must render its commands");
        }
        "/zz" => {
            assert_eq!(host.state().unknown(), Some(b"/zz".as_slice()));
            assert!(text(&mut host).contains("Unknown command /zz."));
        }
        "hello" | "q" => {
            assert_eq!(host.words(), words.as_bytes(), "ordinary words send nothing");
            assert_eq!(host.state().box_written(), written);
        }
        "/" => assert!(!text(&mut host).contains("Close temper"), "Escape dismisses the list"),
        _ => unreachable!("named composer case"),
    }
    if words.starts_with('/') {
        assert!(host.words().is_empty(), "command execution and Escape clear the composer");
        assert!(host.state().box_written() > written);
        assert!(text(&mut host).contains("❯ / for commands"));
        key_words(&mut host, "/he");
        host.frame();
        empty(&host.report(Report::Key(Key::Tab)));
        assert_eq!(host.words(), b"/help", "Tab completes into the same composer");
        assert!(text(&mut host).contains("❯ /help"));
    }
    host.frame();
    assert!(host.report(Report::Key(Key::Control(b'c'))).close, "Ctrl+C closes anywhere");
}

fn sent_ask(duties: &Duties) -> Vec<(skein_lib::Token, w::Ask)> {
    duties
        .asked
        .iter()
        .filter_map(|request| {
            if let Request::Exchange { exchange, message: Message::Request(bytes), .. } = request {
                let request = w::Keyed::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("request");
                if request.owner() == &w::Owner::Jig {
                    return Some((
                        *exchange,
                        w::Ask::decode(&w::CEILINGS, &mut Reader::new(request.request())).expect("ask"),
                    ));
                }
            }
            None
        })
        .collect()
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table holds the host's look, navigation and closing rules")]
fn the_host_goes_to_the_launchpad_and_follows_the_root() {
    let mut failures = Vec::new();
    for row in [
        "first project once",
        "project appears",
        "project lost",
        "dark look",
        "light look",
        "theme",
        "band",
        "dim words",
        "boxed",
        "unboxed",
        "created chat",
        "show chat",
        "show ask",
        "show without project",
        "new",
        "exit",
    ] {
        let (mut host, me, _) =
            signed_host_look(row != "project appears", (row == "band").then_some(BACKGROUND), row == "boxed");
        let passed = match row {
            "first project once" => {
                let before = host.state().focus();
                delivery(&mut host, me, &w::DeliveryValue::Me(launchpad_me(true)));
                host.root.pages().address() == &Address::Home { project: 3 } && host.state().focus() == before
            }
            "project appears" => {
                let stayed = host.root.pages().address() == &Address::Inbox { filter: root::address::Filter::ALL };
                delivery(&mut host, me, &w::DeliveryValue::Me(launchpad_me(true)));
                stayed && host.root.pages().address() == &Address::Home { project: 3 }
            }
            "project lost" => {
                delivery(&mut host, me, &w::DeliveryValue::Me(launchpad_me(false)));
                let inbox = host.root.pages().address() == &Address::Inbox { filter: root::address::Filter::ALL };
                let mut next = launchpad_me(true).into_parts();
                let mut reader = Reader::new(&next.projects);
                let _ = skein_codec::piece::split(&mut reader).expect("first project");
                let project = skein_codec::piece::split(&mut reader).expect("second project").expect("project");
                next.projects = pieces(&[project]);
                delivery(&mut host, me, &w::DeliveryValue::Me(w::Me::new(&w::CEILINGS, next).expect("next me")));
                inbox && host.root.pages().address() == &Address::Home { project: 5 }
            }
            "dark look" | "light look" => {
                let mut host = self::host();
                let light = row == "light look";
                host.report(Report::Start(Start {
                    front: false,
                    size: Size { columns: 80, rows: 24 },
                    scheme: if light { Scheme::Light } else { Scheme::Dark },
                    background: None,
                    boxed: false,
                    address: b"127.0.0.1:8367".as_slice().into(),
                    local: false,
                    kept: vec![],
                    credential: None,
                    note: None,
                }));
                let look = host.root.pages().look();
                look.terminal
                    && look.theme == if light { Theme::Light } else { Theme::Dark }
                    && look.font == Font::DejaVu
                    && !look.night
            }
            "theme" => {
                typed(&mut host, b"/theme");
                give(&mut host, Event::Send);
                let palette = temper_client_tui::theme::palette(Scheme::Light);
                let light = host.root.pages().look().theme == Theme::Light;
                let frame = host.frame();
                light
                    && (0..frame.size().rows)
                        .filter_map(|row| frame.row(row))
                        .flatten()
                        .any(|cell| palette.iter().any(|(_, colour)| cell.paint.fore == Some(*colour)))
            }
            "band" | "dim words" => {
                person_chat(&mut host);
                let frame = host.frame();
                (0..frame.size().rows).filter_map(|row| frame.row(row)).any(|cells| {
                    cells.windows(b"Person words".len()).any(|words| {
                        words.iter().map(|cell| cell.glyph).eq("Person words".chars())
                            && words.iter().all(|cell| {
                                cell.paint.back == (row == "band").then_some(BAND)
                                    && cell.paint.fore == (row == "dim words").then_some(DIM)
                            })
                    })
                })
            }
            "boxed" | "unboxed" => host.state().boxed() == (row == "boxed"),
            "created chat" => {
                typed(&mut host, b"Please help");
                let duties = give(&mut host, Event::Send);
                let asks = sent_ask(&duties);
                if let Some((exchange, _)) = asks.first() {
                    let before = host.state().focus();
                    let duties = reply(&mut host, *exchange, created_bytes(9));
                    host.root.pages().address() == &Address::Chat { task: 9 }
                        && host.state().focus() > before
                        && duties
                            .asked
                            .iter()
                            .any(|request| matches!(request, Request::Exchange { message: Message::Watch(_), .. }))
                } else {
                    false
                }
            }
            "show without project" => {
                let (mut host, _, _) = signed_host(false);
                let before = host.state().focus();
                host.out.push(root::Request::Show { address: Address::Accounts, replace: false });
                host.report(Report::Due);
                host.root.pages().address() == &Address::Inbox { filter: root::address::Filter::ALL }
                    && host.state().focus() == before
            }
            "show chat" | "show ask" => {
                let address = if row == "show chat" { Address::Chat { task: 9 } } else { Address::Ask { project: 5 } };
                host.out.push(root::Request::Show { address: address.clone(), replace: true });
                let before = host.state().focus();
                host.report(Report::Due);
                host.root.pages().address() == &if row == "show chat" { address } else { Address::Home { project: 5 } }
                    && host.state().focus() > before
            }
            "new" => {
                host.step(root::Event::Went { place: Place::Address(Address::Chat { task: 9 }) });
                host.report(Report::Due);
                give(&mut host, Event::Command { command: Command::New });
                host.root.pages().address() == &Address::Home { project: 3 }
            }
            "exit" => {
                let duties = give(&mut host, Event::Command { command: Command::Exit });
                duties.close && duties.asked.is_empty() && host.root.pages().look().terminal
            }
            _ => unreachable!("named row"),
        };
        if !passed {
            failures.push(row);
        }
    }
    assert!(failures.is_empty(), "host rows: {failures:?}");
}

#[derive(Clone, Copy)]
enum LaunchAt {
    Home,
    Aside,
    AsideByKey,
    Need,
    NewChat,
}

#[derive(Debug)]
enum LaunchWant {
    Started { label: u32, words: &'static [u8], from: Address },
    Words(&'static [u8]),
    Chat(u64),
    Unknown,
    Brief,
    Reply,
    Walk,
    Opened,
    New,
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks launchpad commands and refused starts")]
fn the_launchpad_takes_its_own_words() {
    for (words, at, want) in [
        (
            b"/change Add a CSV export".as_slice(),
            LaunchAt::Home,
            LaunchWant::Started { label: 8, words: b"Add a CSV export", from: Address::NewChat { draft: false } },
        ),
        (b"/change", LaunchAt::Home, LaunchWant::Words(b"/change ")),
        (b"/c", LaunchAt::Home, LaunchWant::Words(b"/change ")),
        (b"/", LaunchAt::Home, LaunchWant::Words(b"/change ")),
        (b"/ask", LaunchAt::Home, LaunchWant::Words(b"/ask ")),
        (b"/a", LaunchAt::Home, LaunchWant::Words(b"/ask ")),
        (b"/n", LaunchAt::Need, LaunchWant::New),
        (b"/needs", LaunchAt::Need, LaunchWant::Walk),
        (b"/open", LaunchAt::Home, LaunchWant::Opened),
        (
            b"/ask What landed?",
            LaunchAt::Home,
            LaunchWant::Started { label: 7, words: b"What landed?", from: Address::Home { project: 3 } },
        ),
        (
            b"/ask What landed?",
            LaunchAt::NewChat,
            LaunchWant::Started { label: 7, words: b"What landed?", from: Address::Home { project: 3 } },
        ),
        (b"/chats 1", LaunchAt::Home, LaunchWant::Chat(NEWEST)),
        (b"/chats 2", LaunchAt::Home, LaunchWant::Unknown),
        (b"/br", LaunchAt::Home, LaunchWant::Brief),
        (b"Use a comma", LaunchAt::Aside, LaunchWant::Reply),
        (b"Use a comma", LaunchAt::AsideByKey, LaunchWant::Reply),
        (
            b"Please help",
            LaunchAt::Home,
            LaunchWant::Started { label: 7, words: b"Please help", from: Address::Home { project: 3 } },
        ),
    ] {
        let aside = matches!(at, LaunchAt::Aside | LaunchAt::AsideByKey);
        let mut host = if matches!(at, LaunchAt::Aside | LaunchAt::AsideByKey | LaunchAt::Need) {
            waiting_at(false)
        } else {
            one_chat_host(matches!(&want, LaunchWant::Brief))
        };
        if matches!(at, LaunchAt::NewChat) {
            host.went(Address::NewChat { draft: false });
            let duties = host.report(Report::Due);
            admit(&mut host, duties);
        }
        let object = aside.then(|| shared_question(&host));
        if let Some(object) = object {
            host.tell(Told::Walk { by_key: matches!(at, LaunchAt::AsideByKey) });
            give(&mut host, Event::Aside { object });
        }
        typed(&mut host, words);
        let duties = give(&mut host, Event::Send);
        match want {
            LaunchWant::Started { label, words: expected, from } => {
                assert_eq!(host.root.pages().address(), &from, "{words:?}");
                let duties = admit(&mut host, duties);
                let asks = sent_ask(&duties);
                let [(exchange, w::Ask::Create(create))] = asks.as_slice() else { panic!("{words:?}: {asks:?}") };
                let mut reader = Reader::new(create.tasks());
                let bytes = skein_codec::piece::split(&mut reader).expect("task piece").expect("created task");
                let task = w::NewTask::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("new task");
                assert_eq!((create.project(), task.label(), task.spec().words()), (3, label, expected), "{words:?}");
                reply(&mut host, *exchange, created_bytes(NEWEST));
                assert_eq!(host.root.pages().address(), &Address::Chat { task: NEWEST }, "started chat opened");
                assert!(host.words().is_empty(), "started chat box cleared");
            }
            LaunchWant::Words(expected) => assert_eq!(host.words(), expected, "{words:?}"),
            LaunchWant::Chat(task) => assert_eq!(host.root.pages().address(), &Address::Chat { task }, "{words:?}"),
            LaunchWant::Unknown => assert_eq!(host.state().unknown(), Some(words), "{words:?}"),
            LaunchWant::Brief => assert!(host.state().brief_shown(), "{words:?}"),
            LaunchWant::Walk => {
                assert!(host.state().walk().is_some_and(|walk| !walk.by_key()), "{words:?}");
                assert!(host.words().is_empty());
            }
            LaunchWant::Opened => {
                assert_eq!(duties.open.as_deref(), Some(b"http://127.0.0.1:8367/projects/3".as_slice()));
                assert!(host.words().is_empty());
            }
            LaunchWant::New => {
                assert_eq!(host.root.pages().address(), &Address::Home { project: 3 });
                assert!(host.state().walk().is_none(), "/n runs /new before /needs");
            }
            LaunchWant::Reply => {
                let object = object.expect("aside question");
                let answered =
                    host.root.pages().needs().answered(Wall::EPOCH, root::pages::needs::ANSWERED_ON_HOME, Some(3));
                assert!(
                    answered.iter().any(|line| line.object == object
                        && line.label == root::pages::needs::Label::Reply(b"Use a comma".as_slice().into())),
                    "reply words reach card"
                );
                assert!(host.state().walk().is_some_and(|walk| walk.shown(&object)), "answered need marked shown");
                assert!(host.words().is_empty());
            }
        }
    }
    for missing_template in [true, false] {
        let (mut host, me_watch, _) = signed_host(true);
        if missing_template {
            let mut me = launchpad_me(true).into_parts();
            let mut projects = Vec::new();
            let mut reader = Reader::new(&me.projects);
            while let Some(bytes) = skein_codec::piece::split(&mut reader).expect("project piece") {
                let mut project =
                    w::MeProject::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("project").into_parts();
                project.templates = Box::default();
                let project = w::MeProject::new(&w::CEILINGS, project).expect("project without templates");
                projects.push(project_bytes(&project));
            }
            me.projects = pieces(&projects);
            delivery(&mut host, me_watch, &w::DeliveryValue::Me(w::Me::new(&w::CEILINGS, me).expect("no template")));
        }
        typed(&mut host, b"/change Add a CSV export");
        let duties = give(&mut host, Event::Send);
        if !missing_template {
            let asks = sent_ask(&duties);
            let [(exchange, _)] = asks.as_slice() else { panic!("one change start: {asks:?}") };
            let answer = w::Final::Refused(
                w::Refused::new(&w::CEILINGS, w::RefusedParts { refusal: w::Refusal::NotOffered, at: None })
                    .expect("refusal"),
            );
            reply(&mut host, *exchange, final_bytes(&answer));
        }
        assert_eq!(host.root.pages().address(), &Address::NewChat { draft: false }, "refusal stays with its composer");
        assert_eq!(host.words(), b"Add a CSV export", "refused words retained");
        let refusal = temper_client_view::shared::refusal(&host.root, host.view.words().view());
        assert!(refusal.is_some(), "missing template={missing_template}: refusal drawn under box");
        host.went(Address::Home { project: 3 });
        assert!(
            host.root.pages().chats().is_some_and(|chats| chats.refusal.is_none() && chats.outcome.is_none()),
            "leaving NewChat drops refusal"
        );
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table holds the host's TUI event translations")]
fn each_event_becomes_its_acts() {
    let page = b"http://127.0.0.1:8367/projects/3".as_slice();
    for (event, words, brief, walking, open) in [
        (Event::Own { command: Own::Change }, b"/change ".as_slice(), false, false, None),
        (Event::Own { command: Own::Ask }, b"/ask ".as_slice(), false, false, None),
        (Event::Own { command: Own::Brief }, b"".as_slice(), true, false, None),
        (Event::Own { command: Own::Needs }, b"".as_slice(), false, true, None),
        (Event::Own { command: Own::Open }, b"".as_slice(), false, false, Some(page)),
        (Event::Open { address: page.into() }, b"".as_slice(), false, true, Some(page)),
        (Event::EndWalk, b"".as_slice(), false, false, None),
    ] {
        let (mut host, _, _) = signed_host(true);
        host.tell(Told::Walk { by_key: true });
        let before = host.state().box_written();
        let mut duties = Duties::default();
        host.event(event.clone(), &mut duties);
        assert_eq!(host.words(), words, "{event:?}");
        assert!(words.is_empty() || host.state().box_written() > before, "{event:?}: box write count");
        assert_eq!(host.state().brief_shown(), brief, "{event:?}");
        assert_eq!(host.state().walk().is_some(), walking, "{event:?}");
        assert_eq!(duties.open.as_deref(), open, "{event:?}");
        assert!(duties.asked.is_empty(), "{event:?}");
    }
    for (projects, chat, selected, address) in [
        (false, false, false, None),
        (true, false, false, Some(b"http://127.0.0.1:8367/projects/3".as_slice())),
        (true, true, false, Some(b"http://127.0.0.1:8367/chats/9".as_slice())),
        (true, false, true, Some(b"http://127.0.0.1:8367/chats/9".as_slice())),
    ] {
        let mut host = if projects { left_off_host() } else { signed_host(false).0 };
        if chat {
            give(&mut host, Event::Act { act: Act::Go { address: Address::Chat { task: NEWEST } } });
        }
        if selected {
            host.report(Report::Key(Key::Down));
        }
        assert_eq!(give(&mut host, Event::Own { command: Own::Open }).open.as_deref(), address);
    }
    for (chat, words, after, event, walking, expected) in [
        (false, b"".as_slice(), None, Event::Own { command: Own::Needs }, true, b"".as_slice()),
        (false, b"a".as_slice(), None, Event::Typed { value: b"an".as_slice().into() }, false, b"an".as_slice()),
        (true, b"".as_slice(), None, Event::Typed { value: b"n".as_slice().into() }, false, b"n".as_slice()),
        (false, b"".as_slice(), Some('o'), Event::Own { command: Own::Needs }, false, b"no".as_slice()),
    ] {
        let mut host = waiting_at(chat);
        typed(&mut host, words);
        host.frame();
        assert_eq!(host.event_of(&Key::Char('n')), Some(event.clone()), "chat={chat}, words={words:?}");
        host.report(Report::Key(Key::Char('n')));
        if let Some(character) = after {
            host.frame();
            host.report(Report::Key(Key::Char(character)));
        }
        assert_eq!(host.words(), expected, "chat={chat}, words={words:?}, after={after:?}");
        assert_eq!(host.state().walk().is_some_and(|walk| walk.by_key() && !walk.taken()), walking);
    }
    for (by_key, taken, expected) in
        [(false, false, b"o".as_slice()), (true, true, b"o".as_slice()), (true, false, b"no".as_slice())]
    {
        let mut host = waiting_at(false);
        host.tell(Told::Walk { by_key });
        if taken {
            host.tell(Told::Walked { object: shared_question(&host) });
        }
        typed(&mut host, b"o");
        assert_eq!(host.words(), expected, "by_key={by_key}, taken={taken}");
        assert!(host.state().walk().is_none(), "typing ends the prompt");
    }
    for (step, label, open) in [
        (
            WalkStep::Choose { words: b"Comma".as_slice().into() },
            Some(root::pages::needs::Label::Choice(b"Comma".as_slice().into())),
            None,
        ),
        (WalkStep::Open { address: page.into() }, None, Some(page)),
        (WalkStep::Next, None, None),
    ] {
        let mut host = waiting_host();
        let object = shared_question(&host);
        host.tell(Told::Walk { by_key: false });
        let duties = give(&mut host, Event::Walk { object, step: step.clone() });
        let answered = host.root.pages().needs().answered(Wall::EPOCH, root::pages::needs::ANSWERED_ON_HOME, Some(3));
        assert_eq!(
            answered.iter().find(|line| line.object == object).map(|line| &line.label),
            label.as_ref(),
            "{step:?}: only a choice answers, in its words"
        );
        assert_eq!(duties.open.as_deref(), open, "{step:?}: page opened");
        assert!(host.state().walk().is_some_and(|walk| walk.shown(&object) && walk.taken()), "{step:?}: need shown");
    }
    for (own, brief, walk) in [(Own::Brief, true, false), (Own::Needs, false, true), (Own::Open, false, false)] {
        let mut host = waiting_at(false);
        give(&mut host, Event::Command { command: Command::Model });
        typed(&mut host, own.name());
        let before = host.state().box_written();
        give(&mut host, Event::Own { command: own });
        assert!(host.words().is_empty(), "{own:?}: command box cleared");
        assert!(host.root.pages().listing().is_none(), "{own:?}: standing list dismissed");
        assert!(host.state().box_written() > before, "{own:?}: box write advances");
        assert_eq!(host.state().brief_shown(), brief, "{own:?}");
        assert_eq!(host.state().walk().is_some_and(|walk| !walk.by_key()), walk, "{own:?}");
    }
    let mut failures = Vec::new();
    for row in [
        "stop",
        "stay",
        "front log",
        "bare log",
        "home words",
        "left off arrows",
        "left off enter",
        "left off escape",
        "left off typed",
        "left off commands",
        "chat words",
        "chat send",
        "create",
        "unknown",
        "unknown send",
        "unknown went",
        "no project",
        "help",
        "dismiss",
        "escape",
        "aside",
        "reply words",
        "reply choice",
        "act",
        "size",
    ] {
        let (mut host, _, _) = signed_host(row != "no project");
        let passed = match row {
            "left off arrows" => {
                let mut host = left_off_host();
                let mut passed = true;
                for (key, task, title) in
                    [(Key::Down, NEWEST, "Newest"), (Key::Down, OLDER, "Older"), (Key::Up, NEWEST, "Newest")]
                {
                    passed &= host.event_of(&key) == Some(Event::LeftOff { task: Some(task) });
                    host.report(Report::Key(key));
                    passed &= host.state().left_off() == Some(task)
                        && text(&mut host)
                            .lines()
                            .any(|line| line.starts_with(&format!(" ❯ Project   {title}, a chat")));
                }
                passed
            }
            "left off enter" => {
                let mut host = left_off_host();
                let event = host.event_of(&Key::Down).expect("first chat");
                give(&mut host, event);
                host.frame();
                let selected = host.state().left_off() == Some(NEWEST)
                    && host.event_of(&Key::Enter)
                        == Some(Event::Act { act: Act::Go { address: Address::Chat { task: NEWEST } } });
                let duties = host.report(Report::Key(Key::Enter));
                read_exchange(&duties, "activity");
                watch(
                    &duties,
                    &w::Subject::Task(
                        w::SubjectTask::new(&w::CEILINGS, w::SubjectTaskParts { task: NEWEST }).expect("task subject"),
                    ),
                );
                selected && host.root.pages().address() == &Address::Chat { task: NEWEST }
            }
            "left off escape" | "left off typed" | "left off commands" => {
                let mut host = left_off_host();
                host.report(Report::Key(Key::Down));
                host.frame();
                if row == "left off escape" {
                    let event = host.event_of(&Key::Escape);
                    host.report(Report::Key(Key::Escape));
                    event == Some(Event::LeftOff { task: None }) && host.state().left_off().is_none()
                } else if row == "left off typed" {
                    host.report(Report::Key(Key::Char('a')));
                    host.state().left_off().is_none() && host.event_of(&Key::Down).is_none()
                } else {
                    host.report(Report::Key(Key::Char('/')));
                    host.frame();
                    let event = host.event_of(&Key::Down);
                    host.report(Report::Key(Key::Down));
                    host.state().left_off().is_none()
                        && event.is_none()
                        && text(&mut host).lines().any(|line| line.starts_with("❯ /ask"))
                }
            }
            "stop" => give(&mut host, Event::Stop).close,
            "stay" => {
                host.tell(temper_client_tui::Told::Quitting);
                !give(&mut host, Event::Stay).close && !host.state().quitting()
            }
            "front log" | "bare log" => {
                let front = row == "front log";
                if front {
                    host.tell(temper_client_tui::Told::Front);
                }
                typed(&mut host, b"/log");
                let before = host.state().box_written();
                let duties = give(&mut host, Event::Log);
                duties.asked.is_empty()
                    && host.state().log_shown() == front
                    && host.root.pages().chats().is_some_and(|chats| chats.words.is_empty() == front)
                    && (host.state().box_written() > before) == front
            }
            "home words" | "create" => {
                typed(&mut host, b"Please help");
                if row == "home words" {
                    host.root.pages().chats().is_some_and(|chats| chats.words.as_ref() == b"Please help")
                } else {
                    let duties = give(&mut host, Event::Send);
                    let asks = sent_ask(&duties);
                    asks.len() == 1
                        && matches!(&asks[0].1, w::Ask::Create(create) if create.project() == 3 && one_asking_task(create.tasks()))
                        && host.state().box_written() > 0
                }
            }
            "chat words" => {
                host.step(root::Event::Went { place: Place::Address(Address::Chat { task: 9 }) });
                host.report(Report::Due);
                typed(&mut host, b"Chat words");
                host.root.pages().chat().is_some_and(|chat| chat.words.as_ref() == b"Chat words")
            }
            "chat send" => {
                let mut host = waiting_host();
                let object = temper_client_view::shared::question(&host.root, host.view.words().view())
                    .expect("question")
                    .object;
                give(
                    &mut host,
                    Event::Act {
                        act: Act::Needs(root::pages::needs::Act::Choose { object, words: b"Comma".as_slice().into() }),
                    },
                );
                typed(&mut host, b"Continue");
                let before = host.state().box_written();
                let duties = give(&mut host, Event::Send);
                let asks = sent_ask(&duties);
                asks.len() == 1
                    && matches!(&asks[0].1, w::Ask::Message(message) if message.task() == 7 && message.words() == b"Continue")
                    && host.state().box_written() > before
            }
            "unknown" => {
                typed(&mut host, b"/zz");
                let before = host.state().box_written();
                let duties = give(&mut host, Event::Send);
                duties.asked.is_empty()
                    && host.state().unknown() == Some(b"/zz".as_slice())
                    && host.root.pages().chats().is_some_and(|chat| chat.words.is_empty())
                    && host.state().box_written() > before
            }
            "unknown send" | "unknown went" => {
                host.tell(temper_client_tui::Told::Unknown { words: Some(b"/zz".as_slice().into()) });
                if row == "unknown send" {
                    give(&mut host, Event::Send);
                } else {
                    host.out.push(root::Request::Show { address: Address::Chat { task: 9 }, replace: false });
                    host.report(Report::Due);
                }
                host.state().unknown().is_none()
            }
            "no project" => {
                typed(&mut host, b"Please help");
                let before = host.state().box_written();
                give(&mut host, Event::Send).asked.is_empty() && host.state().box_written() == before
            }
            "help" => {
                typed(&mut host, b"/help");
                let before = host.state().box_written();
                give(&mut host, Event::Command { command: Command::Help });
                host.root.pages().notes().last() == Some(&terminal::Note::Help)
                    && host.root.pages().chats().is_some_and(|chats| chats.words.is_empty())
                    && host.state().box_written() > before
            }
            "dismiss" => {
                typed(&mut host, b"/");
                let before = host.state().box_written();
                give(&mut host, Event::Dismiss);
                host.root.pages().chats().is_some_and(|chats| chats.words.is_empty())
                    && host.state().box_written() > before
            }
            "escape" => {
                typed(&mut host, b"/");
                host.frame();
                let expected = temper_client_tui::escape(&host.view);
                let event = host.event_of(&Key::Escape);
                let mut direct = host;
                let (mut keyed, _, _) = signed_host(true);
                typed(&mut keyed, b"/");
                keyed.frame();
                if let Some(event) = expected.clone() {
                    give(&mut direct, event);
                } else {
                    direct.report(Report::Due);
                }
                keyed.report(Report::Key(Key::Escape));
                event == expected
                    && keyed.root.pages().chats().map(|chats| &chats.words)
                        == direct.root.pages().chats().map(|chats| &chats.words)
                    && keyed.state().box_written() == direct.state().box_written()
                    && keyed.state().aside() == direct.state().aside()
            }
            "aside" => {
                let object = jig_client::contract::ObjectKey::Task(9);
                host.event(Event::Aside { object }, &mut Duties::default());
                let aside = host.state().aside() == Some(object);
                // No question stands: an aside expires in the same report.
                host.report(Report::Due);
                aside && host.state().aside().is_none()
            }
            "reply words" | "reply choice" => {
                let mut host = waiting_host();
                let object = temper_client_view::shared::question(&host.root, host.view.words().view())
                    .expect("question")
                    .object;
                let mut aside = true;
                let duties = if row == "reply words" {
                    let mut duties = Duties::default();
                    host.event(Event::Aside { object }, &mut duties);
                    aside = host.state().aside() == Some(object);
                    typed(&mut host, b"Use a comma");
                    give(&mut host, Event::Send)
                } else {
                    give(
                        &mut host,
                        Event::Act {
                            act: Act::Needs(root::pages::needs::Act::Choose {
                                object,
                                words: b"Comma".as_slice().into(),
                            }),
                        },
                    )
                };
                assert!(duties.asked.is_empty(), "answers wait through the undo hold");
                host.clocks(Time::from_nanos(root::pages::needs::HOLD.as_nanos()), Wall::EPOCH);
                let duties = host.report(Report::Due);
                let duties = stored(&mut host, duties);
                let asks = sent_ask(&duties);
                aside
                    && asks.len() == 1
                    && matches!(&asks[0].1, w::Ask::Answer(answer) if answer.project() == 3 && matches!(answer.on(), w::AnswerOn::Question(question) if question.words() == if row == "reply words" { b"Use a comma".as_slice() } else { b"Comma".as_slice() }))
                    && host.root.pages().chat().is_some_and(|chat| chat.words.is_empty())
                    && host.state().aside().is_none()
            }
            "act" => {
                give(&mut host, Event::Act { act: Act::Go { address: Address::Chat { task: 9 } } });
                host.root.pages().address() == &Address::Chat { task: 9 }
            }
            "size" => {
                let at_start = host.state().columns() == 80;
                host.report(Report::Resize(Size { columns: 40, rows: 12 }));
                at_start && host.state().columns() == 40
            }
            _ => unreachable!("named row"),
        };
        if !passed {
            failures.push(row);
        }
    }
    assert!(failures.is_empty(), "event rows: {failures:?}");
}

fn read_exchange(duties: &Duties, wanted: &str) -> skein_lib::Token {
    duties
        .asked
        .iter()
        .find_map(|request| {
            if let Request::Exchange { exchange, message: Message::Read(bytes), .. } = request {
                let owned = w::Owned::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("read envelope");
                if owned.owner() == &w::Owner::Jig {
                    let read = w::Read::decode(&w::CEILINGS, &mut Reader::new(owned.record())).expect("read");
                    if wanted == "updates"
                        && matches!(&read, w::Read::Inbox(read) if read.group() == &w::InboxReadGroup::Updates)
                    {
                        return Some(*exchange);
                    }
                    if matches!(
                        (&read, wanted),
                        (w::Read::Inbox(_), "inbox")
                            | (w::Read::Question(_), "question")
                            | (w::Read::Activity(_), "activity")
                    ) {
                        return Some(*exchange);
                    }
                }
            }
            None
        })
        .unwrap_or_else(|| panic!("fixture read {wanted}: {:?}", duties.asked))
}

fn read_reply(host: &mut Host, exchange: skein_lib::Token, value: &w::ReadValue) -> Duties {
    let mut writer = Writer::new(usize::try_from(value.measure()).expect("read size"));
    value.encode(&mut writer).expect("read fits");
    let answer = w::ReadAnswer::Value(
        w::Owned::new(
            &w::CEILINGS,
            w::OwnedParts { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: writer.finish() },
        )
        .expect("read document"),
    );
    let mut writer = Writer::new(usize::try_from(answer.measure()).expect("answer size"));
    answer.encode(&mut writer).expect("answer fits");
    reply(host, exchange, writer.finish())
}

fn waiting_host() -> Host {
    waiting_at(true)
}

fn shared_question(host: &Host) -> jig_client::contract::ObjectKey {
    temper_client_view::shared::need_prompts(&host.root, host.view.words().view())
        .first()
        .expect("fixture question")
        .object
}

#[expect(
    clippy::too_many_lines,
    reason = "one wire fixture delivers a waiting chat or launchpad and its answerable question"
)]
fn waiting_at(chat: bool) -> Host {
    let (mut host, _, mut watches) = signed_host(true);
    // The fixture supplies the terminal look so event rows isolate event translation.
    host.step(root::Event::LookKept { look: terminal::Look { terminal: true, ..terminal::Look::default() } });
    host.step(root::Event::Went { place: Place::Address(Address::Home { project: 3 }) });
    watches.asked.extend(host.report(Report::Due).asked);
    if chat {
        host.step(root::Event::Went { place: Place::Address(Address::Chat { task: 7 }) });
    }
    let duties = host.report(Report::Due);
    let duties = admit(&mut host, duties);
    watches.asked.extend(duties.asked);
    if !chat {
        host.clocks(Time::from_nanos(host.env.limits.client.domain.backoff.first.as_nanos()), Wall::EPOCH);
        let duties = host.report(Report::Due);
        watches.asked.extend(admit(&mut host, duties).asked);
    }
    let task_watch = chat.then(|| {
        watch(
            &watches,
            &w::Subject::Task(
                w::SubjectTask::new(&w::CEILINGS, w::SubjectTaskParts { task: 7 }).expect("task subject"),
            ),
        )
    });
    let duties = if let Some(task_watch) = task_watch {
        let mut task = records::task_record().into_parts();
        task.project = 3;
        let offered = w::Offered::new(
            &w::CEILINGS,
            w::OfferedParts { act: w::ActKind::Message, admission: w::Admission::Allowed, effect: None },
        )
        .expect("message offered");
        let mut writer = Writer::new(usize::try_from(offered.measure()).expect("offer size"));
        offered.encode(&mut writer).expect("offer fits");
        task.acts = pieces(&[writer.finish()]);
        task.chip = records::owner_chip_record(
            w::ChipPhase::Active(
                w::ChipPhaseActive::new(&w::CEILINGS, w::ChipPhaseActiveParts { value: w::RunStage::Idle })
                    .expect("idle"),
            ),
            w::Phase::Active,
        );
        task.waits_on = w::WaitsOn::Party(
            w::WaitsOnParty::new(
                &w::CEILINGS,
                w::WaitsOnPartyParts {
                    whom: w::Whom::Person(
                        w::WhomPerson::new(&w::CEILINGS, w::WhomPersonParts { value: 7 }).expect("person"),
                    ),
                    entries: 1,
                    task: 7,
                    kind: w::WaitedFor::Question,
                },
            )
            .expect("waiting"),
        );
        delivery(
            &mut host,
            task_watch,
            &w::DeliveryValue::Task(Box::new(w::Task::new(&w::CEILINGS, task).expect("task"))),
        )
    } else {
        Duties::default()
    };
    let duties = admit(&mut host, duties);
    let window = host.root.pages().inbox().expect("waiting inbox").window;
    watches.asked.extend(duties.asked);
    let inbox_watch = watch(
        &watches,
        &w::Subject::Inbox(w::SubjectInbox::new(&w::CEILINGS, w::SubjectInboxParts { window }).expect("inbox subject")),
    );
    let mut inbox = records::owner_inbox_record().into_parts();
    inbox.party = 7;
    let duties =
        delivery(&mut host, inbox_watch, &w::DeliveryValue::Inbox(w::Inbox::new(&w::CEILINGS, inbox).expect("inbox")));
    let query = read_exchange(&duties, "inbox");
    if !chat {
        let updates = read_exchange(&duties, "updates");
        read_reply(
            &mut host,
            updates,
            &w::ReadValue::Inbox(
                w::InboxPage::new(
                    &w::CEILINGS,
                    w::InboxPageParts { selection: None, cut: false, rows: Box::default(), before: None, after: None },
                )
                .expect("empty updates"),
            ),
        );
    }
    let entry = w::InboxEntry::new(
        &w::CEILINGS,
        w::InboxEntryParts {
            projects: List::with_capacity(0),
            reply: None,
            task: records::stored_link_record(),
            project: Some(3),
            object: w::InboxObject::Question(
                w::InboxObjectQuestion::new(&w::CEILINGS, w::InboxObjectQuestionParts { message: 7 })
                    .expect("question object"),
            ),
            activity: None,
            at: None,
            unread: None,
            cursor: records::live_inbox_cursor_record(),
        },
    )
    .expect("entry");
    let mut writer = Writer::new(usize::try_from(entry.measure()).expect("entry size"));
    entry.encode(&mut writer).expect("entry fits");
    let page = w::InboxPage::new(
        &w::CEILINGS,
        w::InboxPageParts { selection: None, cut: false, rows: pieces(&[writer.finish()]), before: None, after: None },
    )
    .expect("inbox page");
    let duties = read_reply(&mut host, query, &w::ReadValue::Inbox(page));
    let query = read_exchange(&duties, "question");
    let mut question = records::read_wire_question().into_parts();
    question.words = w::DocumentText::new(
        &w::CEILINGS,
        w::DocumentTextParts {
            bytes: b"Which?\nChoice: Comma (recommended)\nChoice: Semicolon".as_slice().into(),
            cut: false,
        },
    )
    .expect("question words");
    let offered = w::Offered::new(
        &w::CEILINGS,
        w::OfferedParts { act: w::ActKind::Answer, admission: w::Admission::Allowed, effect: None },
    )
    .expect("answer offered");
    let mut writer = Writer::new(usize::try_from(offered.measure()).expect("offer size"));
    offered.encode(&mut writer).expect("offer fits");
    question.acts = pieces(&[writer.finish()]);
    read_reply(&mut host, query, &w::ReadValue::Question(w::Question::new(&w::CEILINGS, question).expect("question")));
    host
}

fn admit(host: &mut Host, mut duties: Duties) -> Duties {
    let mut pending = std::collections::VecDeque::from(core::mem::take(&mut duties.asked));
    while let Some(request) = pending.pop_front() {
        if let Some(report) = background_reply(&request) {
            pending.extend(host.report(report).asked);
        }
        duties.asked.push(request);
    }
    duties
}

pub(crate) fn background_reply(request: &Request) -> Option<Report> {
    let Request::Exchange { exchange, message, .. } = request else {
        return matches!(request, Request::Save { .. } | Request::Erase { .. })
            .then_some(Report::Heard(jig_client::Event::Stored));
    };
    let read = w::ReadAnswer::None(
        w::ReadAnswerNone::new(&w::CEILINGS, w::ReadAnswerNoneParts { value: w::ReadNone::Unknown })
            .expect("background read"),
    );
    let size = match message {
        Message::Watch(_) => w::OpenAnswer::Admitted.measure(),
        Message::Unwatch(_) => w::CloseAnswer::Closed.measure(),
        Message::Read(_) => read.measure(),
        Message::Authorize(_)
        | Message::ApprovalAsk(_)
        | Message::ApprovalRead(_)
        | Message::ApprovalDecide(_)
        | Message::Request(_)
        | Message::StreamGone(_)
        | Message::SignInLink(_)
        | Message::SignInApproval(_)
        | Message::SignInService
        | Message::SignOut => return None,
    };
    let mut writer = Writer::new(usize::try_from(size).expect("answer size"));
    match message {
        Message::Watch(_) => w::OpenAnswer::Admitted.encode(&mut writer).expect("watch admitted"),
        Message::Unwatch(_) => w::CloseAnswer::Closed.encode(&mut writer).expect("watch closed"),
        Message::Read(_) => read.encode(&mut writer).expect("read answer"),
        Message::Authorize(_)
        | Message::ApprovalAsk(_)
        | Message::ApprovalRead(_)
        | Message::ApprovalDecide(_)
        | Message::Request(_)
        | Message::StreamGone(_)
        | Message::SignInLink(_)
        | Message::SignInApproval(_)
        | Message::SignInService
        | Message::SignOut => unreachable!("background answer"),
    }
    Some(Report::Heard(jig_client::Event::Reply { exchange: *exchange, message: writer.finish() }))
}

fn one_asking_task(bytes: &[u8]) -> bool {
    let mut reader = Reader::new(bytes);
    let bytes = skein_codec::piece::split(&mut reader).expect("created task piece").expect("one task");
    let task = w::NewTask::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("created task");
    reader.is_empty()
        && task.spec().words() == b"Please help"
        && task.label() == 7
        && matches!(task.executor(), w::Executor::Agent(agent) if agent.charter() == 5)
}

fn created_bytes(task: u64) -> Box<[u8]> {
    let outcome = w::DocumentOutcome::Created(
        w::OutcomeCreated::new(&w::CEILINGS, w::OutcomeCreatedParts { first: task, last: task, given: None })
            .expect("created"),
    );
    let outcome = w::Outcome::Document(
        w::OutcomeDocument::new(&w::CEILINGS, w::OutcomeDocumentParts { value: outcome }).expect("document outcome"),
    );
    final_bytes(&w::Final::Done(w::Done::new(&w::CEILINGS, w::DoneParts { outcome }).expect("done")))
}

fn final_bytes(answer: &w::Final) -> Box<[u8]> {
    let mut writer = Writer::new(usize::try_from(answer.measure()).expect("final size"));
    answer.encode(&mut writer).expect("final fits");
    keyed_bytes(
        w::Owned::new(
            &w::CEILINGS,
            w::OwnedParts { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: writer.finish() },
        )
        .expect("final document"),
    )
}

fn keyed_bytes(document: w::Owned) -> Box<[u8]> {
    let answer = w::KeyedAnswer::FinalAnswer(document);
    let mut writer = Writer::new(usize::try_from(answer.measure()).expect("answer size"));
    answer.encode(&mut writer).expect("answer fits");
    writer.finish()
}

fn hello_for(host: &Host) -> Box<[u8]> {
    let owners = host.env.limits.client.domain.owners;
    let mut served = List::with_capacity(u32::try_from(owners.len() + 1).expect("owner count"));
    served
        .push(
            w::Served::new(
                &w::CEILINGS,
                w::ServedParts {
                    owner: w::Owner::Jig,
                    first: jig_client_protocol_wire::VERSION,
                    last: jig_client_protocol_wire::VERSION,
                },
            )
            .expect("Jig version"),
        )
        .expect("Jig owner");
    for owner in owners {
        let family = w::Family::new(&w::CEILINGS, w::FamilyParts { number: owner.family }).expect("family");
        served
            .push(
                w::Served::new(
                    &w::CEILINGS,
                    w::ServedParts { owner: w::Owner::Family(family), first: owner.version, last: owner.version },
                )
                .expect("family version"),
            )
            .expect("family owner");
    }
    stream_bytes(&w::StreamEvent::Hello(
        w::Hello::new(&w::CEILINGS, w::HelloParts { stream: [7; 16], served, heartbeat: Duration::from_secs(30) })
            .expect("hello"),
    ))
}

enum FrontAct {
    Key(Key),
    Event(Event),
    Send(&'static [u8]),
    Line(temper_client_tui::BoxLine),
}

enum FrontWant {
    Quit { close: bool, asking: bool },
    Log { shown: bool, unknown: Option<&'static [u8]> },
    Line(temper_client_tui::BoxLine),
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks the front’s quit, command and box-line rules")]
fn in_front_quitting_asks_while_work_runs() {
    use temper_client_tui::{BoxLine, BoxState, Told};
    let close = || FrontAct::Key(Key::Control(b'c'));
    let asked = || FrontWant::Quit { close: false, asking: true };
    let closed = || FrontWant::Quit { close: true, asking: false };
    let mut rows = vec![
        ("bare working", false, true, vec![(close(), closed())]),
        ("front idle", true, false, vec![(close(), closed())]),
        ("ask twice", true, true, vec![(close(), asked()), (close(), FrontWant::Quit { close: true, asking: true })]),
        (
            "exit",
            true,
            true,
            vec![
                (FrontAct::Send(b"/exit"), asked()),
                (FrontAct::Send(b"/exit"), FrontWant::Quit { close: true, asking: true }),
            ],
        ),
        (
            "stay",
            true,
            true,
            vec![
                (close(), asked()),
                (FrontAct::Event(Event::Stay), FrontWant::Quit { close: false, asking: false }),
                (close(), asked()),
            ],
        ),
        (
            "stop",
            true,
            true,
            vec![(close(), asked()), (FrontAct::Event(Event::Stop), FrontWant::Quit { close: true, asking: true })],
        ),
    ];
    let gave_up = BoxLine::GaveUp { step: b"forge".as_slice().into() };
    rows.push((
        "given up",
        true,
        true,
        vec![(FrontAct::Line(gave_up.clone()), FrontWant::Line(gave_up)), (close(), closed())],
    ));
    for (name, front, words) in [
        ("log", true, b"/log".as_slice()),
        ("log begun", true, b"/lo"),
        ("bare log", false, b"/log"),
        ("bare log begun", false, b"/lo"),
    ] {
        rows.push((
            name,
            front,
            true,
            vec![(
                FrontAct::Send(words),
                FrontWant::Log { shown: front, unknown: if front { None } else { Some(words) } },
            )],
        ));
    }
    let printed = local_link("127.0.0.1:8367", FRESH_SECRET);
    for (name, line) in [
        ("link", BoxLine::Link { url: printed }),
        ("box log", BoxLine::Log { line: b"recent".as_slice().into() }),
        ("restarting", BoxLine::Restarting { step: b"forge".as_slice().into() }),
        ("running", BoxLine::Running { step: b"forge".as_slice().into() }),
        ("gave-up", BoxLine::GaveUp { step: b"forge".as_slice().into() }),
    ] {
        rows.push((name, true, true, vec![(FrontAct::Line(line.clone()), FrontWant::Line(line))]));
    }
    let mut failures = Vec::new();
    for (name, front, running, steps) in rows {
        let mut host = if running { home_host(true) } else { signed_host(true).0 };
        if front {
            host.tell(Told::Front);
        }
        // Running must clear a known restart, rather than leave an already running box alone.
        host.report(Report::Front(BoxLine::Restarting { step: b"forge".as_slice().into() }));
        for (act, want) in steps {
            let duties = match act {
                FrontAct::Key(key) => host.report(Report::Key(key)),
                FrontAct::Event(event) => give(&mut host, event),
                FrontAct::Send(words) => {
                    typed(&mut host, words);
                    give(&mut host, Event::Send)
                }
                FrontAct::Line(line) => host.report(Report::Front(line)),
            };
            let passed = match want {
                FrontWant::Quit { close, asking } => duties.close == close && host.state().quitting() == asking,
                FrontWant::Log { shown, unknown } => {
                    duties.asked.is_empty()
                        && !duties.close
                        && host.state().log_shown() == shown
                        && host.state().unknown() == unknown
                        && host.words().is_empty()
                }
                FrontWant::Line(line) => {
                    !duties.close
                        && match line {
                            BoxLine::Link { url } => host.state().printed() == Some(&url),
                            BoxLine::Log { line } => host.state().box_log().collect::<Vec<_>>() == vec![line.as_ref()],
                            BoxLine::Restarting { step } => {
                                host.state().box_state() == BoxState::Restarting { step: &step }
                            }
                            BoxLine::Running { .. } => host.state().box_state() == BoxState::Running,
                            BoxLine::GaveUp { step } => host.state().box_state() == BoxState::GaveUp { step: &step },
                        }
                }
            };
            if !passed {
                failures.push(name);
            }
        }
    }
    assert!(failures.is_empty(), "front rows: {failures:?}");
}

fn working_chip(task: u64, title: &[u8]) -> w::Chip {
    chat_chip_stage(task, title, w::RunStage::Due)
}

pub(crate) fn home_host(running: bool) -> Host {
    home_with_tasks(home_tasks(running))
}

fn one_chat_host(brief: bool) -> Host {
    let mut tasks = home_tasks(false).into_parts();
    let mut reader = Reader::new(&tasks.rows);
    skein_codec::piece::split(&mut reader).expect("older chat");
    let newest = skein_codec::piece::split(&mut reader).expect("newest chat").expect("one chat");
    tasks.rows = pieces(&[newest]);
    home_with_brief(w::Tasks::new(&w::CEILINGS, tasks).expect("one chat"), brief)
}

fn home_with_tasks(tasks: w::Tasks) -> Host {
    home_with_brief(tasks, false)
}

fn home_with_brief(tasks: w::Tasks, brief: bool) -> Host {
    let (mut host, _, mut duties) = signed_host(true);
    // The first wave can fill the exchange slots; give its watch retry a turn.
    host.clocks(Time::from_nanos(host.env.limits.client.domain.backoff.first.as_nanos()), Wall::EPOCH);
    let again = host.report(Report::Due);
    duties.asked.extend(admit(&mut host, again).asked);
    let subject = home_subject(&duties);
    let watched = watch(&duties, &subject);
    let more = delivery(&mut host, watched, &w::DeliveryValue::Tasks(tasks));
    duties.asked.extend(stored(&mut host, more).asked);
    if brief {
        deliver_brief(&mut host, &mut duties);
    }
    host.frame();
    host
}

fn deliver_brief(host: &mut Host, duties: &mut Duties) {
    use temper_client_protocol_wire::{self as wire, v1 as f};
    let jig_client::Watchable::Family { record, .. } =
        root::forge::status::watch(3, &host.env.limits.forge).expect("status watch")
    else {
        panic!("family watch")
    };
    let watched = duties
        .asked
        .iter()
        .find_map(|request| {
            let Request::Exchange { message: Message::Watch(bytes), .. } = request else { return None };
            let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("watch");
            if open.subject().owner() == &w::Owner::Jig {
                return None;
            }
            let envelope = w::FamilyRecord::decode(&w::CEILINGS, &mut Reader::new(open.subject().record()))
                .expect("family subject");
            (envelope.record() == record.bytes.as_ref()).then_some(open.watch())
        })
        .expect("status watched");
    let ended = duties
        .asked
        .iter()
        .filter_map(|request| {
            let Request::Exchange { message: Message::Watch(bytes), .. } = request else { return None };
            let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("watch");
            if open.watch() == watched {
                return None;
            }
            let end = if open.subject().owner() == &w::Owner::Jig {
                matches!(
                    w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject().record())).expect("subject"),
                    w::Subject::Settings
                )
            } else {
                true
            };
            end.then_some(open.watch())
        })
        .collect::<Vec<_>>();
    for watch in ended {
        let event = w::StreamEvent::Ended(
            w::WatchEnded::new(&w::CEILINGS, w::WatchEndedParts { watch, why: w::WatchEnd::Unknown })
                .expect("background watch ended"),
        );
        let more = host.report(Report::Heard(jig_client::Event::Streamed {
            message: jig_client::Streamed::Record(stream_bytes(&event)),
        }));
        duties.asked.extend(stored(host, more).asked);
    }
    let document = brief_document();
    let event = w::StreamEvent::Delivery(
        w::Delivery::new(&w::CEILINGS, w::DeliveryParts { watch: watched, document }).expect("status delivery"),
    );
    host.report(Report::Heard(jig_client::Event::Streamed {
        message: jig_client::Streamed::Record(stream_bytes(&event)),
    }));
    // The look's answer makes the delivered status visible.
    for request in &duties.asked {
        let Request::Exchange { exchange, message: Message::Request(bytes), .. } = request else { continue };
        let ask = w::Keyed::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("keyed ask");
        if ask.owner() != &w::Owner::Jig {
            let answer = f::Answer::new(&f::CEILINGS, f::AnswerParts { outcome: f::Outcome::Looked }).expect("looked");
            let document = w::Owned::new(
                &w::CEILINGS,
                w::OwnedParts {
                    owner: ask.owner().clone(),
                    version: wire::CLIENT_VERSION,
                    record: wire::answer(&answer),
                },
            )
            .expect("looked answer");
            reply(host, *exchange, keyed_bytes(document));
        }
    }
}

fn brief_document() -> w::Owned {
    use temper_client_protocol_wire::{self as wire, v1 as f};
    let status = f::ProjectStatus::new(
        &f::CEILINGS,
        f::ProjectStatusParts {
            project: 3,
            landed: 1,
            last_landed: Some(1),
            previous: Some(0),
            seen: 0,
            queued: 0,
            red: false,
            branch: b"main".as_slice().into(),
            brief: Some(
                f::StatusBrief::new(
                    &f::CEILINGS,
                    f::StatusBriefParts {
                        sentence: b"One change landed.".as_slice().into(),
                        lede: b"The CSV export landed.".as_slice().into(),
                        at: 1,
                        landed: 1,
                    },
                )
                .expect("brief"),
            ),
        },
    )
    .expect("project status");
    w::Owned::new(
        &w::CEILINGS,
        w::OwnedParts {
            owner: w::Owner::Family(
                w::Family::new(&w::CEILINGS, w::FamilyParts { number: wire::FORGE }).expect("forge owner"),
            ),
            version: wire::CLIENT_VERSION,
            record: wire::project_status(&status),
        },
    )
    .expect("status document")
}

pub(crate) fn home_tasks(running: bool) -> w::Tasks {
    let mut rows = Vec::new();
    for (task, title, activity) in [(OLDER, b"Older".as_slice(), 1), (NEWEST, b"Newest".as_slice(), 2)] {
        let mut row = records::task_line_record().into_parts();
        row.chip = if running { working_chip(task, title) } else { chat_chip(task, title) };
        if running {
            row.waits_on = w::WaitsOn::Nothing;
        }
        row.project = 3;
        row.last_activity = activity;
        row.ended_at = None;
        let row = w::TaskLine::new(&w::CEILINGS, row).expect("chat line");
        let mut writer = Writer::new(usize::try_from(row.measure()).expect("chat size"));
        row.encode(&mut writer).expect("chat fits");
        rows.push(writer.finish());
    }
    w::Tasks::new(&w::CEILINGS, w::TasksParts { rows: pieces(&rows), more: false }).expect("two chats")
}

pub(crate) fn home_subject(duties: &Duties) -> w::Subject {
    duties
        .asked
        .iter()
        .find_map(|request| {
            let Request::Exchange { message: Message::Watch(bytes), .. } = request else { return None };
            let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("watch");
            if open.subject().owner() != &w::Owner::Jig {
                return None;
            }
            let subject = w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject().record())).expect("subject");
            matches!(&subject, w::Subject::Tasks(tasks) if *tasks.query().label() == Some(8)).then_some(subject)
        })
        .expect("launchpad's chats watch")
}
