use super::fixtures as f;
use crate::{Told, Tui, approval_address, may_ask_again};
use jig_client::{ApprovalCode, Link, NotAsked, NotSignedIn, Taking};
use jig_client_protocol_wire::v22 as w;
use skein_lib::{List, Queue, Secret, Token, Wall, Writer};
use temper_client_domain::{Domain, Event, Request, asked::State};

pub(super) fn code() -> ApprovalCode {
    ApprovalCode::parse(b"WDJBMJHT").unwrap()
}

#[test]
fn the_approval_address_uses_the_tuis_address_and_shown_code() {
    let rows: &[(&[u8], &[u8], &[u8])] = &[
        (b"127.0.0.1:8367", b"WDJBMJHT", b"http://127.0.0.1:8367/terminal?code=WDJB-MJHT"),
        (b"127.0.0.1:9000", b"bcdfghjk", b"http://127.0.0.1:9000/terminal?code=BCDF-GHJK"),
        (b"[::1]:8367", b"bcdf-ghjk", b"http://[::1]:8367/terminal?code=BCDF-GHJK"),
    ];
    for (address, letters, expected) in rows {
        let code = ApprovalCode::parse(letters).unwrap();
        let actual = approval_address(address, code);
        assert_eq!(actual.as_ref(), *expected);
        assert!(actual.ends_with(&code.shown()));
    }
}

pub(super) fn exchange(mut out: Queue<Request>) -> Token {
    let mut found = None;
    while let Some(request) = out.pop() {
        if let Request::Client { request: jig_client::Request::Exchange { exchange, .. } } = request {
            assert!(found.replace(exchange).is_none());
        }
    }
    found.expect("one approval exchange")
}

pub(super) fn asked(state: Option<State>) -> Domain {
    let mut root = temper_client_domain::new(&f::limits(), f::SEED);
    f::event(
        &mut root,
        Event::Start { address: temper_client_domain::Address::Accounts, kept: List::with_capacity(0), with: None },
    );
    if let Some(state) = state {
        let id = exchange(f::event(&mut root, Event::AskApproval));
        let answer = match state {
            State::Asking => None,
            State::Asked { code, expires } => Some(w::ApprovalAskAnswer::Asked(
                w::ApprovalAsked::new(
                    &w::CEILINGS,
                    w::ApprovalAskedParts {
                        secret: Secret::new(Box::new([7; 32])),
                        code: *code.letters(),
                        expires: expires.as_nanos(),
                    },
                )
                .unwrap(),
            )),
            State::NotAsked(NotAsked::Refused) => Some(w::ApprovalAskAnswer::Refused),
            State::NotAsked(NotAsked::Busy) => Some(w::ApprovalAskAnswer::Busy),
            State::NotAsked(NotAsked::NotReady) => Some(w::ApprovalAskAnswer::NotReady),
            State::NotAsked(NotAsked::Lost) => {
                f::client(&mut root, jig_client::Event::Lost { exchange: id });
                None
            }
        };
        if let Some(answer) = answer {
            let mut writer = Writer::new(usize::try_from(answer.measure()).unwrap());
            answer.encode(&mut writer).unwrap();
            f::client(&mut root, jig_client::Event::Reply { exchange: id, message: writer.finish() });
        }
    }
    assert_eq!(root.asked(), state);
    assert_eq!(root.client().link().state, Link::SignedOut);
    root
}

pub(super) fn outcome(root: &mut Domain, why: Option<NotSignedIn>) {
    let Some(why) = why else { return };
    let link_answer = match why {
        NotSignedIn::LinkUsed => Some(w::SignInAnswer::LinkUsed),
        NotSignedIn::LinkExpired => Some(w::SignInAnswer::LinkExpired),
        NotSignedIn::LinkUnknown => Some(w::SignInAnswer::LinkUnknown),
        NotSignedIn::Approval(_) | NotSignedIn::Full { .. } | NotSignedIn::Busy | NotSignedIn::NotReady => None,
    };
    if let Some(answer) = link_answer {
        let out = f::client(
            root,
            jig_client::Event::SignIn {
                with: jig_client::With::Link { secret: Secret::new(Box::new([7; 32])), held: None },
            },
        );
        let id = exchange(out);
        f::answer(root, id, answer);
        return;
    }
    let out = f::client(
        root,
        jig_client::Event::SignIn { with: jig_client::With::Approval { secret: Secret::new(Box::new([7; 32])) } },
    );
    let id = exchange(out);
    let answer = match why {
        NotSignedIn::Approval(Taking::Waiting) => w::ApprovalSignInAnswer::Waiting,
        NotSignedIn::Approval(Taking::Refused) => w::ApprovalSignInAnswer::Refused,
        NotSignedIn::Approval(Taking::Expired) => w::ApprovalSignInAnswer::Expired,
        NotSignedIn::Approval(Taking::Used) => w::ApprovalSignInAnswer::Used,
        NotSignedIn::Approval(Taking::Unknown) => w::ApprovalSignInAnswer::Unknown,
        NotSignedIn::Full { sign_ins, earliest, link_until } => w::ApprovalSignInAnswer::Full(
            w::ApprovalFull::new(
                &w::CEILINGS,
                w::ApprovalFullParts { sign_ins, earliest: earliest.as_nanos(), until: link_until.as_nanos() },
            )
            .unwrap(),
        ),
        NotSignedIn::Busy => w::ApprovalSignInAnswer::Busy,
        NotSignedIn::NotReady => w::ApprovalSignInAnswer::NotReady,
        NotSignedIn::LinkUsed | NotSignedIn::LinkExpired | NotSignedIn::LinkUnknown => unreachable!("link answered"),
    };
    let mut writer = Writer::new(usize::try_from(answer.measure()).unwrap());
    answer.encode(&mut writer).unwrap();
    f::client(root, jig_client::Event::Reply { exchange: id, message: writer.finish() });
}

#[test]
fn asking_again_is_the_approval_rules_table() {
    use NotSignedIn::{Approval, Busy, Full, NotReady};
    let states = [
        None,
        Some(State::Asking),
        Some(State::Asked { code: code(), expires: Wall::from_nanos(300) }),
        Some(State::NotAsked(NotAsked::Refused)),
        Some(State::NotAsked(NotAsked::NotReady)),
        Some(State::NotAsked(NotAsked::Busy)),
        Some(State::NotAsked(NotAsked::Lost)),
    ];
    let waiting = [false, false, false, true, true, true, true];
    let ended = [true, false, true, true, true, true, true];
    let rows = [
        (None, waiting),
        (Some(Approval(Taking::Waiting)), waiting),
        (Some(Approval(Taking::Refused)), ended),
        (Some(Approval(Taking::Expired)), ended),
        (Some(Approval(Taking::Used)), ended),
        (Some(Approval(Taking::Unknown)), ended),
        (Some(Full { sign_ins: 4, earliest: Wall::EPOCH, link_until: Wall::from_nanos(300) }), ended),
        (Some(Busy), waiting),
        (Some(NotReady), waiting),
        (Some(NotSignedIn::LinkUsed), waiting),
        (Some(NotSignedIn::LinkExpired), waiting),
        (Some(NotSignedIn::LinkUnknown), waiting),
    ];
    for (why, unpasted) in rows {
        for (index, state) in states.iter().enumerate() {
            let mut root = asked(*state);
            outcome(&mut root, why);
            assert_eq!(root.asked(), *state);
            assert_eq!(root.client().link().not_signed_in, why);
            assert_eq!(root.client().link().state, Link::SignedOut);
            for (pasted, expected) in [(false, unpasted[index]), (true, ended[index])] {
                let mut tui = Tui::new(f::ADDRESS);
                if pasted {
                    tui.step(Told::Asked);
                }
                assert_eq!(may_ask_again(&tui, &root), expected, "{state:?}, {why:?}, pasted={pasted}");
            }
        }
    }
    for link in [Link::Starting, Link::Live, Link::Offline, Link::OutOfDate] {
        let (mut root, watch) = if link == Link::Starting { f::starting() } else { f::live(None) };
        match link {
            Link::Starting | Link::Live => {}
            Link::Offline => {
                f::client(
                    &mut root,
                    jig_client::Event::Up {
                        up: jig_client::Up::WatchEnded { watch, end: jig_client::contract::WatchEnd::Dropped },
                    },
                );
            }
            Link::OutOfDate => {
                f::client(
                    &mut root,
                    jig_client::Event::Up { up: jig_client::Up::Served { served: false, versions: None } },
                );
            }
            Link::SignedOut => unreachable!("signed in rows"),
        }
        assert_eq!(root.client().link().state, link);
        for pasted in [false, true] {
            let mut tui = Tui::new(f::ADDRESS);
            if pasted {
                tui.step(Told::Asked);
            }
            assert!(!may_ask_again(&tui, &root), "{link:?}, pasted={pasted}");
        }
    }
}

#[test]
#[expect(clippy::manual_map, reason = "the subset requires an exhaustive match instead of Option::map")]
fn the_view_clock_ticks_only_when_its_words_change() {
    let root = f::signed_out();
    let tui = Tui::new(f::ADDRESS);
    let mut view = f::view();
    assert_eq!(view.tick(), None);
    for (wall, changes) in
        [(Wall::EPOCH, true), (Wall::from_nanos(1), false), (Wall::from_nanos(60_000_000_000), false)]
    {
        let mut out = Queue::with_capacity(crate::limits(&f::limits(), &skein_ui::Limits::EMPTY, 512).unwrap().patches);
        crate::build(&mut view, &tui, &root, wall, &mut out);
        assert_eq!(!out.is_empty(), changes);
        assert_eq!(view.tick(), None);
    }
    let mut root = asked(Some(State::Asked { code: code(), expires: Wall::from_nanos(600_000_000_000) }));
    let mut view = f::view();
    for (nanos, span, tick, changes) in [
        (0, "The code expires in 10 min.", Some(1), true),
        (1, "The code expires in 9 min.", Some(60_000_000_001), true),
        (59_999_999_999, "The code expires in 9 min.", Some(60_000_000_001), false),
        (60_000_000_000, "The code expires in 9 min.", Some(60_000_000_001), false),
        (60_000_000_001, "The code expires in 8 min.", Some(120_000_000_001), true),
        (540_000_000_000, "The code expires in 1 min.", Some(540_000_000_001), true),
        (540_000_000_001, "The code expires in under 1 min.", None, true),
        (600_000_000_000, "The code expires in under 1 min.", None, false),
        (600_000_000_001, "The code expires in under 1 min.", None, false),
    ] {
        let mut out = Queue::uncapped();
        crate::build(&mut view, &tui, &root, Wall::from_nanos(nanos), &mut out);
        assert_eq!(!out.is_empty(), changes, "wall {nanos}");
        f::find(view.tree(), skein_ui::Role::Paragraph, span.as_bytes());
        assert_eq!(
            view.tick(),
            match tick {
                Some(tick) => Some(Wall::from_nanos(tick)),
                None => None,
            },
            "wall {nanos}"
        );
    }
    // A changed root or TUI replaces the countdown and cancels its wake.
    let mut view = f::view();
    f::build(&mut view, &tui, &root);
    assert!(view.tick().is_some());
    outcome(&mut root, Some(NotSignedIn::Approval(Taking::Expired)));
    f::build(&mut view, &tui, &root);
    assert_eq!(view.tick(), None);
    let root = asked(Some(State::Asked { code: code(), expires: Wall::from_nanos(86_400_000_000_007) }));
    let mut view = f::view();
    for (nanos, span, tick) in [
        (0, "The code expires in 1 d.", 8),
        (7, "The code expires in 1 d.", 8),
        (8, "The code expires in 23 h 59 min.", 60_000_000_008),
    ] {
        let mut out = Queue::uncapped();
        crate::build(&mut view, &tui, &root, Wall::from_nanos(nanos), &mut out);
        f::find(view.tree(), skein_ui::Role::Paragraph, span.as_bytes());
        assert_eq!(view.tick(), Some(Wall::from_nanos(tick)));
    }
    let mut tui = tui;
    tui.step(Told::Asked);
    f::build(&mut view, &tui, &root);
    assert_eq!(view.tick(), None);
}
