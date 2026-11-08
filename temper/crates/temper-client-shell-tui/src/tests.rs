use crate::{
    host::{Duties, Host, Report, Settings, Start},
    limits,
};
use jig_client::{Message, Request};
use jig_client_protocol_wire::v19 as w;
use skein_lib::{Duration, List, Reader, Secret, Time, Wall, Writer};
use skein_ui_screen::Size;
use skein_ui_screen_terminal::Key;
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
    let settings = limits::terminal(&limits, Depth::Indexed);
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
