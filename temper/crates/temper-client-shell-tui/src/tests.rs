use crate::{
    host::{Duties, Host, Report, Settings, Start},
    limits,
};
use jig_client::{Message, Request};
use skein_lib::{Secret, Time, Wall};
use skein_ui_screen::Size;
use skein_ui_screen_terminal::Key;
use temper_client_tui::{Event, Scheme};

fn host() -> Host {
    Host::new(&Settings { limits: limits::tui() }, 7, Time::ZERO, Wall::EPOCH)
}

fn start(host: &mut Host, credential: Option<Secret>) -> Duties {
    host.report(Report::Start(Start {
        size: Size { columns: 80, rows: 24 },
        scheme: Scheme::Dark,
        address: b"127.0.0.1:8367".as_slice().into(),
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
        empty(&start(&mut host, None));
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
            assert!(duties.asked.is_empty());
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
    empty(&start(&mut host, None));
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
    let exchanges = duties.asked.iter().filter_map(|request| {
        if let Request::Exchange { exchange, message: Message::ApprovalAsk(_), credential: None } = request {
            Some(*exchange)
        } else {
            None
        }
    }).collect::<Vec<_>>();
    assert_eq!(exchanges.len(), 1, "one approval ask");
    exchanges[0]
}

fn reply(host: &mut Host, exchange: skein_lib::Token, message: Box<[u8]>) -> Duties {
    host.report(Report::Heard(jig_client::Event::Reply { exchange, message }))
}

fn approval_bytes(code: &[u8], tail: &[u8]) -> Box<[u8]> {
    use jig_client_protocol_wire::v19 as w;
    let record = w::ApprovalAskAnswer::Asked(w::ApprovalAsked::new(&w::CEILINGS,
        w::ApprovalAskedParts { secret: Secret::new(Box::new([7; 32])), code: *b"WDJBMJHT", expires: 600_000_000_000 }
    ).expect("test approval"));
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
        if held {
            assert!(!duties.asked.iter().any(|request| matches!(request,
                Request::Exchange { message: Message::ApprovalAsk(_), .. })));
            let duties = host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::SignedOut }));
            ask(&duties);
        } else {
            ask(&duties);
        }
        assert!(host.report(Report::Resize(Size { columns: 80, rows: 24 })).asked.is_empty());
        assert!(host.report(Report::Key(Key::Control(b'c'))).close);
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
            size: Size { columns: 80, rows: 24 }, scheme: Scheme::Dark,
            address: address.as_bytes().into(), kept: Vec::new(), credential: None,
        }));
        let exchange = ask(&duties);
        let duties = reply(&mut host, exchange, approval_bytes(code.as_bytes(), tail.as_bytes()));
        let expected = opens.then(|| format!("http://{address}/terminal?code=WDJB-MJHT").into_bytes().into_boxed_slice());
        assert_eq!(duties.open, expected, "{address} {code} {tail}");
        assert!(host.report(Report::Resize(Size { columns: 80, rows: 24 })).open.is_none());
    }
}

#[test]
fn approval_enter_replaces_only_an_ended_or_pasted_take() {
    use jig_client_protocol_wire::v19 as w;
    for answer in [w::ApprovalSignInAnswer::Refused, w::ApprovalSignInAnswer::Expired,
        w::ApprovalSignInAnswer::Used, w::ApprovalSignInAnswer::Unknown,
        w::ApprovalSignInAnswer::Full(w::ApprovalFull::new(&w::CEILINGS,
            w::ApprovalFullParts { sign_ins: 2, earliest: 400, until: 300 }).expect("full")),
        w::ApprovalSignInAnswer::Waiting] {
        let waiting = answer == w::ApprovalSignInAnswer::Waiting;
        let mut host = host();
        let exchange = ask(&start(&mut host, None));
        let duties = reply(&mut host, exchange, approval_bytes(b"WDJBMJHT", b""));
        let take = duties.asked.iter().find_map(|request| match request {
            Request::Exchange { exchange, message: Message::SignInApproval(_), .. } => Some(*exchange),
            _ => None,
        }).expect("approval take");
        let mut writer = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("test length"));
        answer.encode(&mut writer).expect("take answer");
        reply(&mut host, take, writer.finish());
        host.frame();
        let duties = host.report(Report::Key(Key::Enter));
        if waiting {
            assert!(duties.asked.is_empty());
            assert_eq!(host.state().signing(), &temper_client_tui::Signing::Refused(temper_client_tui::Refusal::NotALink));
            for key in link("127.0.0.1:8367") { host.report(Report::Key(key)); }
            assert!(host.state().pasted());
            ask(&host.report(Report::Key(Key::Enter)));
        } else {
            let exchange = ask(&duties);
            let duties = reply(&mut host, exchange, approval_bytes(b"BCDFGHJK", b""));
            assert_eq!(duties.open.as_deref(), Some(b"http://127.0.0.1:8367/terminal?code=BCDF-GHJK".as_slice()));
            assert!(!host.state().pasted());
        }
    }
    for answer in [w::ApprovalAskAnswer::Refused, w::ApprovalAskAnswer::NotReady, w::ApprovalAskAnswer::Busy] {
        let mut host = host();
        let exchange = ask(&start(&mut host, None));
        let mut writer = skein_lib::Writer::new(usize::try_from(answer.measure()).expect("length"));
        answer.encode(&mut writer).expect("answer");
        reply(&mut host, exchange, writer.finish());
        host.frame();
        ask(&host.report(Report::Key(Key::Enter)));
    }
}

#[test]
fn approval_deadline_includes_the_views_tick() {
    let mut host = host();
    let exchange = ask(&start(&mut host, None));
    reply(&mut host, exchange, approval_bytes(b"WDJBMJHT", b""));
    let now = Time::from_nanos(11_000_000_000);
    host.clocks(now, Wall::from_nanos(3_000_000_000));
    host.report(Report::Resize(Size { columns: 80, rows: 24 }));
    let tick = host.view.tick().expect("expiry tick");
    // Move the monotonic clock close enough that the view's tick comes first.
    let root = temper_client_domain::next_deadline(&host.root).expect("take deadline");
    let delta = tick.as_nanos().saturating_sub(host.env.wall.as_nanos());
    host.clocks(Time::from_nanos(root.as_nanos().saturating_sub(delta).saturating_sub(1)), host.env.wall);
    assert_eq!(host.deadline(), Some(host.env.now.saturating_add(skein_lib::Duration::from_nanos(delta))));
}
