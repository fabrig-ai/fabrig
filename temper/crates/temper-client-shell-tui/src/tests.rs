use crate::{
    host::{Duties, Host, Report, Settings, Start},
    limits,
};
use jig_client::{Message, Request};
use jig_client_protocol_wire::v19 as w;
use jig_client_test_records as records;
use skein_lib::{Duration, List, Reader, Secret, Time, Wall, Writer};
use skein_ui_screen::Size;
use skein_ui_screen_terminal::Key;
use temper_client_domain::{
    self as root, Address, Place,
    pages::{
        Act,
        terminal::{self, Command, Font, Theme},
    },
};
use temper_client_tui::{Event, Local, Note, Scheme};

fn host() -> Host {
    Host::new(&Settings { limits: limits::tui() }, 7, Time::ZERO, Wall::EPOCH)
}

fn start(host: &mut Host, credential: Option<Secret>) -> Duties {
    host.report(Report::Start(Start {
        size: Size { columns: 80, rows: 24 },
        scheme: Scheme::Dark,
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
            size: Size { columns: 80, rows: 24 },
            scheme: Scheme::Dark,
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
}

#[test]
fn the_opener_gets_one_argument_and_no_terminal_descriptors() {
    use crate::opener::{self, Opener};
    use skein_io::kernel::{Fd, Pipe, Way};
    for (program, address, env) in [
        ("/usr/bin/xdg-open", "http://127.0.0.1:8367/terminal?code=WDJB-MJHT", vec!["DISPLAY=:1", "PATH=/usr/bin"]),
        ("/usr/bin/open", "http://127.0.0.1:9000/terminal?code=BCDF-GHJK", vec!["HOME=/person"]),
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
    Wait,
    Keep,
    Pasted,
    Taking,
    Until,
    Ended,
    Quiet,
}
struct LocalRow {
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
    use LocalWant::{Ask, Ended, Keep, Pasted, Quiet, Read, Send, Taking, Until, Wait};
    let found = || Found(Some(local_link("127.0.0.1:8367", LOCAL_SECRET)));
    let fresh = || Found(Some(local_link("127.0.0.1:8367", FRESH_SECRET)));
    let row = |name, steps| LocalRow { name, local: true, note: None, held: false, first: Read, steps };
    let mut rows = vec![
        row("fresh", vec![]),
        LocalRow { name: "disabled", local: false, note: None, held: false, first: Ask, steps: vec![] },
        LocalRow {
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
    for (name, wall, want) in [
        ("before slack", 239_999_999_999, Ended),
        ("at slack", 240_000_000_000, Read),
        ("at expiry", UNTIL, Read),
        ("after expiry", UNTIL + 1, Read),
    ] {
        rows.push(LocalRow {
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
        name: "no held note",
        local: true,
        note: None,
        held: true,
        first: Quiet,
        steps: vec![(End(UNTIL), Read)],
    });
    rows.push(LocalRow {
        name: "held signed out",
        local: true,
        note: Some(Note::SignedOut),
        held: true,
        first: Quiet,
        steps: vec![(End(UNTIL), Ask)],
    });
    rows.push(LocalRow {
        name: "paste after sign out",
        local: true,
        note: Some(Note::SignedOut),
        held: false,
        first: Ask,
        steps: vec![(Paste, Pasted), (Answer(signed_in_answer()), Keep), (Connect, Until)],
    });
    rows.push(LocalRow {
        name: "approval after sign out",
        local: true,
        note: Some(Note::SignedOut),
        held: false,
        first: Ask,
        steps: vec![(Approve, Taking), (Answer(signed_in_answer()), Keep), (Connect, Until), (Due(SECOND), Quiet)],
    });
    rows.push(LocalRow {
        name: "same expiry already noted",
        local: true,
        note: Some(Note::Until(Wall::from_nanos(UNTIL))),
        held: false,
        first: Read,
        steps: vec![(found(), Send), (Answer(signed_in_answer()), Keep), (Connect, Quiet)],
    });
    rows.push(LocalRow {
        name: "held has no expiry",
        local: true,
        note: None,
        held: true,
        first: Quiet,
        steps: vec![(Connect, Quiet)],
    });
    let mut failures = Vec::new();
    for row in rows {
        local_row(row, &mut failures);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn local_row(row: LocalRow, failures: &mut Vec<String>) {
    let mut host = host();
    let duties = host.report(Report::Start(Start {
        size: Size { columns: 80, rows: 24 },
        scheme: Scheme::Dark,
        address: b"127.0.0.1:8367".as_slice().into(),
        local: row.local,
        kept: Vec::new(),
        credential: row.held.then(|| Secret::new(Box::new([7; 40]))),
        note: row.note,
    }));
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
            matches!(request, Request::Exchange { message: Message::SignInLink(_), credential: None, .. })
        })
        .count();
    let (read, ask, send, note, keep) = match want {
        LocalWant::Read => (true, 0, 0, None, None),
        LocalWant::Ask => (false, 1, 0, None, None),
        LocalWant::Send | LocalWant::Pasted => (false, 0, 1, None, None),
        LocalWant::Wait | LocalWant::Quiet | LocalWant::Taking => (false, 0, 0, None, None),
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
    if keep.is_some() && duties.keep != keep {
        failures.push(format!("{name}: credential duty"));
    }
    if matches!(want, LocalWant::Read | LocalWant::Wait) && host.state().local() != Local::Signing {
        failures.push(format!("{name}: local signing screen"));
    }
    let frame = host.frame();
    let print = frame.print();
    let text = String::from_utf8(frame.text().into_vec()).expect("UTF-8 frame");
    if matches!(want, LocalWant::Read) && !text.contains("Connecting to 127.0.0.1:8367") {
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

fn launchpad_me(projects: bool) -> w::Me {
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
        let templates = pieces(&[writer.finish()]);
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
            let mut writer = Writer::new(usize::try_from(project.measure()).expect("project size"));
            project.encode(&mut writer).expect("project fits");
            projects.push(writer.finish());
        }
        me.projects = pieces(&projects);
    }
    w::Me::new(&w::CEILINGS, me).expect("Me")
}

fn watch(duties: &Duties, subject: &w::Subject) -> u64 {
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
    host.report(Report::Heard(jig_client::Event::Streamed {
        message: jig_client::Streamed::Record(stream_bytes(&event)),
    }))
}

fn signed_host(projects: bool) -> (Host, u64, Duties) {
    let mut host = host();
    start(&mut host, Some(Secret::new(Box::new([7; 40]))));
    let hello = hello_for(&host);
    let duties =
        host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::Record(hello) }));
    let duties = admit(&mut host, duties);
    let me = watch(&duties, &w::Subject::Me);
    let duties = delivery(&mut host, me, &w::DeliveryValue::Me(launchpad_me(projects)));
    let duties = admit(&mut host, duties);
    (host, me, duties)
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
        "dark look",
        "light look",
        "theme",
        "created chat",
        "show chat",
        "show ask",
        "show without project",
        "new",
        "exit",
    ] {
        let (mut host, me, _) = signed_host(row != "project appears");
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
            "dark look" | "light look" => {
                let mut host = self::host();
                let light = row == "light look";
                host.report(Report::Start(Start {
                    size: Size { columns: 80, rows: 24 },
                    scheme: if light { Scheme::Light } else { Scheme::Dark },
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

#[test]
#[expect(clippy::too_many_lines, reason = "one table holds the host's TUI event translations")]
fn each_event_becomes_its_acts() {
    let mut failures = Vec::new();
    for row in [
        "home words",
        "chat words",
        "chat send",
        "create",
        "unknown",
        "unknown send",
        "unknown went",
        "no project",
        "help",
        "dismiss",
        "aside",
        "reply words",
        "reply choice",
        "act",
        "size",
    ] {
        let (mut host, _, _) = signed_host(row != "no project");
        let passed = match row {
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
                let duties = give(&mut host, Event::Send);
                duties.asked.is_empty()
                    && host.state().unknown() == Some(b"/zz".as_slice())
                    && host.root.pages().chats().is_some_and(|chat| chat.words.is_empty())
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
                give(&mut host, Event::Command { command: Command::Help });
                host.root.pages().notes().last() == Some(&terminal::Note::Help)
            }
            "dismiss" => {
                typed(&mut host, b"/");
                let before = host.state().box_written();
                give(&mut host, Event::Dismiss);
                host.root.pages().chats().is_some_and(|chats| chats.words.is_empty())
                    && host.state().box_written() > before
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
                    if matches!((&read, wanted), (w::Read::Inbox(_), "inbox") | (w::Read::Question(_), "question")) {
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

#[expect(clippy::too_many_lines, reason = "one wire fixture delivers a waiting chat and its answerable question")]
fn waiting_host() -> Host {
    let (mut host, _, mut watches) = signed_host(true);
    // The fixture supplies the terminal look so event rows isolate event translation.
    host.step(root::Event::LookKept { look: terminal::Look { terminal: true, ..terminal::Look::default() } });
    host.step(root::Event::Went { place: Place::Address(Address::Home { project: 3 }) });
    watches.asked.extend(host.report(Report::Due).asked);
    host.step(root::Event::Went { place: Place::Address(Address::Chat { task: 7 }) });
    let duties = host.report(Report::Due);
    let duties = admit(&mut host, duties);
    let task_watch = watch(
        &duties,
        &w::Subject::Task(w::SubjectTask::new(&w::CEILINGS, w::SubjectTaskParts { task: 7 }).expect("task subject")),
    );
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
            w::ChipPhaseActive::new(&w::CEILINGS, w::ChipPhaseActiveParts { value: w::RunStage::Idle }).expect("idle"),
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
    let duties = delivery(
        &mut host,
        task_watch,
        &w::DeliveryValue::Task(Box::new(w::Task::new(&w::CEILINGS, task).expect("task"))),
    );
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
    assert!(
        temper_client_view::shared::question(&host.root, host.view.words().view()).is_some(),
        "fixture's answerable question"
    );
    host
}

fn admit(host: &mut Host, mut duties: Duties) -> Duties {
    let mut pending = std::collections::VecDeque::from(core::mem::take(&mut duties.asked));
    while let Some(request) = pending.pop_front() {
        if let Request::Exchange { exchange, message: Message::Watch(_), .. } = &request {
            let answer = w::OpenAnswer::Admitted;
            let mut writer = Writer::new(usize::try_from(answer.measure()).expect("admission size"));
            answer.encode(&mut writer).expect("admission fits");
            pending.extend(reply(host, *exchange, writer.finish()).asked);
        } else if let Request::Exchange { exchange, message: Message::Read(_), .. } = &request {
            let answer = w::ReadAnswer::None(
                w::ReadAnswerNone::new(&w::CEILINGS, w::ReadAnswerNoneParts { value: w::ReadNone::Unknown })
                    .expect("background read"),
            );
            let mut writer = Writer::new(usize::try_from(answer.measure()).expect("answer size"));
            answer.encode(&mut writer).expect("answer fits");
            pending.extend(reply(host, *exchange, writer.finish()).asked);
        } else if let Request::Exchange { exchange, message: Message::Unwatch(_), .. } = &request {
            let answer = w::CloseAnswer::Closed;
            let mut writer = Writer::new(usize::try_from(answer.measure()).expect("close size"));
            answer.encode(&mut writer).expect("close fits");
            pending.extend(reply(host, *exchange, writer.finish()).asked);
        } else if matches!(&request, Request::Save { .. } | Request::Erase { .. }) {
            pending.extend(host.report(Report::Heard(jig_client::Event::Stored)).asked);
        }
        duties.asked.push(request);
    }
    duties
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
    let answer = w::Final::Done(w::Done::new(&w::CEILINGS, w::DoneParts { outcome }).expect("done"));
    let mut writer = Writer::new(usize::try_from(answer.measure()).expect("final size"));
    answer.encode(&mut writer).expect("final fits");
    let answer = w::KeyedAnswer::FinalAnswer(
        w::Owned::new(
            &w::CEILINGS,
            w::OwnedParts { owner: w::Owner::Jig, version: jig_client_protocol_wire::VERSION, record: writer.finish() },
        )
        .expect("final document"),
    );
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
