#[path = "tests/goal.rs"]
mod goal;

use crate::{
    host::{Duties, Host, Report, Settings, Start},
    limits,
};
use jig_client::{Message, Request};
use jig_client_protocol_wire::v20 as w;
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
use temper_client_tui::{BoxEnd, BoxStage, Event, Local, Note, Own, Scheme, Told, WalkStep};

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
        scheme: Some(Scheme::Dark),
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
    assert!(duties.stop.is_none());
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
    use jig_client_protocol_wire::v20 as w;
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
            scheme: Some(Scheme::Dark),
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
    use jig_client_protocol_wire::v20 as w;
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
    use jig_client_protocol_wire::v20 as w;
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

fn start_local(host: &mut Host, front: bool, local: bool, credential: Option<Secret>, note: Option<Note>) -> Duties {
    host.report(Report::Start(Start {
        front,
        size: Size { columns: 80, rows: 24 },
        scheme: Some(Scheme::Dark),
        background: None,
        boxed: false,
        address: b"127.0.0.1:8367".as_slice().into(),
        local,
        kept: Vec::new(),
        credential,
        note,
    }))
}

fn local_row(row: LocalRow, failures: &mut Vec<String>) {
    let mut host = host();
    let duties =
        start_local(&mut host, row.front, row.local, row.held.then(|| Secret::new(Box::new([7; 40]))), row.note);
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
    maybe_watch(duties, subject).unwrap_or_else(|| panic!("requested watch {subject:?}: {:?}", duties.asked))
}
fn maybe_watch(duties: &Duties, subject: &w::Subject) -> Option<u64> {
    duties.asked.iter().rev().find_map(|request| {
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

#[test]
fn typing_open_opens_the_selected_launchpad_chat() {
    for size in [Size { columns: 80, rows: 24 }, Size { columns: 120, rows: 40 }] {
        for selected in [true, false] {
            let mut host = left_off_host();
            host.report(Report::Resize(size));
            host.frame();
            if selected {
                host.report(Report::Key(Key::Up));
                host.frame();
                assert_eq!(host.state().left_off(), Some(NEWEST));
            }
            let address = if selected {
                b"http://127.0.0.1:8367/chats/9".as_slice()
            } else {
                b"http://127.0.0.1:8367/projects/3".as_slice()
            };
            check_typed_open(&mut host, size, address);
        }
    }
}

fn goal_me() -> w::Me {
    let mut me = launchpad_me(true).into_parts();
    let mut reader = Reader::new(&me.projects);
    let project = skein_codec::piece::split(&mut reader).expect("project piece").expect("project");
    let mut project = w::MeProject::decode(&w::CEILINGS, &mut Reader::new(&project)).expect("project").into_parts();
    let mut reader = Reader::new(&project.templates);
    let template = skein_codec::piece::split(&mut reader).expect("template piece").expect("template");
    let mut template = w::Template::decode(&w::CEILINGS, &mut Reader::new(&template)).expect("template").into_parts();
    template.number = jig_client::shapes::GOAL;
    template.label = 10;
    let template = w::Template::new(&w::CEILINGS, template).expect("goal template");
    let mut writer = Writer::new(usize::try_from(template.measure()).expect("template size"));
    template.encode(&mut writer).expect("template fits");
    project.templates = [project.templates.as_ref(), pieces(&[writer.finish()]).as_ref()].concat().into();
    me.projects = pieces(&[project_bytes(&w::MeProject::new(&w::CEILINGS, project).expect("goal project"))]);
    w::Me::new(&w::CEILINGS, me).expect("Me")
}

fn goal_tasks() -> w::Tasks {
    let mut row = records::task_line_record().into_parts();
    row.chip = chat_chip(NEWEST, b"Goal");
    row.project = 3;
    row.label = 10;
    row.requester = w::Requester::Person(
        w::RequesterPerson::new(&w::CEILINGS, w::RequesterPersonParts { value: 7 }).expect("person"),
    );
    let row = w::TaskLine::new(&w::CEILINGS, row).expect("goal row");
    let mut writer = Writer::new(usize::try_from(row.measure()).expect("goal size"));
    row.encode(&mut writer).expect("goal fits");
    w::Tasks::new(&w::CEILINGS, w::TasksParts { rows: pieces(&[writer.finish()]), more: false }).expect("goal")
}

fn goal_host() -> Host {
    let (mut host, me_watch, _) = signed_host(true);
    let duties = delivery(&mut host, me_watch, &w::DeliveryValue::Me(goal_me()));
    let mut duties = admit(&mut host, duties);
    let tasks = goal_tasks();
    for _ in 0..host.env.limits.client.domain.watches {
        let subjects: Vec<_> = duties
            .asked
            .iter()
            .filter_map(|request| {
                let Request::Exchange { message: Message::Watch(bytes), .. } = request else { return None };
                let open = w::Open::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("watch");
                if open.subject().owner() != &w::Owner::Jig {
                    return None;
                }
                let subject =
                    w::Subject::decode(&w::CEILINGS, &mut Reader::new(open.subject().record())).expect("subject");
                matches!(&subject, w::Subject::Tasks(tasks) if *tasks.query().label() == Some(10)).then_some(subject)
            })
            .collect();
        for subject in subjects {
            let watched = watch(&duties, &subject);
            let arrived = delivery(&mut host, watched, &w::DeliveryValue::Tasks(tasks.clone()));
            duties.asked.extend(admit(&mut host, arrived).asked);
        }
        if host.root.pages().goals().is_some_and(|goals| goals.live(host.root.client()).is_some())
            && host.root.pages().left_off().is_some_and(|reads| reads.goals(host.root.client()).is_some())
        {
            break;
        }
    }
    host.frame();
    host
}

#[test]
fn typing_open_opens_the_selected_launchpad_goal() {
    for size in [Size { columns: 80, rows: 24 }, Size { columns: 120, rows: 40 }] {
        let mut host = goal_host();
        host.report(Report::Resize(size));
        host.frame();
        host.report(Report::Key(Key::Up));
        host.frame();
        assert_eq!(host.state().left_off(), Some(NEWEST));
        check_typed_open(&mut host, size, b"http://127.0.0.1:8367/tasks/9");
    }
}

fn check_typed_open(host: &mut Host, size: Size, address: &[u8]) {
    key_words(host, "/open");
    let duties = host.report(Report::Key(Key::Enter));
    assert_eq!(duties.open.as_deref(), Some(address), "{size:?}: focused page");
    host.report(Report::Opened { opened: false });
    assert_eq!(host.state().not_opened(), Some(address));
    let address = core::str::from_utf8(address).expect("ordinary address");
    assert!(text(host).contains(&format!("Open {address} in your browser.")));
}

#[test]
fn launchpad_selection_clears_after_command_composition() {
    for (words, key) in [("", Key::Char('a')), ("/", Key::Escape), ("/open", Key::Escape), ("/new", Key::Enter)] {
        let mut host = left_off_host();
        host.frame();
        host.report(Report::Key(Key::Up));
        host.frame();
        key_words(&mut host, words);
        assert_eq!(host.state().left_off(), Some(NEWEST), "{words}: composing");
        host.report(Report::Key(key));
        host.frame();
        assert_eq!(host.state().left_off(), None, "{words}: cleared");
    }
}

#[test]
fn a_kept_theme_starts_the_next_host_in_it() {
    use temper_client_tui::theme::{THEME_KEY, ThemeChoice};
    for (bytes, scheme, expected) in [
        (Some(b"light".as_slice()), None, Scheme::Light),
        (Some(b"junk".as_slice()), None, Scheme::Dark),
        (Some(b"junk".as_slice()), Some(Scheme::Light), Scheme::Light),
        (None, None, Scheme::Dark),
    ] {
        let mut host = host();
        let mut kept = vec![jig_client::Record { key: b"other".as_slice().into(), bytes: Box::new([]) }];
        if let Some(bytes) = bytes {
            kept.push(jig_client::Record { key: THEME_KEY.into(), bytes: b"dark".as_slice().into() });
            kept.push(jig_client::Record { key: THEME_KEY.into(), bytes: bytes.into() });
        }
        host.start(
            Start {
                front: false,
                size: Size { columns: 80, rows: 24 },
                scheme,
                background: None,
                boxed: false,
                address: b"127.0.0.1:8367".as_slice().into(),
                local: false,
                note: None,
                kept,
                credential: None,
            },
            &mut Duties::default(),
        );
        // Only the other record reaches jig's store reader, which drops its invalid bytes.
        let mut dropped = 0;
        while let Some(request) = host.out.pop() {
            if let root::Request::Dropped { count } = request {
                dropped += count;
            }
        }
        assert_eq!(dropped, 1, "every theme record is taken before the root's Start");
        host.report(Report::Due);
        assert_eq!(
            host.root.pages().look().theme,
            match expected {
                Scheme::Light => Theme::Light,
                Scheme::Dark => Theme::Dark,
            }
        );
        assert_eq!(host.theme, temper_client_tui::theme(expected, None));
        let expected_choice = bytes.map_or(ThemeChoice::Auto, ThemeChoice::read);
        assert_eq!(host.choice, expected_choice);
        assert_eq!(host.state().theme_choice(), expected_choice);
        assert_eq!(host.state().terminal_scheme(), scheme);
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one ordered-write story takes choices before answering root and host writes"
)]
fn a_theme_write_answer_never_reaches_the_root() {
    use temper_client_tui::theme::{THEME_KEY, ThemeChoice};
    for (closing, erase_first) in [(false, false), (true, false), (false, true)] {
        let (mut host, _, _) = signed_host(false);
        let first = jig_client::Key { asker: jig_client::Asker::JIG, number: 1001 };
        let second = jig_client::Key { asker: jig_client::Asker::JIG, number: 1002 };
        let send = |key| root::Event::Client {
            event: jig_client::Event::Send {
                key,
                name: None,
                request: jig_client::Ask::Jig(Box::new(jig_client::contract::Ask::Stop { project: 3, task: 9 })),
            },
        };
        key_words(&mut host, "/theme");
        host.report(Report::Key(Key::Enter));
        host.frame();
        host.report(Report::Key(Key::Down));
        host.step(send(first));
        if erase_first {
            let duties = host.report(Report::Due);
            assert!(matches!(duties.asked.as_slice(), [Request::Save { .. }]));
            host.heard(jig_client::Event::NotStored);
            assert!(host.root.client().pending(first).is_none());
        }
        let duties = host.report(Report::Key(Key::Enter));
        let writes: Vec<_> = duties
            .asked
            .iter()
            .filter(|request| matches!(request, Request::Save { .. } | Request::Erase { .. }))
            .collect();
        let [root_write, Request::Save { records }] = writes.as_slice() else {
            panic!("root write followed by theme save: {writes:?}");
        };
        if erase_first {
            assert!(matches!(root_write, Request::Erase { .. }));
        } else {
            let Request::Save { records: root_records } = root_write else {
                panic!("root save");
            };
            assert!(!root_records.as_slice().iter().any(|record| record.key.as_ref() == THEME_KEY));
        }
        assert_eq!(
            records.as_slice(),
            &[jig_client::Record { key: THEME_KEY.into(), bytes: b"light".as_slice().into() }]
        );
        assert_eq!(host.theme, temper_client_tui::theme(Scheme::Light, None));
        host.step(send(second));
        let duties = host.report(Report::Due);
        assert_eq!(duties.asked.iter().filter(|request| matches!(request, Request::Save { .. })).count(), 1);
        if closing {
            assert!(host.report(Report::Key(Key::Control(b'c'))).close);
        }
        let requests = |duties: &Duties| {
            duties
                .asked
                .iter()
                .filter(|request| matches!(request, Request::Exchange { message: Message::Request(_), .. }))
                .count()
        };
        assert_eq!(requests(&host.report(Report::Heard(jig_client::Event::Stored))), 0);
        let before = host.root.client().pending(second).expect("second root write waits");
        let refused = host.report(Report::Heard(jig_client::Event::NotStored));
        assert!(refused.asked.is_empty(), "host refusal emits no root duties: {:?}", refused.asked);
        empty(&refused);
        assert_eq!(
            host.root.client().pending(second),
            Some(before),
            "host refusal cannot answer the root's next write"
        );
        assert_eq!(
            requests(&host.report(Report::Heard(jig_client::Event::Stored))),
            if erase_first { 1 } else { 2 },
            "three answers release only the root's saved requests"
        );
        assert_eq!(host.root_writes.len(), 0);
        if !closing {
            key_words(&mut host, "/theme");
            host.report(Report::Key(Key::Enter));
            host.frame();
            host.report(Report::Key(Key::Down));
            let duties = host.report(Report::Key(Key::Enter));
            let [Request::Save { records }] = duties.asked.as_slice() else {
                panic!("next choice writes whole");
            };
            assert_eq!(
                records.as_slice(),
                &[jig_client::Record { key: THEME_KEY.into(), bytes: b"dark".as_slice().into() }]
            );
            host.report(Report::Heard(jig_client::Event::Stored));
            let look = host.root.pages().look();
            let mut duties = Duties::default();
            host.keep_theme(ThemeChoice::Auto, &mut duties);
            assert_eq!(host.root.pages().look(), look, "keeping a theme preserves the root's other look fields");
            assert_eq!(host.choice, ThemeChoice::Auto);
            let [Request::Save { records }] = duties.asked.as_slice() else {
                panic!("Auto is saved too");
            };
            assert_eq!(
                records.as_slice(),
                &[jig_client::Record { key: THEME_KEY.into(), bytes: b"auto".as_slice().into() }]
            );
        }
    }
}

#[test]
fn the_theme_list_takes_keeps_and_notices_a_choice() {
    use temper_client_tui::theme::{THEME_KEY, ThemeChoice};
    let (mut host, _, _) = signed_host(false);
    for (command, sent) in [("/theme", false), ("/th", false), ("/theme", true), ("/th", true)] {
        key_words(&mut host, command);
        let duties = if sent { give(&mut host, Event::Send) } else { host.report(Report::Key(Key::Enter)) };
        empty(&duties);
        host.frame();
        assert!(host.state().theme_list());
        assert!(host.words().is_empty());
        assert_eq!(host.event_of(&Key::Enter), Some(Event::Theme { choice: Some(ThemeChoice::Auto) }));
        empty(&host.report(Report::Key(Key::Escape)));
        assert!(!host.state().theme_list());
    }
    give(&mut host, Event::Own { command: Own::Theme });
    host.frame();
    host.report(Report::Key(Key::Down));
    let duties = host.report(Report::Key(Key::Enter));
    let [Request::Save { records }] = duties.asked.as_slice() else {
        panic!("one theme saved: {:?}", duties.asked);
    };
    let kept = records.as_slice().to_vec();
    assert_eq!(kept, [jig_client::Record { key: THEME_KEY.into(), bytes: b"light".as_slice().into() }]);
    assert_eq!(host.root.pages().look().theme, Theme::Light);
    assert_eq!(host.state().theme_choice(), ThemeChoice::Light);
    assert_eq!(
        host.state().notice(),
        Some(&temper_client_tui::Notice::Theme { choice: ThemeChoice::Light, terminal: Some(Scheme::Dark) })
    );
    assert!(text(&mut host).contains("Theme: light."));
    assert!(!host.state().theme_list());
    assert!(host.words().is_empty());
    assert_eq!(host.theme, temper_client_tui::theme(Scheme::Light, None));
    let palette = temper_client_tui::theme::palette(Scheme::Light);
    let frame = host.frame();
    assert!(
        (0..frame.size().rows)
            .filter_map(|row| frame.row(row))
            .flatten()
            .any(|cell| palette.iter().any(|(_, colour)| cell.paint.fore == Some(*colour)))
    );
    host.report(Report::Heard(jig_client::Event::Stored));
    give(&mut host, Event::Own { command: Own::Theme });
    host.frame();
    assert_eq!(host.event_of(&Key::Enter), Some(Event::Theme { choice: Some(ThemeChoice::Light) }));
    host.report(Report::Key(Key::Char('h')));
    assert!(!host.state().theme_list());
    assert_eq!(host.words(), b"h");
    give(&mut host, Event::Own { command: Own::Theme });
    host.frame();
    host.report(Report::Key(Key::Up));
    host.report(Report::Key(Key::Enter));
    assert_eq!(host.state().theme_choice(), ThemeChoice::Auto);
    assert_eq!(host.root.pages().look().theme, Theme::Dark);

    let mut restarted = self::host();
    restarted.report(Report::Start(Start {
        front: true,
        size: Size { columns: 80, rows: 24 },
        scheme: None,
        background: None,
        boxed: true,
        address: b"127.0.0.1:8367".as_slice().into(),
        local: false,
        note: None,
        kept,
        credential: None,
    }));
    give(&mut restarted, Event::Own { command: Own::Theme });
    restarted.frame();
    assert_eq!(restarted.state().theme_choice(), ThemeChoice::Light);
    assert_eq!(restarted.event_of(&Key::Enter), Some(Event::Theme { choice: Some(ThemeChoice::Light) }));
    restarted.report(Report::Key(Key::Up));
    restarted.report(Report::Key(Key::Enter));
    assert_eq!(restarted.state().terminal_scheme(), None);
    assert_eq!(restarted.state().theme_choice(), ThemeChoice::Auto);
    assert_eq!(restarted.root.pages().look().theme, Theme::Dark);
    assert!(text(&mut restarted).contains("Theme: auto, dark: your terminal doesn't say."));
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

const REPLY_TURN: jig_client::contract::TurnName = jig_client::contract::TurnName { attempt: 44, number: 1 };

fn reply_host() -> Host {
    reply_host_draft(false)
}
fn pane_document() -> w::NotePage {
    let body = format!("# Draft\n\n{}", "line\n\n".repeat(40));
    let note = w::Note::new(
        &w::CEILINGS,
        w::NoteParts {
            number: 1,
            revision: w::Revision::new(&w::CEILINGS, w::RevisionParts { value: 1 }).unwrap(),
            scope: w::NoteScope::Task(
                w::NoteScopeTask::new(&w::CEILINGS, w::NoteScopeTaskParts { project: 3, task: 7 }).unwrap(),
            ),
            description: records::wire_text(b"Draft"),
            body: w::DocumentText::new(
                &w::CEILINGS,
                w::DocumentTextParts { bytes: body.into_bytes().into(), cut: false },
            )
            .unwrap(),
            writer: w::NoteAuthor::Task(
                w::NoteAuthorTask::new(&w::CEILINGS, w::NoteAuthorTaskParts { task: 7, attempt: 1 }).unwrap(),
            ),
            tasks: List::with_capacity(0),
            changed: None,
            acts: Box::default(),
            cut: false,
        },
    )
    .unwrap();
    let mut writer = Writer::new(usize::try_from(note.measure()).unwrap());
    note.encode(&mut writer).unwrap();
    w::NotePage::new(
        &w::CEILINGS,
        w::NotePageParts { cut: false, rows: pieces(&[writer.finish()]), before: None, after: None },
    )
    .unwrap()
}
fn reply_host_draft(draft: bool) -> Host {
    use smith_transcript as s;
    let (mut host, me_watch, _) = signed_host(true);
    let mut me = launchpad_me(true).into_parts();
    let mut reader = Reader::new(&me.projects);
    let project = skein_codec::piece::split(&mut reader).unwrap().unwrap();
    let mut project = w::MeProject::decode(&w::CEILINGS, &mut Reader::new(&project)).unwrap().into_parts();
    let mut reader = Reader::new(&project.templates);
    let template = skein_codec::piece::split(&mut reader).unwrap().unwrap();
    let mut template = w::Template::decode(&w::CEILINGS, &mut Reader::new(&template)).unwrap().into_parts();
    template.number = 6;
    let template = w::Template::new(&w::CEILINGS, template).unwrap();
    let mut writer = Writer::new(usize::try_from(template.measure()).unwrap());
    template.encode(&mut writer).unwrap();
    project.templates = [project.templates.as_ref(), pieces(&[writer.finish()]).as_ref()].concat().into();
    me.projects = pieces(&[project_bytes(&w::MeProject::new(&w::CEILINGS, project).unwrap())]);
    let duties = delivery(&mut host, me_watch, &w::DeliveryValue::Me(w::Me::new(&w::CEILINGS, me).unwrap()));
    admit(&mut host, duties);
    let duties = give(&mut host, Event::Act { act: Act::Go { address: Address::Chat { task: 7 } } });
    let duties = admit_watches(&mut host, duties);
    let task_watch =
        watch(&duties, &w::Subject::Task(w::SubjectTask::new(&w::CEILINGS, w::SubjectTaskParts { task: 7 }).unwrap()));
    let activity = read_exchange(&duties, "activity");
    let mut task = records::task_record().into_parts();
    task.project = 3;
    let chip = chat_chip(7, b"Choices");
    let mut reader = Reader::new(chip.rows());
    let mut rows = Vec::new();
    while let Some(bytes) = skein_codec::piece::split(&mut reader).unwrap() {
        let mut fact = w::Fact::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
        if let w::Fact::Chip(row) = fact {
            let mut row = row.into_parts();
            let mut fields = row.value.into_parts();
            fields.executor = w::ChipExecutor::Agent(
                w::ChipExecutorAgent::new(&w::CEILINGS, w::ChipExecutorAgentParts { charter: 5 }).unwrap(),
            );
            row.value = w::ChipFields::new(&w::CEILINGS, fields).unwrap();
            fact = w::Fact::Chip(w::ChipRow::new(&w::CEILINGS, row).unwrap());
        } else if let w::Fact::ChipDetails(row) = fact {
            let mut row = row.into_parts();
            let mut fields = row.value.into_parts();
            fields.live_delegates = 0;
            row.value = w::ChipDetailsFields::new(&w::CEILINGS, fields).unwrap();
            fact = w::Fact::ChipDetails(Box::new(w::ChipDetailsRow::new(&w::CEILINGS, row).unwrap()));
        }
        rows.push(fact);
    }
    task.chip = records::chip_from_rows(&rows);
    task.activity = 1;
    task.result = None;
    task.waits_on = w::WaitsOn::News;
    let offer = w::Offered::new(
        &w::CEILINGS,
        w::OfferedParts { act: w::ActKind::Message, admission: w::Admission::Allowed, effect: None },
    )
    .unwrap();
    let mut writer = Writer::new(usize::try_from(offer.measure()).unwrap());
    offer.encode(&mut writer).unwrap();
    task.acts = pieces(&[writer.finish()]);
    let duties =
        delivery(&mut host, task_watch, &w::DeliveryValue::Task(Box::new(w::Task::new(&w::CEILINGS, task).unwrap())));
    let duties = admit_watches(&mut host, duties);
    if draft {
        let read = read_exchange(&duties, "notes");
        let page = pane_document();
        read_reply(&mut host, read, &w::ReadValue::Notes(page));
    }
    let mut turn = s::decode_archived(&s::CEILINGS, records::FULL_TURN).unwrap().into_parts();
    let text = s::Block::Text(
        s::Text::new(
            &s::CEILINGS,
            s::TextParts {
                text: "Choose.\n\n1. Move it at once — no mixed state\n2. Side by side\n3. Tell me something else"
                    .as_bytes()
                    .into(),
                replay: None,
            },
        )
        .unwrap(),
    );
    turn.messages = List::from_boxed(Box::new([s::Message::new(
        &s::CEILINGS,
        s::MessageParts { role: s::Role::Assistant, blocks: List::from_boxed(Box::new([text])) },
    )
    .unwrap()]));
    let turn = s::Turn::new(&s::CEILINGS, turn).unwrap();
    let mut writer = Writer::new(usize::try_from(turn.measure()).unwrap());
    turn.encode(&mut writer).unwrap();
    let mut page = records::activity_rows(&[records::turn_line(1, 0, 1, &writer.finish())]).into_parts();
    page.after = None;
    page.cut = false;
    read_reply(&mut host, activity, &w::ReadValue::Activity(w::ActivityPage::new(&w::CEILINGS, page).unwrap()));
    host.frame();
    assert_eq!(temper_client_tui::reply_number(&host.view, b"3"), Some(Event::ReplyAside { turn: REPLY_TURN }));
    host
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
    for command in ["/theme", "/help", "/quit", "Close temper"] {
        assert!(frame.contains(command), "offered command list: {command}\n{frame}");
    }
    key_words(&mut host, "quit");
    assert!(text(&mut host).contains("❯ /quit"));
    assert!(host.report(Report::Key(Key::Enter)).close, "/quit must close from the launchpad");
    assert!(host.words().is_empty(), "closing clears the command composer");
    for (words, complete) in [("/q", false), ("/q", true), ("/qu", true), ("/exit", false)] {
        let (mut host, _, _) = signed_host(false);
        key_words(&mut host, words);
        if complete {
            host.frame();
            empty(&host.report(Report::Key(Key::Tab)));
            assert_eq!(host.words(), b"/quit", "Tab completes the listed closing command");
        }
        assert!(host.report(Report::Key(Key::Enter)).close, "{words} must close from the launchpad");
        assert!(host.words().is_empty(), "closing clears the command composer");
    }
    for words in ["/ex", "/exi"] {
        let (mut host, _, _) = signed_host(false);
        key_words(&mut host, words);
        assert!(!text(&mut host).contains("Close temper"), "an alias prefix draws no list");
        host.frame();
        empty(&host.report(Report::Key(Key::Tab)));
        assert_eq!(host.words(), words.as_bytes(), "Tab never completes the hidden alias");
        empty(&host.report(Report::Key(Key::Enter)));
        assert_eq!(
            host.state().notice(),
            Some(&temper_client_tui::Notice::Unknown { words: words.as_bytes().into() }),
            "an alias prefix is unknown"
        );
        assert!(host.words().is_empty(), "unknown commands clear the composer");
        assert!(!text(&mut host).contains("Close temper"), "an unknown command draws no list");
    }
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
            assert_eq!(host.state().command_words(), b"/he", "sign-out keeps command words");
            assert_eq!(host.state().box_written(), written);
        } else {
            delivery(&mut host, me, &w::DeliveryValue::Me(launchpad_me(true)));
            assert_eq!(host.root.pages().address(), &Address::Home { project: 3 });
            assert_eq!(host.words(), b"/he", "navigation carries words into the project composer");
            assert!(text(&mut host).contains("❯ /he"));
            assert!(host.state().command_words().is_empty(), "navigation clears the no-project composer");
            assert!(host.state().box_written() > written);
        }
    }
}

#[test]
fn keys_typed_during_the_file_sign_in_stay_in_the_box() {
    for (front, first) in [(true, "Welcome. Set temper up"), (false, "Connecting to 127.0.0.1:8367")] {
        let mut host = host();
        let start = start_local(&mut host, front, true, None, None);
        assert!(start.local);
        assert!(text(&mut host).contains(first));
        empty(&host.report(Report::Key(Key::Char('a'))));
        assert!(text(&mut host).contains("❯ a"), "first frame, front {front}");
        assert!(host.state().field().is_empty());
        let signing = host.report(Report::Local { link: Some(local_link("127.0.0.1:8367", LOCAL_SECRET)) });
        let exchange = local_exchange(&signing);
        assert_eq!(signing.asked.len(), 1, "only the sign-in request");
        empty(&host.report(Report::Key(Key::Enter)));
        assert!(text(&mut host).contains("Still signing in"));
        assert_eq!(host.words(), b"a");
        let duties = reply(&mut host, exchange, sign_in_bytes(&signed_in_answer()));
        admit(&mut host, duties);
        assert_eq!(host.words(), b"a", "the sign-in's link change keeps words");
        let hello = hello_for(&host);
        let duties =
            host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::Record(hello) }));
        let duties = admit(&mut host, duties);
        let me = watch(&duties, &w::Subject::Me);
        empty(&host.report(Report::Key(Key::Enter)));
        assert_eq!(host.words(), b"a", "Enter before me keeps words");
        assert!(text(&mut host).contains("Still signing in"));
        let duties = delivery(&mut host, me, &w::DeliveryValue::Me(launchpad_me(false)));
        admit(&mut host, duties);
        assert_eq!(host.words(), b"a", "no project keeps words");
        assert!(text(&mut host).contains("❯ a"));
        let duties = delivery(&mut host, me, &w::DeliveryValue::Me(launchpad_me(true)));
        admit(&mut host, duties);
        assert_eq!(host.root.pages().address(), &Address::Home { project: 3 });
        assert_eq!(host.root.pages().chats().expect("Home composer").words.as_ref(), b"a");
        assert!(text(&mut host).contains("❯ a"));
        let duties = host.report(Report::Key(Key::Enter));
        let duties = admit(&mut host, duties);
        let asks = sent_ask(&duties);
        let [(_, w::Ask::Create(create))] = asks.as_slice() else {
            panic!("Enter after landing starts a chat: {asks:?}");
        };
        let mut reader = Reader::new(create.tasks());
        let bytes = skein_codec::piece::split(&mut reader).expect("task piece").expect("created task");
        let task = w::NewTask::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("new task");
        assert_eq!(create.project(), 3);
        assert_eq!(task.spec().words(), b"a");
    }
}

fn no_project_keys(words: &str) {
    let (mut host, _, _) = signed_host(false);
    key_words(&mut host, words);
    let written = host.state().box_written();
    let duties = host.report(Report::Key(if words == "/" { Key::Escape } else { Key::Enter }));
    empty(&duties);
    match words {
        "/theme" => {
            assert!(host.state().theme_list());
            host.frame();
            host.report(Report::Key(Key::Down));
            let duties = host.report(Report::Key(Key::Enter));
            assert!(matches!(duties.asked.as_slice(), [Request::Save { .. }]));
            empty(&host.report(Report::Heard(jig_client::Event::Stored)));
            assert_eq!(host.root.pages().look().theme, Theme::Light);
        }
        "/help" => {
            assert_eq!(host.root.pages().notes().last(), Some(&terminal::Note::Help));
            assert!(text(&mut host).contains("/quit"), "help must render its commands");
        }
        "/zz" => {
            assert_eq!(
                host.state().notice(),
                Some(&temper_client_tui::Notice::Unknown { words: b"/zz".as_slice().into() })
            );
            assert!(text(&mut host).contains("Unknown command /zz · /help lists them"));
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

#[test]
fn a_notice_stands_six_seconds_and_a_newer_replaces_it() {
    use temper_client_tui::Notice;
    let (mut host, _, _) = signed_host(true);
    let first = Notice::Unknown { words: b"/qiut".as_slice().into() };
    let end = Time::ZERO.saturating_add(Duration::from_secs(6));
    typed(&mut host, b"/qiut");
    empty(&give(&mut host, Event::Send));
    assert_eq!(host.state().notice(), Some(&first));
    assert_eq!(host.deadline(), skein_lib::earliest(root::next_deadline(&host.root), Some(end)));
    assert!(host.words().is_empty());
    assert!(host.root.pages().notes().is_empty());
    assert!(host.root.pages().listing().is_none());
    let frame = text(&mut host);
    let line = frame.lines().find(|line| line.contains("Unknown command /qiut")).expect("quiet notice");
    assert!(line.contains("● Live"), "notice and link share the quiet line");
    assert_eq!(frame.matches("Unknown command").count(), 1);
    host.frame();
    empty(&host.report(Report::Key(Key::Char('x'))));
    assert_eq!(host.state().notice(), Some(&first), "a key leaves the notice");
    typed(&mut host, b"");
    give(&mut host, Event::Send);
    assert_eq!(host.state().notice(), Some(&first), "a send leaves the notice");
    typed(&mut host, b"/new");
    give(&mut host, Event::Send);
    assert_eq!(host.state().notice(), Some(&first), "/new leaves the notice");
    host.clocks(Time::ZERO.saturating_add(Duration::from_secs(3)), Wall::EPOCH);
    host.report(Report::Due);
    assert_eq!(host.state().notice(), Some(&first), "a due before the end leaves it");
    typed(&mut host, b"/other");
    give(&mut host, Event::Send);
    let second = Notice::Unknown { words: b"/other".as_slice().into() };
    let later = Time::ZERO.saturating_add(Duration::from_secs(9));
    assert_eq!(host.state().notice(), Some(&second));
    assert_eq!(host.deadline(), skein_lib::earliest(root::next_deadline(&host.root), Some(later)));
    host.clocks(end, Wall::EPOCH);
    host.report(Report::Due);
    assert_eq!(host.state().notice(), Some(&second), "the replaced end cannot lower it");
    host.clocks(later, Wall::EPOCH);
    host.report(Report::Due);
    assert_eq!(host.state().notice(), None);
    assert_eq!(host.deadline(), root::next_deadline(&host.root));
    assert!(!text(&mut host).contains("Unknown command"));
    // With no background retry pending, the notice itself is the next deadline.
    let (mut quiet, _, _) = signed_host(false);
    typed(&mut quiet, b"/zz");
    give(&mut quiet, Event::Send);
    assert_eq!(quiet.deadline(), Some(end));
    for (words, seconds) in [(b"/chats 9".as_slice(), 6), (b"/zz".as_slice(), 7)] {
        let (mut host, _, _) = signed_host(true);
        typed(&mut host, words);
        give(&mut host, Event::Send);
        assert_eq!(host.state().notice(), Some(&Notice::Unknown { words: words.into() }));
        assert!(host.words().is_empty());
        assert!(host.root.pages().notes().is_empty());
        host.clocks(Time::ZERO.saturating_add(Duration::from_secs(seconds)), Wall::EPOCH);
        host.report(Report::Due);
        assert_eq!(host.state().notice(), None, "at and past the end");
    }
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
    goal::navigation();
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
                    scheme: Some(if light { Scheme::Light } else { Scheme::Dark }),
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
                host.frame();
                host.report(Report::Key(Key::Down));
                host.report(Report::Key(Key::Enter));
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

#[derive(Clone, Copy, Debug)]
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
            b"/draft Design an export".as_slice(),
            LaunchAt::Home,
            LaunchWant::Started {
                label: 8,
                words: b"Design an export\n\nWrite this up as a draft beside us.",
                from: Address::NewChat { draft: true },
            },
        ),
        (
            b"/change Add a CSV export".as_slice(),
            LaunchAt::Home,
            LaunchWant::Started { label: 8, words: b"Add a CSV export", from: Address::NewChat { draft: false } },
        ),
        (b"/draft", LaunchAt::Home, LaunchWant::Words(b"/draft ")),
        (b"/d", LaunchAt::Home, LaunchWant::Words(b"/draft ")),
        (b"/change", LaunchAt::Home, LaunchWant::Words(b"/change ")),
        (b"/c", LaunchAt::Home, LaunchWant::Words(b"/change ")),
        (b"/", LaunchAt::Home, LaunchWant::Words(b"/draft ")),
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
                let mut duties = admit(&mut host, duties);
                if sent_ask(&duties).is_empty() {
                    // Returning from New Chat can fill the exchange slots with page reads.
                    host.clocks(
                        host.env.now.saturating_add(host.env.limits.client.domain.backoff.first),
                        host.env.wall,
                    );
                    let more = host.report(Report::Due);
                    duties.asked.extend(admit(&mut host, more).asked);
                }
                let asks = sent_ask(&duties);
                let [(exchange, w::Ask::Create(create))] = asks.as_slice() else {
                    panic!("{at:?}, {words:?}: {asks:?}")
                };
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
            LaunchWant::Unknown => assert_eq!(
                host.state().notice(),
                Some(&temper_client_tui::Notice::Unknown { words: words.into() }),
                "{words:?}"
            ),
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
        let mut duties = admit(&mut host, duties);
        if !missing_template {
            if sent_ask(&duties).is_empty() {
                host.clocks(host.env.now.saturating_add(host.env.limits.client.domain.requests.first), host.env.wall);
                let more = host.report(Report::Due);
                duties.asked.extend(admit(&mut host, more).asked);
            }
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
        (Event::Own { command: Own::Draft }, b"/draft ".as_slice(), false, false, None),
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
    for (projects, chat, selected, linked, address) in [
        (false, false, false, false, None),
        (true, false, false, false, Some(b"http://127.0.0.1:8367/projects/3".as_slice())),
        (true, true, false, false, Some(b"http://127.0.0.1:8367/chats/9".as_slice())),
        (true, false, true, false, Some(b"http://127.0.0.1:8367/chats/9".as_slice())),
        (true, true, false, true, Some(b"http://127.0.0.1:8367/tasks/25".as_slice())),
    ] {
        let mut host = if linked {
            goal::host(true, false)
        } else if projects {
            left_off_host()
        } else {
            signed_host(false).0
        };
        if chat && !linked {
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
        host.typed(b"o".as_slice().into());
        assert_eq!(host.words(), expected, "by_key={by_key}, taken={taken}");
        assert!(host.state().walk().is_none(), "typing ends the prompt");
        assert!(host.state().field().is_empty(), "walking keys belong only to the box");
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
    pane_events();
    reply_events();
    goal::events();
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
                    host.state().left_off() == Some(NEWEST)
                        && event.is_none()
                        && text(&mut host).lines().any(|line| line.starts_with("❯ /change"))
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
                    && host.state().notice()
                        == Some(&temper_client_tui::Notice::Unknown { words: b"/zz".as_slice().into() })
                    && host.root.pages().chats().is_some_and(|chat| chat.words.is_empty())
                    && host.state().box_written() > before
            }
            "unknown send" | "unknown went" => {
                host.tell(temper_client_tui::Told::Notice {
                    notice: Some(temper_client_tui::Notice::Unknown { words: b"/zz".as_slice().into() }),
                });
                if row == "unknown send" {
                    give(&mut host, Event::Send);
                } else {
                    host.out.push(root::Request::Show { address: Address::Chat { task: 9 }, replace: false });
                    host.report(Report::Due);
                }
                host.state().notice().is_some()
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
                            | (w::Read::Notes(_), "notes")
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

fn reply_events() {
    for (case, typed_words, keys, message, typed_aside, aside, kept) in [
        ("menu", b"".as_slice(), &[Key::Enter][..], Some(b"Move it at once".as_slice()), false, false, b"".as_slice()),
        ("menu two", b"", &[Key::Down, Key::Enter][..], Some(b"Side by side"), false, false, b""),
        ("menu else", b"", &[Key::Down, Key::Down, Key::Enter][..], None, false, true, b""),
        ("number one", b"1", &[Key::Enter][..], Some(b"Move it at once"), false, false, b""),
        ("number two", b"2", &[Key::Enter][..], Some(b"Side by side"), false, false, b""),
        ("number else", b"3", &[Key::Enter][..], None, false, true, b""),
        ("words", b"my own words", &[Key::Enter][..], Some(b"my own words"), true, true, b""),
        ("not a number", b"1 ", &[Key::Enter][..], Some(b"1 "), true, true, b""),
        ("out of range", b"4", &[Key::Enter][..], Some(b"4"), true, true, b""),
        ("leading zero", b"01", &[Key::Enter][..], Some(b"01"), true, true, b""),
        ("escape", b"", &[Key::Escape][..], None, false, true, b""),
        ("escape number", b"2", &[Key::Escape][..], None, false, true, b"2"),
        ("escaped number sent", b"2", &[Key::Escape, Key::Enter][..], Some(b"2"), false, true, b""),
        ("erased words", b"x", &[Key::Backspace][..], None, true, true, b""),
        ("erased number", b"2", &[Key::Backspace, Key::Enter][..], Some(b"Move it at once"), false, false, b""),
        ("edited number", b"2x", &[Key::Backspace, Key::Enter][..], Some(b"2"), true, true, b""),
    ] {
        let mut host = reply_host();
        host.report(Report::Key(Key::Paste(typed_words.into())));
        host.frame();
        assert_eq!(host.words(), typed_words, "{case}: typed words before Enter");
        let key_event = host.event_of(&Key::Enter);
        let number_event = temper_client_tui::reply_number(&host.view, typed_words);
        let before = host.state().box_written();
        assert_eq!(host.state().reply_aside(), typed_aside.then_some(REPLY_TURN), "{case}: aside on typing");
        let mut duties = Duties::default();
        for key in keys {
            duties = host.report(Report::Key(key.clone()));
            host.frame();
        }
        let duties = stored(&mut host, duties);
        let asks = sent_ask(&duties);
        if let Some(words) = message {
            let [(_, w::Ask::Message(sent))] = asks.as_slice() else {
                panic!("{case}: {asks:?}; key={key_event:?}; number={number_event:?}; words={:?}", host.words())
            };
            assert_eq!((sent.task(), sent.words()), (7, words), "{case}: choice words, not the number");
            assert!(host.state().box_written() > before, "{case}: box write advances");
        } else {
            assert!(asks.is_empty(), "{case}: no send");
        }
        assert_eq!(host.state().reply_aside(), aside.then_some(REPLY_TURN), "{case}");
        assert_eq!(host.words(), kept, "{case}: composer");
        if aside {
            host.frame();
            assert_eq!(temper_client_tui::reply_number(&host.view, b"1"), None, "{case}: set aside");
        }
    }
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
    let entries = pieces(&[writer.finish()]);
    let mut inbox = records::owner_inbox_record().into_parts();
    inbox.party = 7;
    if !chat {
        inbox.needs = entries.clone();
        inbox.cached_needs = 1;
        inbox.updates = Box::default();
    }
    let duties =
        delivery(&mut host, inbox_watch, &w::DeliveryValue::Inbox(w::Inbox::new(&w::CEILINGS, inbox).expect("inbox")));
    let duties = admit_watches(&mut host, duties);
    let duties = if chat {
        let query = read_exchange(&duties, "inbox");
        let page = w::InboxPage::new(
            &w::CEILINGS,
            w::InboxPageParts { selection: None, cut: false, rows: entries, before: None, after: None },
        )
        .expect("inbox page");
        let duties = read_reply(&mut host, query, &w::ReadValue::Inbox(page));
        admit_watches(&mut host, duties)
    } else {
        duties
    };
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

// Watch terminals free credit for the fixture's reads, whose values it supplies separately.
fn admit_watches(host: &mut Host, mut duties: Duties) -> Duties {
    let mut pending = std::collections::VecDeque::from(core::mem::take(&mut duties.asked));
    while let Some(request) = pending.pop_front() {
        if matches!(
            &request,
            Request::Exchange { message: Message::Watch(_) | Message::Unwatch(_), .. }
                | Request::Save { .. }
                | Request::Erase { .. }
        ) && let Some(report) = background_reply(&request)
        {
            pending.extend(host.report(report).asked);
        }
        duties.asked.push(request);
    }
    duties
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
    let read = if let Message::Read(bytes) = message
        && let owned = w::Owned::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("read envelope")
        && owned.owner() != &w::Owner::Jig
        && let envelope = w::FamilyRecord::decode(&w::CEILINGS, &mut Reader::new(owned.record())).expect("family read")
        && matches!(temper_client_protocol_wire::decode_reading(envelope.record(), &temper_client_protocol_wire::v1::CEILINGS).as_ref().map(temper_client_protocol_wire::v1::Reading::read), Some(temper_client_protocol_wire::v1::Read::Setting(setting)) if setting.kind() == 7)
    {
        use temper_client_protocol_wire::{self as wire, v1 as f};
        let record = wire::setting_answer(
            &f::SettingAnswer::new(&f::CEILINGS, f::SettingAnswerParts { value: None }).expect("stock ways"),
        );
        w::ReadAnswer::Value(
            w::Owned::new(
                &w::CEILINGS,
                w::OwnedParts { owner: owned.owner().clone(), version: wire::CLIENT_VERSION, record },
            )
            .expect("ways answer"),
        )
    } else {
        w::ReadAnswer::None(
            w::ReadAnswerNone::new(&w::CEILINGS, w::ReadAnswerNoneParts { value: w::ReadNone::Unknown })
                .expect("background read"),
        )
    };
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
    Stop,
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
        ("front idle", true, false, vec![(close(), FrontWant::Stop)]),
        ("ask twice", true, true, vec![(close(), asked()), (close(), FrontWant::Stop)]),
        ("quit", true, true, vec![(FrontAct::Send(b"/quit"), asked()), (FrontAct::Send(b"/quit"), FrontWant::Stop)]),
        ("exit alias", true, true, vec![(FrontAct::Send(b"/exit"), asked())]),
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
        ("stop", true, true, vec![(close(), asked()), (FrontAct::Event(Event::Stop), FrontWant::Stop)]),
    ];
    let gave_up = BoxLine::GaveUp { step: b"forge".as_slice().into() };
    rows.push((
        "given up",
        true,
        true,
        vec![(FrontAct::Line(gave_up.clone()), FrontWant::Line(gave_up)), (close(), FrontWant::Stop)],
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
            host.report(Report::Stage(BoxStage::Running));
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
                FrontWant::Quit { close, asking } => {
                    duties.close == close && duties.stop.is_none() && host.state().quitting() == asking
                }
                FrontWant::Stop => {
                    duties.stop == Some(crate::host::Stop::First)
                        && !duties.close
                        && host.state().box_stage() == &BoxStage::Stopping
                        && !host.state().quitting()
                }
                FrontWant::Log { shown, unknown } => {
                    duties.asked.is_empty()
                        && !duties.close
                        && host.state().log_shown() == shown
                        && host.state().notice()
                            == unknown.map(|words| temper_client_tui::Notice::Unknown { words: words.into() }).as_ref()
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

#[test]
fn front_stages_wait_for_an_asked_end_and_quits_stop_twice() {
    use crate::host::Stop;
    let ended = BoxStage::Ended { end: BoxEnd::Exited { code: 0 } };
    for stage in [
        BoxStage::Finding,
        BoxStage::Fetching { done: 7, total: None },
        BoxStage::Fetching { done: 7, total: Some(10) },
        BoxStage::Starting,
        BoxStage::Running,
        BoxStage::Stopping,
        BoxStage::Failed { why: b"cannot fetch".as_slice().into() },
        ended.clone(),
        BoxStage::Ended { end: BoxEnd::Killed { signal: 9 } },
    ] {
        for command in [false, true] {
            let mut host = host();
            let duties = host.report(Report::Start(Start {
                front: true,
                size: Size { columns: 80, rows: 24 },
                scheme: Some(Scheme::Dark),
                background: None,
                address: b"127.0.0.1:8367".as_slice().into(),
                local: true,
                boxed: false,
                kept: Vec::new(),
                credential: None,
                note: None,
            }));
            assert!(duties.local && duties.asked.is_empty());
            assert!(host.state().front());
            assert_eq!(host.state().box_stage(), &BoxStage::Finding);
            let duties = host.report(Report::Stage(stage.clone()));
            assert!(!duties.close, "an unasked end stays open");
            assert_eq!(host.state().box_stage(), &stage);
            let quit = |host: &mut Host| {
                if command {
                    typed(host, b"/quit");
                    give(host, Event::Send)
                } else {
                    host.report(Report::Key(Key::Control(b'c')))
                }
            };
            let first = quit(&mut host);
            if matches!(stage, BoxStage::Ended { .. }) {
                assert!(first.close && first.stop.is_none());
                continue;
            }
            assert_eq!(
                first.stop,
                Some(if stage == BoxStage::Stopping { Stop::Second } else { Stop::First }),
                "{stage:?}, /quit={command}"
            );
            assert!(!first.close);
            // Failed is terminal for the displayed stage, but the host still waits on its stop duty.
            if !matches!(stage, BoxStage::Failed { .. }) {
                assert_eq!(host.state().box_stage(), &BoxStage::Stopping);
            }
            let second = quit(&mut host);
            assert_eq!(second.stop, Some(Stop::Second));
            assert!(!second.close);
            assert!(host.report(Report::Stage(ended.clone())).close);
        }
    }
}

fn working_chip(task: u64, title: &[u8]) -> w::Chip {
    chat_chip_stage(task, title, w::RunStage::Due)
}

pub(crate) fn home_host(running: bool) -> Host {
    home_with_brief(&home_tasks(running), false)
}

fn one_chat_host(brief: bool) -> Host {
    let mut tasks = home_tasks(false).into_parts();
    let mut reader = Reader::new(&tasks.rows);
    skein_codec::piece::split(&mut reader).expect("older chat");
    let newest = skein_codec::piece::split(&mut reader).expect("newest chat").expect("one chat");
    tasks.rows = pieces(&[newest]);
    home_with_brief(&w::Tasks::new(&w::CEILINGS, tasks).expect("one chat"), brief)
}

fn home_with_brief(tasks: &w::Tasks, brief: bool) -> Host {
    let (mut host, _, mut duties) = signed_host(true);
    // The first wave can fill the exchange slots; give its watch retry a turn.
    host.clocks(Time::from_nanos(host.env.limits.client.domain.backoff.first.as_nanos()), Wall::EPOCH);
    let again = host.report(Report::Due);
    duties.asked.extend(admit(&mut host, again).asked);
    let subject = home_subject(&duties);
    // Navigation replaces the rail's watch; rows go to the current page's watch.
    for _ in 0..host.env.limits.client.domain.watches {
        let watched = watch(&duties, &subject);
        let arrived = delivery(&mut host, watched, &w::DeliveryValue::Tasks(tasks.clone()));
        duties.asked.extend(admit(&mut host, arrived).asked);
        let chats = host.root.pages().chats().expect("launchpad chats");
        if chats.live(host.root.client()).is_some() {
            break;
        }
        let due = root::next_deadline(&host.root).expect("chat watch retry");
        host.clocks(due, Wall::EPOCH);
        let again = host.report(Report::Due);
        duties.asked.extend(admit(&mut host, again).asked);
    }
    let w::Subject::Tasks(subject) = subject else { panic!("chat tasks subject") };
    let mut subject = subject.into_parts();
    let mut query = subject.query.into_parts();
    query.project = None;
    subject.query = w::TaskQuery::new(&w::CEILINGS, query).expect("across query");
    let subject = w::Subject::Tasks(w::SubjectTasks::new(&w::CEILINGS, subject).expect("across subject"));
    for _ in 0..host.env.limits.client.domain.watches {
        if maybe_watch(&duties, &subject).is_some() {
            break;
        }
        let due = root::next_deadline(&host.root).expect("across chat watch retry");
        host.clocks(due, Wall::EPOCH);
        let again = host.report(Report::Due);
        duties.asked.extend(admit(&mut host, again).asked);
    }
    let across = watch(&duties, &subject);
    let arrived = delivery(&mut host, across, &w::DeliveryValue::Tasks(tasks.clone()));
    duties.asked.extend(admit(&mut host, arrived).asked);
    let window = host.root.pages().inbox().expect("launchpad inbox").window;
    let inbox_watch = watch(
        &duties,
        &w::Subject::Inbox(w::SubjectInbox::new(&w::CEILINGS, w::SubjectInboxParts { window }).expect("inbox subject")),
    );
    let mut inbox = records::owner_inbox_record().into_parts();
    inbox.needs = Box::default();
    inbox.updates = Box::default();
    inbox.cached_needs = 0;
    let arrived = delivery(
        &mut host,
        inbox_watch,
        &w::DeliveryValue::Inbox(w::Inbox::new(&w::CEILINGS, inbox).expect("empty inbox")),
    );
    duties.asked.extend(admit(&mut host, arrived).asked);
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

#[test]
fn the_terminal_control_p_cycles_the_pane() {
    use root::pages::chat::document::Mode;
    use skein_ui_screen_terminal::Decoder;

    let mut decoder = Decoder::new(1);
    let mut keys = skein_lib::Queue::with_capacity(1);
    decoder.read(b"\x10", &mut keys);
    let key = keys.pop().expect("the terminal's Ctrl-P");
    let mut host = reply_host_draft(true);
    give(&mut host, Event::Act { act: Act::Mode { mode: Mode::Split } });
    for mode in [Mode::Draft, Mode::Chat, Mode::Split] {
        host.frame();
        assert_eq!(host.event_of(&key), Some(Event::Act { act: Act::Mode { mode } }));
        host.report(Report::Key(key.clone()));
        assert_eq!(host.root.pages().chat().expect("chat with a draft").mode, mode);
    }
}

fn pane_events() {
    use root::pages::chat::document::Mode;
    for (name, draft, columns, mode, words, next) in [
        ("no pane", false, 80, Mode::Split, "", None),
        ("folded", true, 80, Mode::Chat, "", Some(Mode::Split)),
        ("writing", true, 80, Mode::Split, "", Some(Mode::Draft)),
        ("typed words", true, 80, Mode::Split, "my words", Some(Mode::Draft)),
        ("list", true, 80, Mode::Draft, "/", Some(Mode::Chat)),
        ("narrow", true, 79, Mode::Chat, "/", Some(Mode::Draft)),
    ] {
        let mut host = reply_host_draft(draft && name != "writing");
        if name == "writing" {
            give(&mut host, Event::Act { act: Act::AddDraft });
        }
        give(&mut host, Event::Act { act: Act::Mode { mode } });
        host.report(Report::Resize(Size { columns, rows: 24 }));
        typed(&mut host, words.as_bytes());
        host.frame();
        let want = next.map(|mode| Event::Act { act: Act::Mode { mode } });
        assert_eq!(host.event_of(&Key::Control(b'p')), want, "{name}");
        host.report(Report::Key(Key::Control(b'p')));
        assert_eq!(host.root.pages().chat().unwrap().mode, next.unwrap_or(mode), "{name}");
        assert_eq!(host.words(), words.as_bytes(), "{name}: composer unchanged");
    }
    for (before, after, columns, all) in [
        (Mode::Chat, Mode::Draft, 80, true),
        (Mode::Split, Mode::Draft, 80, true),
        (Mode::Draft, Mode::Chat, 80, false),
        (Mode::Draft, Mode::Split, 80, false),
        (Mode::Chat, Mode::Split, 79, true),
    ] {
        let mut host = reply_host_draft(true);
        host.report(Report::Resize(Size { columns, rows: 24 }));
        give(&mut host, Event::Act { act: Act::Mode { mode: before } });
        host.frame();
        host.focus.scroll = 10;
        host.focus.follow = all;
        give(&mut host, Event::Act { act: Act::Mode { mode: after } });
        host.frame();
        assert_eq!(host.focus.follow, !all, "{before:?} -> {after:?}");
        if all {
            assert_eq!(host.focus.scroll, 0, "draft begins at its head");
        }
    }
    let mut host = reply_host_draft(true);
    host.report(Report::Resize(Size { columns: 80, rows: 24 }));
    host.frame();
    host.focus.scroll = 10;
    host.focus.follow = true;
    host.report(Report::Resize(Size { columns: 79, rows: 24 }));
    host.frame();
    assert_eq!(host.focus.scroll, 0, "resizing half to all starts at the head");
    assert!(!host.focus.follow);
    host.report(Report::Resize(Size { columns: 80, rows: 24 }));
    host.frame();
    assert!(host.focus.follow, "resizing back follows the conversation");
}

#[test]
fn start_gives_the_look_then_launchpad_once_but_a_theme_only_gives_the_look() {
    use core::mem::discriminant;
    let mut host = host();
    start(&mut host, None);
    let look = terminal::Look { terminal: true, ..terminal::Look::default() };
    assert_eq!(
        host.events.get(..3),
        Some(
            [
                discriminant(&root::Event::Start {
                    address: Address::Inbox { filter: root::address::Filter::ALL },
                    kept: List::with_capacity(0),
                    with: None,
                }),
                discriminant(&root::Event::LookKept { look }),
                discriminant(&root::Event::Launchpad),
            ]
            .as_slice()
        )
    );
    assert_eq!(host.events.iter().filter(|event| **event == discriminant(&root::Event::Launchpad)).count(), 1);
    host.events.clear();
    host.keep_theme(temper_client_tui::theme::ThemeChoice::Light, &mut Duties::default());
    assert_eq!(host.events, [discriminant(&root::Event::LookKept { look })]);
}

#[test]
fn chats_one_and_enter_open_the_newer_chat_of_shop() {
    let mut tasks = home_tasks(false).into_parts();
    let mut reader = Reader::new(&tasks.rows);
    let mut rows = Vec::new();
    while let Some(bytes) = skein_codec::piece::split(&mut reader).expect("task piece") {
        let mut row = w::TaskLine::decode(&w::CEILINGS, &mut Reader::new(&bytes)).expect("task line").into_parts();
        if row.last_activity == 2 {
            row.project = 5;
        }
        let row = w::TaskLine::new(&w::CEILINGS, row).expect("project's row");
        let mut writer = Writer::new(usize::try_from(row.measure()).expect("row size"));
        row.encode(&mut writer).expect("row fits");
        rows.push(writer.finish());
    }
    tasks.rows = pieces(&rows);
    let tasks = w::Tasks::new(&w::CEILINGS, tasks).expect("two projects' chats");
    for numbered in [false, true] {
        let mut host = home_with_brief(&tasks, false);
        if numbered {
            typed(&mut host, b"/chats 1");
        } else {
            host.frame();
            host.report(Report::Key(Key::Down));
            host.frame();
            assert_eq!(host.state().left_off(), Some(NEWEST));
        }
        host.report(Report::Key(Key::Enter));
        assert_eq!(host.root.pages().address(), &Address::Chat { task: NEWEST });
    }
}
