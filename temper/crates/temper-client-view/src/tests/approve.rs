//! The approval column and frame (temper/docs/design/client/pages/README.md and temper/docs/design/client/view.md, 1 and 6).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary view tests inspect semantic trees and compare collected words"
)]
use super::fixtures as f;
use crate::{look, sign_in::Ways};
use jig_client::{Approval, ApprovalCode, ApprovalState as Read, Decided};
use skein_lib::{Queue, Wall};
use skein_ui::{Role, Style, Tree};
use temper_client_domain::{
    Action, Domain, Event, Place,
    approve::{Act, State},
};

const TWO_MINUTES: Wall = Wall::from_nanos(120_000_000_000);
const THREE_MINUTES: Wall = Wall::from_nanos(180_000_000_000);
const ASK: &str = "Sign in a terminal?";
const WARNING: &str = "Approve only a code your own terminal shows.";
const UNREAD: &str = "That isn’t a terminal’s code. Type the eight letters your terminal shows.";
fn code() -> ApprovalCode {
    ApprovalCode::parse(b"WDJB-MJHT").expect("fixture code")
}
fn act(root: &mut Domain, act: Act) {
    f::root_event(root, Event::Act { action: Action::Approve(act) });
}
fn read(root: &mut Domain, state: Read) {
    f::root_event(
        root,
        Event::Client { event: jig_client::Event::ApprovalAnswer { answer: Approval::Read { code: code(), state } } },
    );
}
fn reached(state: State) -> Domain {
    let mut root = f::root_in_project(None);
    f::root_event(
        &mut root,
        Event::Went {
            place: Place::Approve { code: if let State::Typing { .. } = state { None } else { Some(code()) } },
        },
    );
    match state {
        State::Typing { unread } => {
            act(
                &mut root,
                Act::Words {
                    words: if unread { b"WDJB-MJH".as_slice().into() } else { b"wdjb mjht".as_slice().into() },
                },
            );
            if unread {
                act(&mut root, Act::Check);
            }
        }
        State::Reading => {}
        State::Asking { asked } => read(&mut root, Read::Waiting { asked }),
        State::Deciding { approve, asked } => {
            read(&mut root, Read::Waiting { asked });
            act(&mut root, if approve { Act::Approve } else { Act::Refuse });
        }
        State::Approved => read(&mut root, Read::Approved),
        State::SignedIn => read(&mut root, Read::Taken),
        State::Refused => read(&mut root, Read::Refused),
        State::Expired => read(&mut root, Read::Expired),
        State::Used => {
            read(&mut root, Read::Waiting { asked: Wall::EPOCH });
            act(&mut root, Act::Approve);
            f::root_event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::ApprovalAnswer {
                        answer: Approval::Decided { code: code(), decided: Decided::Used },
                    },
                },
            );
        }
        State::Unknown => read(&mut root, Read::Unknown),
        State::SignedOut => f::root_event(&mut root, Event::SignedOut),
        State::Busy => read(&mut root, Read::Busy),
    }
    assert_eq!(root.approval().expect("page").state(), state);
    root
}
fn named(tree: &Tree, role: Role, name: &str) -> skein_ui::NodeId {
    f::find(tree, role, name.as_bytes()).unwrap_or_else(|| panic!("missing {role:?} {name}"))
}
fn parent(tree: &Tree, id: skein_ui::NodeId) -> &skein_ui::Node {
    let at = tree.nodes().iter().position(|n| n.id == id).expect("child index");
    tree.nodes()[..at]
        .iter()
        .enumerate()
        .rev()
        .find(|(index, n)| index.checked_add(usize::try_from(n.size).expect("subtree size")).expect("subtree end") > at)
        .map(|(_, n)| n)
        .expect("node parent")
}
fn build(root: &Domain) -> crate::View {
    build_with_ways(root, Ways::NONE)
}
fn build_with_ways(root: &Domain, ways: Ways) -> crate::View {
    let mut view = crate::View::with_sign_in(
        &crate::Limits { tree: f::tree_limits() },
        &f::domain_limits(),
        f::browser_words(),
        f::FORGE,
        ways,
    );
    crate::build(&mut view, root, TWO_MINUTES, &mut Queue::uncapped());
    view
}
#[test]
#[expect(
    clippy::too_many_lines,
    clippy::type_complexity,
    reason = "one state table holds the approval column words, shapes, controls, return links and clock wake"
)]
fn each_state_draws_its_column() {
    let asked = Wall::EPOCH;
    let rows: &[(State, &str, &[&str], &[(Role, &str, &[Style], Role)])] = &[
        (
            State::Typing { unread: false },
            ASK,
            &["Type the code your terminal shows."],
            &[
                (Role::TextField, "Code", &[look::APPROVE_FIELD], Role::Form),
                (Role::Button, "Check", &[look::OUTSIDE_PRIMARY], Role::Form),
            ],
        ),
        (
            State::Typing { unread: true },
            ASK,
            &["Type the code your terminal shows.", UNREAD],
            &[
                (Role::TextField, "Code", &[look::APPROVE_FIELD], Role::Form),
                (Role::Button, "Check", &[look::OUTSIDE_PRIMARY], Role::Form),
                (Role::Alert, UNREAD, &[], Role::Form),
            ],
        ),
        (State::Reading, ASK, &["Reading WDJB-MJHT…"], &[]),
        (
            State::Asking { asked },
            ASK,
            &["WDJB-MJHT", "Asked 2 min ago", WARNING],
            &[
                (Role::Button, "Approve", &[look::OUTSIDE_PRIMARY], Role::Slot),
                (Role::Button, "Refuse", &[look::OUTSIDE_SECONDARY], Role::Slot),
            ],
        ),
        (
            State::Deciding { approve: true, asked },
            ASK,
            &["WDJB-MJHT", "Asked 2 min ago", WARNING, "Approving…"],
            &[(Role::Status, "Approving…", &[], Role::Slot)],
        ),
        (
            State::Deciding { approve: false, asked },
            ASK,
            &["WDJB-MJHT", "Asked 2 min ago", WARNING, "Refusing…"],
            &[(Role::Status, "Refusing…", &[], Role::Slot)],
        ),
        (State::Approved, "Approved", &["WDJB-MJHT", "Waiting for your terminal to sign in…"], &[]),
        (State::SignedIn, "Terminal signed in", &["Signed in. You can close this tab."], &[]),
        (State::Refused, "Sign-in refused", &["This terminal’s sign-in was refused. Nothing was signed in."], &[]),
        (State::Expired, "Code expired", &["This code has expired. Ask again in your terminal."], &[]),
        (
            State::Used,
            "Code already used",
            &["This code was already approved or refused. If your terminal isn’t signed in, ask again there."],
            &[],
        ),
        (
            State::Unknown,
            "Unknown code",
            &["temper doesn’t know this code. Check the code your terminal shows, or ask again there."],
            &[(Role::Button, "Type a code", &[look::OUTSIDE_PRIMARY], Role::Main)],
        ),
        (
            State::SignedOut,
            "Sign in first",
            &[
                "This browser isn’t signed in to temper. Open the sign-in link temper printed first, then open this page’s address again: your terminal shows it.",
            ],
            &[],
        ),
        (
            State::Busy,
            "temper couldn’t answer just now",
            &["It is starting or stopping, busy, or out of reach. Try again in a moment."],
            &[(Role::Button, "Try again", &[look::OUTSIDE_PRIMARY], Role::Main)],
        ),
    ];
    for (state, heading, sentences, acts) in rows {
        let root = reached(*state);
        let mut view = build(&root);
        let tree = view.tree();
        named(tree, Role::Heading { level: 1 }, heading);
        let text = String::from_utf8(f::main_text(tree).into_vec()).expect("UTF-8");
        for sentence in *sentences {
            assert!(text.contains(sentence), "{state:?}: missing {sentence}: {text}");
        }
        for (role, name, styles, parent_role) in *acts {
            let id = named(tree, *role, name);
            let node = tree.find(id).expect("named control");
            assert_eq!(node.styles.as_slice(), *styles, "{state:?}: {name}");
            let parent = parent(tree, id);
            assert_eq!(parent.role, *parent_role, "{state:?}: {name} parent");
            if *parent_role == Role::Slot {
                assert_eq!(parent.styles.as_slice(), &[look::OUTSIDE_ACTS]);
            }
        }
        let main = tree.nodes().iter().position(|n| n.role == Role::Main).expect("main");
        assert_eq!(tree.nodes()[main].styles.as_slice(), &[look::OUTSIDE, look::APPROVE]);
        let end = main + usize::try_from(tree.nodes()[main].size).expect("main size");
        for node in &tree.nodes()[main..end] {
            for old in [look::MAIN, look::CARD, look::BRAND] {
                assert!(!node.styles.as_slice().contains(&old), "{state:?}: absent {old:?}");
            }
        }
        let buttons: Vec<_> = tree.nodes()[main..end]
            .iter()
            .filter(|n| n.role == Role::Button)
            .map(|n| n.name.as_deref().expect("button name"))
            .collect();
        let expected: Vec<_> =
            acts.iter().filter(|(r, ..)| *r == Role::Button).map(|(_, name, ..)| name.as_bytes()).collect();
        assert_eq!(buttons, expected, "{state:?}: only this state's acts");
        if let State::Typing { unread } = state {
            let form = tree.find(named(tree, Role::Form, "Code")).expect("form");
            let at = tree.nodes().iter().position(|n| n.id == form.id).expect("form index");
            let end = at + usize::try_from(form.size).expect("form subtree");
            let roles: Vec<_> = tree.nodes()[at..end].iter().filter(|n| n.role != Role::Text).map(|n| n.role).collect();
            let expected: &[Role] = if *unread {
                &[Role::Form, Role::TextField, Role::Button, Role::Alert]
            } else {
                &[Role::Form, Role::TextField, Role::Button]
            };
            assert_eq!(roles, expected, "the alert follows Check inside the form");
            let field = tree.find(named(tree, Role::TextField, "Code")).expect("field");
            assert_eq!(
                field.value.as_ref().expect("value").text.as_ref(),
                if *unread { b"WDJB-MJH".as_slice() } else { b"wdjb mjht" }
            );
            assert!(field.states.as_slice().contains(&skein_ui::State::SubmitsOnEnter));
        }
        if let State::Asking { .. } | State::Deciding { .. } = state {
            assert_eq!(view.next_change(), Some(THREE_MINUTES));
            let revision = view.revision();
            crate::build(&mut view, &root, THREE_MINUTES, &mut Queue::uncapped());
            assert_ne!(view.revision(), revision, "the clock wakes without a domain event");
            assert!(
                String::from_utf8(f::main_text(view.tree()).into_vec()).expect("UTF-8").contains("Asked 3 min ago")
            );
        }
    }
    let printed = "This browser isn’t signed in to temper. Open the sign-in link temper printed first, then open this page’s address again: your terminal shows it.";
    let google = "This browser isn’t signed in to temper. Sign in, and this page comes back with your terminal’s code.";
    let origin = b"https://temper.example".as_slice();
    for (ways, code, body, target) in [
        (Ways::NONE, Some(code()), printed, None),
        (Ways { google: false, origin: Some(origin.into()) }, Some(code()), printed, None),
        (Ways { google: true, origin: None }, Some(code()), printed, None),
        (
            Ways::parse("google", origin),
            Some(code()),
            google,
            Some("https://temper.example/sign-in/google?then=/terminal%3Fcode%3DWDJB-MJHT"),
        ),
        (Ways::parse("google", origin), None, google, Some("https://temper.example/sign-in/google?then=/terminal")),
    ] {
        let mut root = reached(State::SignedOut);
        f::root_event(&mut root, Event::Went { place: Place::Approve { code } });
        let view = build_with_ways(&root, ways);
        let tree = view.tree();
        named(tree, Role::Heading { level: 1 }, "Sign in first");
        assert_eq!(
            f::main_text(tree).as_ref(),
            ["Sign in first", body, if target.is_some() { "Continue with Google" } else { "" }].concat().as_bytes()
        );
        let links: Vec<_> = tree
            .nodes()
            .iter()
            .filter(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Continue with Google"))
            .collect();
        let targets: Vec<_> = links.iter().map(|n| n.external.as_deref().expect("Google destination")).collect();
        assert_eq!(targets, target.into_iter().map(str::as_bytes).collect::<Vec<_>>());
        for link in links {
            assert_eq!(link.styles.as_slice(), &[look::OUTSIDE_PRIMARY]);
            assert_eq!(parent(tree, link.id).role, Role::Main);
        }
    }
}

#[test]
fn each_act_decodes_to_its_action() {
    for (state, role, name, event, expected) in [
        (
            State::Typing { unread: false },
            Role::TextField,
            "Code",
            skein_ui::Act::Input { value: b"wdjb mjht".as_slice().into() },
            Act::Words { words: b"wdjb mjht".as_slice().into() },
        ),
        (State::Typing { unread: false }, Role::TextField, "Code", skein_ui::Act::Submit, Act::Check),
        (State::Typing { unread: false }, Role::Button, "Check", skein_ui::Act::Press, Act::Check),
        (State::Asking { asked: Wall::EPOCH }, Role::Button, "Approve", skein_ui::Act::Press, Act::Approve),
        (State::Asking { asked: Wall::EPOCH }, Role::Button, "Refuse", skein_ui::Act::Press, Act::Refuse),
        (State::Unknown, Role::Button, "Type a code", skein_ui::Act::Press, Act::Type),
        (State::Busy, Role::Button, "Try again", skein_ui::Act::Press, Act::Again),
    ] {
        let mut view = build(&reached(state));
        let id = named(view.tree(), role, name);
        assert_eq!(crate::act(&mut view, id, event), Some(Action::Approve(expected)), "{name}");
        for wrong in [skein_ui::Act::Toggle, skein_ui::Act::Near { edge: skein_ui::LogEdge::Head }] {
            assert_eq!(crate::act(&mut view, id, wrong), None, "{name}: unrelated events");
        }
    }
}
#[test]
fn the_approval_place_draws_the_banner_alone() {
    use jig_client::contract as c;
    use skein_lib::List;
    let (mut accounts, watch) = f::account_root();
    f::root_event(
        &mut accounts,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(c::Accounts {
                        program_hosts: List::with_capacity(0),
                        accounts: List::with_capacity(0),
                        signing: List::with_capacity(0),
                        lendable: false,
                        lending: None,
                        lent_end: None,
                        last_end: None,
                        usable: false,
                        settled: true,
                        cut: false,
                        readiness: List::with_capacity(0),
                        readiness_cut: false,
                    }))),
                },
            },
        },
    );
    f::root_event(
        &mut accounts,
        Event::Went { place: Place::Address(temper_client_domain::Address::NewChat { draft: false }) },
    );
    let playground = f::playground_root(
        temper_client_domain::Address::Chats { filter: temper_client_domain::ChatsFilter::All },
        f::PROJECT,
        false,
    );
    for mut root in [Box::new(accounts), Box::new(playground)] {
        let before = build(&root);
        assert!(before.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::RAIL)));
        assert!(
            before.tree().nodes().iter().any(|n| n.styles.as_slice().contains(&look::ACCOUNTS)),
            "fixture offers a banner"
        );
        f::root_event(&mut root, Event::Went { place: Place::Approve { code: None } });
        for unavailable in [false, true] {
            if unavailable {
                f::root_event(&mut root, Event::SignedOut);
            }
            let view = build(&root);
            let tree = view.tree();
            assert!(tree.nodes().iter().any(|n| n.role == Role::Banner));
            for style in [look::RAIL, look::TABBAR, look::ACCOUNTS] {
                assert!(!tree.nodes().iter().any(|n| n.styles.as_slice().contains(&style)), "absent {style:?}");
            }
            assert!(f::find(tree, Role::Region, b"Playground").is_none());
            assert!(
                f::find(tree, Role::Heading { level: 1 }, if unavailable { b"Sign in first" } else { ASK.as_bytes() })
                    .is_some()
            );
        }
    }
}
