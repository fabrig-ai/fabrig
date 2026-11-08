use super::fixtures as f;
use crate::{Local, Refusal, Signing, Told, Tui, look};
use jig_client::Link;
use jig_client_protocol_wire::SignInAnswer as Answer;
use skein_ui::{Role, Style};

fn signing(tui: &mut Tui, signing: &Signing) {
    match signing {
        Signing::Idle => {}
        Signing::Refused(refusal) => tui.step(Told::Refused { refusal: refusal.clone() }),
        Signing::Asked => tui.step(Told::Asked),
        Signing::Answered { lost } => {
            tui.step(Told::Asked);
            tui.step(Told::Answered { lost: *lost });
        }
    }
}
#[test]
fn the_sign_in_screen_says_each_line() {
    type Row<'a> = (Signing, Option<Answer>, Local, Option<&'a [u8]>, Option<Style>);
    let rows: &[Row<'_>] = &[
        (Signing::Idle, None, Local::Idle, None, None),
        (Signing::Idle, Some(Answer::Busy), Local::Idle, None, None),
        (Signing::Refused(Refusal::NotALink), None, Local::Idle, Some(b"That is not a sign-in link: paste the whole link temper printed."), Some(look::REFUSED)),
        (Signing::Refused(Refusal::OtherAddress { address: Box::from(*b"localhost:8367") }), Some(Answer::Busy), Local::Idle, Some(b"That link is for localhost:8367. To sign in there, run temper http://localhost:8367."), Some(look::REFUSED)),
        (Signing::Asked, None, Local::Idle, Some("Signing in…".as_bytes()), Some(look::WAITING)),
        (Signing::Answered { lost: true }, Some(Answer::LinkUsed), Local::Idle, Some(b"Signing in: no answer from the engine; trying again."), Some(look::WAITING)),
        (Signing::Answered { lost: false }, Some(Answer::Busy), Local::Idle, Some(b"Signing in: the engine is busy; trying again."), Some(look::WAITING)),
        (Signing::Answered { lost: false }, Some(Answer::NotReady), Local::Idle, Some(b"Signing in: the engine is starting or stopping; trying again."), Some(look::WAITING)),
        (Signing::Answered { lost: false }, Some(Answer::LinkUsed), Local::Idle, Some(b"This link was already used. A fresh one is in your terminal."), Some(look::REFUSED)),
        (Signing::Answered { lost: false }, Some(Answer::LinkExpired), Local::Idle, Some(b"This link has expired. A fresh one is in your terminal."), Some(look::REFUSED)),
        (Signing::Answered { lost: false }, Some(Answer::LinkUnknown), Local::Idle, Some(b"This link isn't from this temper. Use the one its terminal printed."), Some(look::REFUSED)),
        (Signing::Answered { lost: false }, Some(f::full()), Local::Idle, Some(b"All 4294967295 sign-ins are in use; the earliest ends Jan 1, 00:00 UTC. This link stays usable until Jan 1, 00:00 UTC; after that, opening it has temper serve print a fresh one."), Some(look::REFUSED)),
        (Signing::Answered { lost: false }, None, Local::Idle, Some(b"This link was refused. Use the fresh one temper printed."), Some(look::REFUSED)),
        (Signing::Answered { lost: false }, Some(Answer::LinkUnknown), Local::Ended, None, None),
        (Signing::Answered { lost: false }, Some(Answer::LinkUsed), Local::Ended, None, None),
        (Signing::Answered { lost: false }, Some(Answer::NotReady), Local::Ended, None, None),
    ];
    let mut failures = skein_lib::List::with_capacity(16);
    for (index, (state, answer, local, line, style)) in rows.iter().enumerate() {
        let root = match answer {
            Some(answer) => f::refused(answer.clone()),
            None => f::signed_out(),
        };
        let mut tui = Tui::new(f::ADDRESS);
        signing(&mut tui, state);
        tui.step(Told::Local { local: *local });
        let mut view = f::view();
        f::build(&mut view, &tui, &root);
        let paragraphs = f::count(view.tree(), Role::Paragraph).checked_sub(1).unwrap();
        let expected = if line.is_some() { 3 } else { 2 };
        if paragraphs != expected {
            failures.push((index, paragraphs, expected)).unwrap();
        }
        if let Some(line) = line {
            let node = f::find(view.tree(), Role::Paragraph, line);
            assert_eq!(node.styles.as_slice(), &[style.unwrap()], "row {index}");
        }
        let field = f::find(view.tree(), Role::TextField, b"Link");
        assert_eq!(field.entry, Some(skein_ui::EntryKind::Secret));
        assert!(field.states.as_slice().contains(&skein_ui::State::SubmitsOnEnter));
        assert_eq!(field.focus, Some(if *local == Local::Ended { 2 } else { 1 }));
        assert_eq!(field.placeholder.as_deref(), Some("http://127.0.0.1:8367/jig/v1/sign-in/link?secret=…".as_bytes()));
        assert_eq!(f::find(view.tree(), Role::Status, "● Signed out".as_bytes()).styles.as_slice(), &[look::DOWN]);
        assert_eq!(f::count(view.tree(), Role::Button), 0);
    }
    assert!(failures.is_empty(), "sign-in rows: {failures:?}");
    let root = f::signed_out();
    let mut tui = Tui::new(f::ADDRESS);
    tui.step(Told::Link { link: Link::Live });
    tui.step(Told::Link { link: Link::SignedOut });
    let mut view = f::view();
    f::build(&mut view, &tui, &root);
    let signed_out = f::find(view.tree(), Role::Paragraph, b"Signed out.");
    assert_eq!(signed_out.styles.as_slice(), &[look::DOWN]);
    let address = f::find(view.tree(), Role::Paragraph, b"Sign in this terminal to 127.0.0.1:8367");
    let before = f::index(view.tree(), signed_out.id);
    let after = f::index(view.tree(), address.id);
    assert!(before < after);
    assert_eq!(f::find(view.tree(), Role::TextField, b"Link").focus, Some(3));
    // A refusal can change while the link stays SignedOut and Tui stays still.
    let (mut root, exchange) = f::asking();
    let mut tui = Tui::new(f::ADDRESS);
    signing(&mut tui, &Signing::Answered { lost: false });
    let mut view = f::view();
    f::build(&mut view, &tui, &root);
    let before = temper_client_domain::shown(&root);
    f::answer(&mut root, exchange, Answer::Busy);
    assert_eq!(root.client().link().state, Link::SignedOut);
    assert!(temper_client_domain::shown(&root) > before);
    assert!(!f::build(&mut view, &tui, &root).is_empty());
    f::find(view.tree(), Role::Paragraph, b"Signing in: the engine is busy; trying again.");
}

#[test]
#[expect(clippy::too_many_lines, reason = "the design's approval outcome table and ordered screen lines")]
fn the_approval_screen_says_each_line() {
    use super::approval::{asked, code, exchange, outcome};
    use crate::Browser;
    use jig_client::{NotAsked, NotSignedIn, Taking};
    use skein_lib::{Secret, Wall};
    use temper_client_domain::asked::State;
    type Row<'a> = (State, Option<NotSignedIn>, Option<Browser>, bool, bool, &'a [&'a str]);
    let waiting = State::Asked { code: code(), expires: Wall::from_nanos(600_000_000_000) };
    let opened = [
        "A tab opened in your browser. Check it shows",
        "WDJB-MJHT, then press Approve.",
        "No tab? Open http://127.0.0.1:8367/terminal?code=WDJB-MJHT",
        "The code expires in 10 min.",
    ];
    let no_browser = [
        "Open this address in a browser that reaches temper:",
        "http://127.0.0.1:8367/terminal?code=WDJB-MJHT",
        "and check it shows WDJB-MJHT, then press Approve.",
        "From another machine, tunnel port 8367 to this one first.",
        "The code expires in 10 min.",
    ];
    let rows: &[Row<'_>] = &[
        (
            waiting,
            Some(NotSignedIn::Approval(Taking::Refused)),
            None,
            false,
            false,
            &["The browser refused this sign-in. Press Enter to ask again."],
        ),
        (
            waiting,
            Some(NotSignedIn::Approval(Taking::Expired)),
            None,
            false,
            false,
            &["The code expired before it was approved. Press Enter to ask again."],
        ),
        (
            waiting,
            Some(NotSignedIn::Approval(Taking::Used)),
            None,
            false,
            false,
            &["That code was already used. Press Enter to ask again."],
        ),
        (
            waiting,
            Some(NotSignedIn::Approval(Taking::Unknown)),
            None,
            false,
            false,
            &["temper doesn't know that code any more: it restarted, or the code is old. Press Enter to ask again."],
        ),
        (
            waiting,
            Some(NotSignedIn::Full { sign_ins: u32::MAX, earliest: Wall::EPOCH, link_until: Wall::EPOCH }),
            None,
            false,
            false,
            &["All 4294967295 sign-ins are in use; the earliest ends Jan 1, 00:00 UTC. Press Enter to ask again then."],
        ),
        (waiting, Some(NotSignedIn::Busy), None, false, false, &["temper is busy; still trying…"]),
        (waiting, Some(NotSignedIn::NotReady), None, false, false, &["temper is busy; still trying…"]),
        (waiting, None, None, false, true, &["temper is busy; still trying…"]),
        (
            State::NotAsked(NotAsked::Refused),
            None,
            None,
            false,
            false,
            &["This temper doesn't sign terminals in from the browser. Paste the link it printed instead."],
        ),
        (
            State::NotAsked(NotAsked::Busy),
            None,
            None,
            false,
            false,
            &["Too many terminals are waiting to sign in. Press Enter to ask again in a moment."],
        ),
        (
            State::NotAsked(NotAsked::NotReady),
            None,
            None,
            false,
            false,
            &["temper is starting or stopping, or out of reach. Press Enter to ask again."],
        ),
        (
            State::NotAsked(NotAsked::Lost),
            None,
            None,
            false,
            false,
            &["temper is starting or stopping, or out of reach. Press Enter to ask again."],
        ),
        (State::Asking, None, None, false, false, &["Asking temper for a code…"]),
        (waiting, Some(NotSignedIn::Approval(Taking::Waiting)), Some(Browser::Opening), false, false, &opened),
        (waiting, Some(NotSignedIn::Approval(Taking::Waiting)), Some(Browser::Opened), false, false, &opened),
        (waiting, None, None, false, false, &opened),
        (waiting, Some(NotSignedIn::Approval(Taking::Waiting)), Some(Browser::None), false, false, &no_browser),
        (waiting, None, Some(Browser::Opened), true, false, &["Signing in…"]),
    ];
    for (index, (state, why, browser, pasted, lost, expected)) in rows.iter().enumerate() {
        let mut root = asked(Some(*state));
        outcome(&mut root, *why);
        if *lost {
            let id = exchange(f::client(
                &mut root,
                jig_client::Event::SignIn {
                    with: jig_client::With::Approval { secret: Secret::new(Box::new([7; 32])) },
                },
            ));
            f::client(&mut root, jig_client::Event::Lost { exchange: id });
        }
        let mut tui = Tui::new(f::ADDRESS);
        if let Some(browser) = browser {
            tui.step(Told::Browse);
            match browser {
                Browser::Opening => {}
                Browser::Opened => tui.step(Told::Opened { opened: true }),
                Browser::None => tui.step(Told::Opened { opened: false }),
            }
        }
        if *pasted {
            tui.step(Told::Asked);
        }
        let mut view = f::view();
        f::build(&mut view, &tui, &root);
        let mut lines = skein_lib::List::with_capacity(8);
        for node in view.tree().nodes() {
            if node.role == Role::Paragraph {
                lines.push(f::text(view.tree(), node)).unwrap();
            }
        }
        let mut wanted = skein_lib::List::with_capacity(8);
        wanted.push(b"Sign in this terminal to 127.0.0.1:8367".as_slice()).unwrap();
        if !*pasted {
            for line in *expected {
                wanted.push(line.as_bytes()).unwrap();
            }
        }
        wanted.push(b"Paste the link temper printed, then Enter.").unwrap();
        if *pasted {
            for line in *expected {
                wanted.push(line.as_bytes()).unwrap();
            }
        }
        wanted.push(b"Ctrl+C to quit").unwrap();
        assert_eq!(lines.len(), wanted.len(), "row {index}");
        for (line, wanted) in lines.as_slice().iter().zip(wanted.as_slice()) {
            assert_eq!(line.as_ref(), *wanted, "row {index}");
        }
        if expected.len() == 1 && !*pasted {
            let node = f::find(view.tree(), Role::Paragraph, expected[0].as_bytes());
            let style = if *state == State::Asking
                || *lost
                || *why == Some(NotSignedIn::Busy)
                || *why == Some(NotSignedIn::NotReady)
            {
                look::WAITING
            } else {
                look::REFUSED
            };
            assert_eq!(node.styles.as_slice(), &[style], "row {index}");
        }
        if expected.len() > 1 {
            assert_eq!(
                f::find(view.tree(), Role::Paragraph, b"The code expires in 10 min.").styles.as_slice(),
                &[look::HINT]
            );
        } else {
            assert_eq!(view.tick(), None, "row {index}");
        }
    }
    let root = asked(Some(waiting));
    let mut tui = Tui::new(f::ADDRESS);
    tui.step(Told::Asked);
    tui.step(Told::Browse);
    let mut view = f::view();
    f::build(&mut view, &tui, &root);
    assert_eq!(f::count(view.tree(), Role::Paragraph), 7, "a new approval replaces the paste's progress");
    f::find(view.tree(), Role::Paragraph, opened[0].as_bytes());
    tui.step(Told::Refused { refusal: Refusal::NotALink });
    f::build(&mut view, &tui, &root);
    assert_eq!(f::count(view.tree(), Role::Paragraph), 8, "a local paste refusal leaves the approval drawn");
    f::find(view.tree(), Role::Paragraph, opened[0].as_bytes());
    f::find(view.tree(), Role::Paragraph, b"That is not a sign-in link: paste the whole link temper printed.");
}
