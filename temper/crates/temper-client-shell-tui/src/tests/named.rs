use super::*;
use crate::loopback::{Answer, Returned};
use temper_client_tui::{Browser, Origin, Signing};

const PORT: u16 = 43210;
const PAGE: &[u8] = b"https://dev.fabrig.ai/terminal?code=WDJB-MJHT";
const GRANT: [u8; 32] = [83; 32];

fn returned() -> Returned {
    let mut encoded = Writer::new(43);
    skein_lib::base64::encode(skein_lib::base64::Alphabet::Url, &GRANT, &mut encoded).expect("grant");
    Returned {
        get: true,
        target: [b"/approved?grant=".as_slice(), &encoded.finish()].concat().into(),
        host: Some(b"127.0.0.1:43210".as_slice().into()),
    }
}

fn listening(host: &mut Host) -> Duties {
    let duties = host.report(Report::Listening { port: Some(PORT) });
    assert!(!duties.unlisten, "keep the listener while the ask is in flight");
    assert_eq!(asked_record(&duties).port, Some(PORT));
    duties
}

fn asked_record(duties: &Duties) -> w::ApprovalAsk {
    duties
        .asked
        .iter()
        .find_map(|request| {
            if let Request::Exchange { message: Message::ApprovalAsk(bytes), .. } = request {
                Some(w::ApprovalAsk::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("ask record"))
            } else {
                None
            }
        })
        .expect("ask sent")
}

fn one_click(host: &mut Host) -> Duties {
    assert!(start_name(host, None, None).listen);
    let exchange = ask(&listening(host));
    reply(host, exchange, approval_bytes(b"WDJBMJHT", b""))
}

fn taken_record(duties: &Duties) -> w::ApprovalSignIn {
    duties
        .asked
        .iter()
        .find_map(|request| {
            if let Request::Exchange { message: Message::SignInApproval(bytes), .. } = request {
                Some(w::ApprovalSignIn::decode(&w::CEILINGS, &mut Reader::new(bytes)).expect("take record"))
            } else {
                None
            }
        })
        .expect("take sent")
}

fn redirected(duties: &Duties) {
    assert!(matches!(&duties.answer, Some(Answer::Redirect { to }) if to.as_ref() == PAGE));
    assert!(!duties.unlisten);
    let record = taken_record(duties);
    assert_eq!(record.secret.expose(), &[7; 32]);
    assert_eq!(record.grant.as_ref().expect("grant sent").expose(), &GRANT);
}

fn not_found(duties: &Duties) {
    assert!(matches!(duties.answer, Some(Answer::NotFound)));
    assert!(duties.asked.is_empty(), "no take for another request");
}

fn start_name(host: &mut Host, credential: Option<Secret>, note: Option<Note>) -> Duties {
    host.report(Report::Start(Start {
        set_up: false,
        box_log: Box::default(),
        front: false,
        size: Size { columns: 80, rows: 24 },
        scheme: Some(Scheme::Dark),
        background: None,
        plain: false,
        version: Box::default(),
        boxed: false,
        address: Origin::Https { name: Box::from(*b"dev.fabrig.ai"), port: 443 },
        local: false,
        note,
        kept: Vec::new(),
        credential,
    }))
}

fn take(duties: &Duties) -> skein_lib::Token {
    duties
        .asked
        .iter()
        .find_map(|request| {
            if let Request::Exchange { exchange, message: Message::SignInApproval(_), .. } = request {
                Some(*exchange)
            } else {
                None
            }
        })
        .expect("approval take")
}

fn take_answer(host: &mut Host, exchange: skein_lib::Token, answer: &w::ApprovalSignInAnswer) -> Duties {
    let mut writer = Writer::new(
        usize::try_from(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
            .expect("answer length"),
    );
    answer.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("approval answer");
    reply(host, exchange, writer.finish())
}

#[test]
fn one_click_waits_for_the_listener_and_opens_the_code_once() {
    let mut host = host();
    let duties = start_name(&mut host, None, None);
    assert!(duties.listen);
    assert!(!duties.unlisten);
    assert_eq!(duties.asked.len(), 0);
    assert!(duties.open.is_none());
    let frame = text(&mut host);
    assert!(frame.contains("Asking temper for a code…"));
    assert!(!frame.contains("Ask again"));
    assert!(!frame.contains("Type a code instead"));
    let exchange = ask(&listening(&mut host));
    empty(&host.report(Report::Due));
    let duties = reply(&mut host, exchange, approval_bytes(b"WDJBMJHT", b""));
    assert_eq!(duties.open.as_deref(), Some(PAGE));
    assert!(!duties.unlisten);
    assert!(taken_record(&duties).grant.is_none());
    let frame = text(&mut host);
    for line in [
        "A tab opened in your browser: press Approve there.",
        "The code there is WDJB-MJHT.",
        "The code expires in 10 min.",
        "No tab? Press Enter for a code to type instead.",
        "❯ Type a code instead",
    ] {
        assert!(frame.contains(line), "{line}: {frame}");
    }
    empty(&host.report(Report::Opened { opened: false }));
    let frame = text(&mut host);
    assert!(frame.contains(std::str::from_utf8(PAGE).expect("page")));
    assert!(frame.contains("in a browser on this computer, then press Approve."));
    empty(&host.report(Report::Due));
}

#[test]
fn a_grant_takes_the_ask_and_keeps_the_credential_before_launching() {
    let mut host = host();
    one_click(&mut host);
    let duties = host.report(Report::Returned(returned()));
    redirected(&duties);
    let w::SignInAnswer::SignedIn(signed) = signed_in_answer() else { panic!("signed in fixture") };
    let duties = take_answer(&mut host, take(&duties), &w::ApprovalSignInAnswer::SignedIn(signed));
    assert!(duties.unlisten);
    assert_eq!(duties.keep, Some(Some(Secret::new(Box::new([7; 40])))));
    assert!(duties.asked.iter().any(|request| matches!(request, Request::OpenStream { .. })));
    host.report(Report::Kept { written: true });
    let hello = hello_for_me(&host, launchpad_me(true));
    let duties =
        host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::Record(hello) }));
    admit(&mut host, duties);
    let frame = text(&mut host);
    assert!(!temper_client_tui::sign_in_drawn(&host.root, host.state()));
    assert!(frame.contains("Start") && frame.contains("/draft") && frame.contains("● Live"), "{frame}");
    empty(&host.report(Report::Resize(Size { columns: 80, rows: 24 })));
    not_found(&host.report(Report::Returned(returned())));
}

#[test]
fn returns_need_the_current_asked_port_and_exact_loopback_request() {
    let mut host = host();
    assert!(start_name(&mut host, None, None).listen);
    not_found(&host.report(Report::Returned(returned())));
    let exchange = ask(&listening(&mut host));
    not_found(&host.report(Report::Returned(returned())));
    let duties = reply(&mut host, exchange, approval_bytes(b"WDJBMJHT", b""));
    let take = take(&duties);
    let good = returned();
    for bad in [
        Returned { host: Some(b"localhost:43210".as_slice().into()), ..good.clone() },
        Returned { host: Some(b"127.0.0.1:43211".as_slice().into()), ..good.clone() },
        Returned { host: None, ..good.clone() },
        Returned { get: false, ..good.clone() },
        Returned { target: b"/other?grant=x".as_slice().into(), ..good.clone() },
        Returned { target: [b"/approved?grant=".as_slice(), &b"A".repeat(42)].concat().into(), ..good.clone() },
        Returned { target: [b"/approved?grant=".as_slice(), &b"A".repeat(44)].concat().into(), ..good.clone() },
        Returned { target: [b"/approved?grant=".as_slice(), &b"!".repeat(43)].concat().into(), ..good.clone() },
    ] {
        let duties = host.report(Report::Returned(bad));
        not_found(&duties);
        assert!(!duties.unlisten);
    }
    assert!(take_answer(&mut host, take, &w::ApprovalSignInAnswer::Expired).unlisten);
    not_found(&host.report(Report::Returned(good)));
}

#[test]
fn a_not_granted_take_keeps_waiting_for_the_next_grant() {
    let mut host = host();
    one_click(&mut host);
    let frame = text(&mut host);
    let duties = host.report(Report::Returned(returned()));
    redirected(&duties);
    let duties = take_answer(&mut host, take(&duties), &w::ApprovalSignInAnswer::NotGranted);
    assert!(!duties.unlisten);
    assert_eq!(text(&mut host), frame);
    redirected(&host.report(Report::Returned(returned())));
}

#[test]
fn typing_a_code_replaces_the_one_click_and_ignores_its_late_opener() {
    let mut host = host();
    one_click(&mut host);
    assert!(text(&mut host).contains("❯ Type a code instead"));
    let duties = host.report(Report::Key(Key::Enter));
    assert!(duties.unlisten);
    assert!(!duties.listen);
    assert!(duties.open.is_none());
    assert_eq!(host.state().browser(), Some(Browser::None));
    assert!(asked_record(&duties).port.is_none());
    let exchange = ask(&duties);
    let duties = reply(&mut host, exchange, approval_bytes(b"WDJBMJHT", b""));
    assert!(duties.open.is_none());
    assert!(!duties.unlisten);
    let frame = text(&mut host);
    for line in [
        "Open",
        "https://dev.fabrig.ai/terminal",
        "in a browser signed in to temper, type WDJB-MJHT,",
        "then press Approve.",
    ] {
        assert!(frame.contains(line), "{line}: {frame}");
    }
    assert!(!frame.contains("?code="));
    empty(&host.report(Report::Opened { opened: true }));
    assert_eq!(host.state().browser(), Some(Browser::None));
    assert_eq!(text(&mut host), frame);
    not_found(&host.report(Report::Returned(returned())));
}

#[test]
fn unwanted_listeners_close_on_replacement_and_late_listening() {
    let mut hosts = Hosts::new();
    for opening in [true, false] {
        let host = hosts.next();
        assert!(start_name(host, None, None).listen);
        if !opening {
            listening(host);
        }
        host.step(root::Event::Client {
            event: jig_client::Event::SignIn {
                with: jig_client::With::Link { secret: Secret::new(Box::new([9; 32])), held: None },
            },
        });
        let duties = host.report(Report::Due);
        let exchange = local_exchange(&duties);
        assert!(reply(host, exchange, sign_in_bytes(&signed_in_answer())).unlisten);
        empty(&host.report(Report::Resize(Size { columns: 80, rows: 24 })));
        assert!(host.report(Report::Listening { port: Some(PORT) }).unlisten);
    }
    let host = hosts.next();
    one_click(host);
    host.step(root::Event::AskApproval { port: None });
    assert!(host.report(Report::Due).unlisten, "another ask replaces this listener's ask");
    let host = hosts.next();
    assert!(start_name(host, None, None).listen);
    host.step(root::Event::AskApproval { port: None });
    let duties = host.report(Report::Due);
    assert!(duties.unlisten, "another ask cancels the bind");
    assert!(host.report(Report::Listening { port: Some(PORT) }).unlisten);
    assert!(
        reply(host, ask(&duties), approval_bytes(b"WDJBMJHT", b"")).open.is_none(),
        "do not open a tab for the replacement"
    );
    let host = hosts.next();
    one_click(host);
    let duties = host.report(Report::Listening { port: None });
    assert!(duties.asked.is_empty(), "losing an open listener does not issue a second ask");
    not_found(&host.report(Report::Returned(returned())));
    let host = hosts.next();
    assert!(start_name(host, None, None).listen);
    let duties = host.report(Report::Key(Key::Control(b'c')));
    assert!(duties.close && duties.unlisten);
    assert!(host.report(Report::Listening { port: Some(PORT) }).unlisten);
    empty(&host.report(Report::Due));
}

#[test]
fn a_named_terminal_asks_and_opens_once_without_a_code() {
    for opened in [false, true] {
        let mut host = host();
        let duties = start_name(&mut host, None, None);
        assert!(!duties.local);
        assert!(temper_client_tui::sign_in_drawn(&host.root, host.state()));
        assert!(duties.listen);
        let duties = host.report(Report::Listening { port: None });
        assert!(asked_record(&duties).port.is_none());
        assert!(!duties.unlisten);
        let exchange = ask(&duties);
        let duties = reply(&mut host, exchange, approval_bytes(b"WDJBMJHT", b""));
        assert_eq!(duties.open.as_deref(), Some(b"https://dev.fabrig.ai/terminal".as_slice()));
        assert_eq!(host.state().browser(), Some(Browser::Opening));
        let frame = text(&mut host);
        assert!(frame.contains("A tab opened in your browser: type WDJB-MJHT there, then press Approve."), "{frame}");
        assert!(!frame.contains("Type a code instead"));
        empty(&host.report(Report::Opened { opened }));
        assert_eq!(host.state().browser(), Some(if opened { Browser::Opened } else { Browser::None }));
        empty(&host.report(Report::Resize(Size { columns: 80, rows: 24 })));
        empty(&host.report(Report::Due));
    }
}

#[test]
fn a_named_sign_in_ignores_keys_pastes_and_sign_in_while_asking_or_waiting() {
    let mut host = host();
    assert!(start_name(&mut host, None, None).listen);
    let mut exchange = None;
    for phase in ["listening", "asking", "waiting"] {
        if phase == "asking" {
            exchange = Some(ask(&host.report(Report::Listening { port: None })));
        }
        if phase == "waiting" {
            let duties = reply(&mut host, exchange.expect("ask sent"), approval_bytes(b"WDJBMJHT", b""));
            take_answer(&mut host, take(&duties), &w::ApprovalSignInAnswer::Waiting);
        }
        host.frame();
        let before = text(&mut host);
        let revision = host.state().revision();
        empty(&give(&mut host, Event::SignIn));
        empty(&give(&mut host, Event::Typed { value: b"not a sign-in link".as_slice().into() }));
        for key in [
            Key::Enter,
            Key::Char(' '),
            Key::Char('q'),
            Key::Char('a'),
            Key::Paste(
                b"http://127.0.0.1:8367/jig/v1/sign-in/link?secret=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"
                    .as_slice()
                    .into(),
            ),
        ] {
            empty(&host.report(Report::Key(key)));
        }
        assert_eq!(host.state().revision(), revision);
        assert_eq!(host.state().field(), b"");
        assert_eq!(host.state().signing(), &Signing::Idle);
        assert!(!host.state().pasted());
        assert_eq!(text(&mut host), before);
    }
    assert!(host.report(Report::Key(Key::Control(b'c'))).close);
}

#[test]
fn named_sign_in_asks_again_only_after_an_outcome_and_refocuses_the_button() {
    for answer in [
        w::ApprovalSignInAnswer::Refused,
        w::ApprovalSignInAnswer::Expired,
        w::ApprovalSignInAnswer::Used,
        w::ApprovalSignInAnswer::Unknown,
    ] {
        let mut host = host();
        assert!(start_name(&mut host, None, None).listen);
        let exchange = ask(&host.report(Report::Listening { port: Some(43210) }));
        let duties = reply(&mut host, exchange, approval_bytes(b"WDJBMJHT", b""));
        assert!(take_answer(&mut host, take(&duties), &answer).unlisten);
        empty(&host.report(Report::Resize(Size { columns: 80, rows: 24 })));
        // The button disappears during each ask; its next appearance requests focus again.
        for key in [Key::Enter, Key::Char(' ')] {
            assert!(text(&mut host).contains("❯ Ask again"));
            let duties = host.report(Report::Key(key));
            assert!(duties.listen);
            assert!(!duties.unlisten);
            assert!(!text(&mut host).contains("Ask again"));
            let exchange = ask(&host.report(Report::Listening { port: Some(43210) }));
            let duties = reply(&mut host, exchange, approval_bytes(b"BCDFGHJK", b""));
            assert_eq!(duties.open.as_deref(), Some(b"https://dev.fabrig.ai/terminal?code=BCDF-GHJK".as_slice()));
            assert!(take_answer(&mut host, take(&duties), &answer).unlisten);
        }
    }
    for answer in [
        Some(w::ApprovalAskAnswer::Refused),
        Some(w::ApprovalAskAnswer::Busy),
        Some(w::ApprovalAskAnswer::NotReady),
        None,
    ] {
        let mut host = host();
        assert!(start_name(&mut host, None, None).listen);
        let exchange = ask(&host.report(Report::Listening { port: Some(43210) }));
        if let Some(answer) = answer {
            let mut writer = Writer::new(
                usize::try_from(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"))
                    .expect("answer length"),
            );
            answer.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("ask answer");
            assert!(reply(&mut host, exchange, writer.finish()).unlisten);
        } else {
            assert!(host.report(Report::Heard(jig_client::Event::Lost { exchange })).unlisten);
        }
        assert!(text(&mut host).contains("❯ Ask again"));
        empty(&host.report(Report::Resize(Size { columns: 80, rows: 24 })));
        assert!(give(&mut host, Event::SignIn).listen);
    }
}

#[test]
fn an_expired_held_sign_in_at_a_name_asks_again_without_a_local_link() {
    for wall in [UNTIL, UNTIL + 1] {
        let mut host = host();
        let duties =
            start_name(&mut host, Some(Secret::new(Box::new([7; 40]))), Some(Note::Until(Wall::from_nanos(UNTIL))));
        assert!(!duties.local);
        assert!(
            !duties
                .asked
                .iter()
                .any(|request| matches!(request, Request::Exchange { message: Message::ApprovalAsk(_), .. }))
        );
        host.clocks(Time::ZERO, Wall::from_nanos(wall));
        let duties =
            host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::SignedOut }));
        assert!(duties.listen);
        assert!(!duties.unlisten);
        assert!(!duties.local);
        assert_eq!(duties.keep, Some(None));
        assert!(temper_client_tui::sign_in_drawn(&host.root, host.state()));
        let screen = text(&mut host);
        assert!(screen.contains("Signed out."));
        assert!(screen.contains("Sign in this terminal to dev.fabrig.ai"));
        assert!(screen.contains("Asking temper for a code…"));
        assert!(!screen.contains("Link:"));
    }
}
