use super::*;
use temper_client_tui::{Browser, Origin, Signing};

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
    let mut writer = Writer::new(usize::try_from(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).expect("answer length"));
    answer.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("approval answer");
    reply(host, exchange, writer.finish())
}

#[test]
fn a_named_terminal_asks_and_opens_once_without_a_code() {
    for opened in [false, true] {
        let mut host = host();
        let duties = start_name(&mut host, None, None);
        assert!(!duties.local);
        assert!(temper_client_tui::sign_in_drawn(&host.root, host.state()));
        let exchange = ask(&duties);
        let duties = reply(&mut host, exchange, approval_bytes(b"WDJBMJHT", b""));
        assert_eq!(duties.open.as_deref(), Some(b"https://dev.fabrig.ai/terminal".as_slice()));
        assert_eq!(host.state().browser(), Some(Browser::Opening));
        empty(&host.report(Report::Opened { opened }));
        assert_eq!(host.state().browser(), Some(if opened { Browser::Opened } else { Browser::None }));
        empty(&host.report(Report::Resize(Size { columns: 80, rows: 24 })));
        empty(&host.report(Report::Due));
    }
}

#[test]
fn a_named_sign_in_ignores_keys_pastes_and_sign_in_while_asking_or_waiting() {
    let mut host = host();
    let exchange = ask(&start_name(&mut host, None, None));
    for asking in [true, false] {
        if !asking {
            let duties = reply(&mut host, exchange, approval_bytes(b"WDJBMJHT", b""));
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
        let exchange = ask(&start_name(&mut host, None, None));
        let duties = reply(&mut host, exchange, approval_bytes(b"WDJBMJHT", b""));
        take_answer(&mut host, take(&duties), &answer);
        // The button disappears during each ask; its next appearance requests focus again.
        for key in [Key::Enter, Key::Char(' ')] {
            assert!(text(&mut host).contains("❯ Ask again"));
            let exchange = ask(&host.report(Report::Key(key)));
            assert!(!text(&mut host).contains("Ask again"));
            let duties = reply(&mut host, exchange, approval_bytes(b"BCDFGHJK", b""));
            assert_eq!(duties.open.as_deref(), Some(b"https://dev.fabrig.ai/terminal".as_slice()));
            take_answer(&mut host, take(&duties), &answer);
        }
    }
    for answer in [
        Some(w::ApprovalAskAnswer::Refused),
        Some(w::ApprovalAskAnswer::Busy),
        Some(w::ApprovalAskAnswer::NotReady),
        None,
    ] {
        let mut host = host();
        let exchange = ask(&start_name(&mut host, None, None));
        if let Some(answer) = answer {
            let mut writer = Writer::new(usize::try_from(answer.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).expect("answer length"));
            answer.write(&jig_client_protocol_wire::CEILINGS, &mut writer).expect("ask answer");
            reply(&mut host, exchange, writer.finish());
        } else {
            host.report(Report::Heard(jig_client::Event::Lost { exchange }));
        }
        assert!(text(&mut host).contains("❯ Ask again"));
        ask(&give(&mut host, Event::SignIn));
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
        ask(&duties);
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
