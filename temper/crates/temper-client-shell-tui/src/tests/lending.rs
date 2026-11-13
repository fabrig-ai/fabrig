//! The offer and login duties, without a terminal or child (client/tui/box.md, Lending its terminal).
use super::*;
use jig_client::contract::LoginExit;

const LINE: &[u8] = b"temper: Claude Code signs in here. Approve it on Anthropic's page, in the tab it opens; if that page shows a code, paste it here. temper comes back when it's done, or when you press Cancel in the browser.";

fn subject(number: u64) -> w::Subject {
    w::Subject::Terminal(w::SubjectTerminal::new(&w::CEILINGS, w::SubjectTerminalParts { terminal: number }).unwrap())
}

fn front_host() -> (Host, u64) {
    let mut host = host();
    start_local(&mut host, true, false, None, None);
    let duties = host.report(Report::Local { link: Some(local_link("127.0.0.1:8367", LOCAL_SECRET)) });
    reply(&mut host, local_exchange(&duties), sign_in_bytes(&signed_in_answer()));
    let hello = hello_for(&host);
    let duties =
        host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::Record(hello) }));
    let duties = admit(&mut host, duties);
    let terminal = watch(&duties, &subject(skein_lib::Rng::new(7).next_u64()));
    let me = watch(&duties, &w::Subject::Me);
    let duties = delivery(&mut host, me, &w::DeliveryValue::Me(launchpad_me(false)));
    admit(&mut host, duties);
    (host, terminal)
}

pub(crate) fn terminal_document(login: Option<(u32, bool)>) -> w::DeliveryValue {
    w::DeliveryValue::Terminal(
        w::Terminal::new(
            &w::CEILINGS,
            w::TerminalParts {
                login: login.map(|(number, cancelled)| {
                    w::TerminalLogin::new(
                        &w::CEILINGS,
                        w::TerminalLoginParts { number, provider: 17, name: b"work".as_slice().into(), cancelled },
                    )
                    .unwrap()
                }),
            },
        )
        .unwrap(),
    )
}

fn lend(duties: &Duties, number: u32) {
    let lend = duties.lend.as_ref().expect("lend duty");
    assert_eq!(lend.number, number);
    assert_eq!(lend.name.as_ref(), b"work");
    assert_eq!(lend.line.as_ref(), LINE);
    assert!(!duties.unlend);
}

#[test]
fn only_a_front_start_offers_its_seeded_terminal_once() {
    use core::mem::discriminant;
    for front in [false, true] {
        for seed in [7, 19] {
            let mut host = Host::new(&Settings { limits: limits::tui() }, seed, Time::ZERO, Wall::EPOCH);
            start_local(&mut host, front, false, None, None);
            let lends = discriminant(&root::Event::Lends { terminal: 0 });
            let launchpad = discriminant(&root::Event::Launchpad);
            let positions: Vec<_> =
                host.events.iter().enumerate().filter(|(_, event)| **event == lends).map(|(at, _)| at).collect();
            if front {
                let at = *positions.first().expect("offer");
                assert_eq!(host.events.get(at.checked_sub(1).unwrap()), Some(&launchpad));
                assert_eq!(host.terminal_number, skein_lib::Rng::new(seed).next_u64());
            }
            assert_eq!(positions.len(), usize::from(front));
            host.report(Report::Due);
            host.report(Report::Resize(Size { rows: 24, columns: 80 }));
            assert_eq!(host.events.iter().filter(|event| **event == lends).count(), usize::from(front));
        }
    }
}

#[test]
fn a_named_login_is_taken_once_until_its_end_and_cancel_is_given_once() {
    let (mut host, terminal) = front_host();
    let cancelled = delivery(&mut host, terminal, &terminal_document(Some((1, true))));
    assert!(cancelled.lend.is_none());
    assert!(!cancelled.unlend);
    lend(&delivery(&mut host, terminal, &terminal_document(Some((1, false)))), 1);
    assert_eq!(host.state().lending().unwrap().name.as_ref(), b"work");
    empty(&host.report(Report::Resize(Size { rows: 24, columns: 80 })));
    empty(&delivery(&mut host, terminal, &terminal_document(Some((1, false)))));
    empty(&delivery(&mut host, terminal, &terminal_document(Some((2, true)))));
    empty(&delivery(&mut host, terminal, &terminal_document(Some((2, false)))));
    empty(&delivery(&mut host, terminal, &terminal_document(None)));
    assert_eq!(host.state().lending().unwrap().number, 1, "a missing document does not end the loan");
    let duties = delivery(&mut host, terminal, &terminal_document(Some((1, true))));
    assert!(duties.unlend);
    assert!(duties.lend.is_none());
    empty(&delivery(&mut host, terminal, &terminal_document(Some((1, true)))));
    assert!(host.state().lending().unwrap().unlend);
}

#[test]
fn a_lent_report_sends_the_end_and_shows_its_own_notice_then_takes_a_new_number() {
    for (exit, cancel, ended_box, notice) in [
        (LoginExit::Exited { code: 0 }, false, false, "Claude Code signed in"),
        (LoginExit::Exited { code: 0 }, false, true, "Claude Code signed in"),
        (LoginExit::Exited { code: 1 }, false, false, "Claude Code's sign-in didn't finish"),
        (LoginExit::Killed { signal: 15 }, false, false, "Claude Code's sign-in didn't finish"),
        (LoginExit::NotStarted, false, false, "Claude Code's sign-in didn't finish"),
        (LoginExit::Exited { code: 0 }, true, false, "Claude Code's sign-in cancelled"),
        (LoginExit::Killed { signal: 15 }, true, false, "Claude Code's sign-in cancelled"),
    ] {
        let (mut host, terminal) = front_host();
        lend(&delivery(&mut host, terminal, &terminal_document(Some((1, false)))), 1);
        empty(&host.report(Report::Resize(Size { rows: 24, columns: 80 })));
        if cancel {
            assert!(delivery(&mut host, terminal, &terminal_document(Some((1, true)))).unlend);
        }
        if ended_box {
            host.report(Report::Stage(BoxStage::Stopping));
            host.report(Report::Stage(BoxStage::Ended { end: BoxEnd::Exited { code: 0 } }));
            assert!(host.closed);
        }
        let duties = host.report(Report::Lent { number: 1, exit });
        let duties = admit(&mut host, duties);
        let asks = sent_ask(&duties);
        let [(_, w::Ask::AccountChange(ask))] = asks.as_slice() else { panic!("one LoginEnded: {asks:?}") };
        assert_eq!(ask.holder(), &w::AskAccountHolder::Own);
        let w::AccountChange::LoginEnded(ended) = ask.change() else { panic!("LoginEnded") };
        assert_eq!(ended.number(), 1);
        let expected = match exit {
            LoginExit::Exited { code } => {
                w::LoginExit::Exited(w::LoginExitExited::new(&w::CEILINGS, w::LoginExitExitedParts { code }).unwrap())
            }
            LoginExit::Killed { signal } => {
                w::LoginExit::Killed(w::LoginExitKilled::new(&w::CEILINGS, w::LoginExitKilledParts { signal }).unwrap())
            }
            LoginExit::NotStarted => w::LoginExit::NotStarted,
        };
        assert_eq!(ended.exit(), &expected);
        assert!(host.state().lending().is_none());
        assert!(text(&mut host).contains(notice), "{exit:?}: {}", text(&mut host));
        empty(&delivery(&mut host, terminal, &terminal_document(Some((1, false)))));
        let next = delivery(&mut host, terminal, &terminal_document(Some((2, false))));
        if ended_box {
            assert!(next.lend.is_none());
        } else {
            lend(&next, 2);
        }
        host.clocks(Time::ZERO.saturating_add(temper_client_tui::limits::NOTICE), Wall::EPOCH);
        host.report(Report::Due);
        if !ended_box {
            assert!(host.state().notice().is_none(), "the normal notice deadline lowers it");
        }
    }
}
