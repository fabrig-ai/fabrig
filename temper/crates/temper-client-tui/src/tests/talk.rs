//! Talk frames and prompts (client/tui/pages/talk.md, 1).
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary state tests inspect text and collect frame diagnostics"
)]
use super::{chat, fixtures as f, goldens, launchpad};
use crate::{
    Event, Own, Scheme, Told, Tui,
    view::{self, foot},
};
use jig_client::contract as c;
use skein_lib::{List, Queue, Wall};
use skein_ui::Role;
use skein_ui_screen::{Focus, Size};
use temper_client_domain::{self as d, Address, Domain};

const WALL: Wall = Wall::from_nanos(9 * 60_000_000_000);
fn build(root: &Domain, tui: &Tui) -> crate::View {
    let mut view = f::view();
    crate::build(&mut view, tui, root, WALL, &mut Queue::uncapped());
    view
}
fn text(view: &crate::View) -> String {
    let mut out = String::new();
    for node in view.tree().nodes() {
        if node.role == Role::Paragraph || node.role == (Role::Heading { level: 1 }) {
            out.push_str(core::str::from_utf8(&f::text(view.tree(), node)).unwrap());
            out.push('\n');
        }
    }
    out
}
#[test]
fn the_talk_head_conversation_and_quiet_line_follow_its_facts() {
    for (case, period, now, conversation, cost) in [
        ("open", "this week 49 of 60", true, true, true),
        ("period", "this period 49 of 60", true, true, true),
        ("no Now", "this week 49 of 60", false, true, true),
        ("closed", "this week 49 of 60", true, false, false),
        ("unknown cost", "this week 49 of 60", true, true, false),
    ] {
        let root = chat::talk_root(case);
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Size { columns: 80 });
        // The quiet line is read with the role's choices set aside.
        tui.step(Told::ReplyAside { turn: chat::REPLY_TURN });
        let head = view::talk::read(&root, &tui, &f::words()).unwrap();
        let tail = if case == "period" { " · Hello's triager" } else { "" };
        assert_eq!(
            core::str::from_utf8(&head.line).unwrap(),
            format!(" · a standing role · owns vsr-rs{tail} · {period}")
        );
        let part = foot::read(&root, &tui, &f::words(), WALL);
        assert!(part.rule.is_none(), "{case}: the rule has no title");
        assert_eq!(
            part.talk_session.as_deref(),
            if conversation {
                Some(if cost { "this session 9 min · 42".as_bytes() } else { b"this session 9 min" })
            } else {
                None
            },
            "{case}"
        );
        assert!(part.state.is_empty(), "Live is quiet");
        assert!(part.closed.is_none(), "no model picker");
        let placeholder = core::str::from_utf8(&part.composer.as_ref().unwrap().placeholder).unwrap();
        assert_eq!(
            placeholder,
            if conversation {
                "Reply, or Esc to leave. Porter writes its Now when you go."
            } else {
                "Talk to Porter, or Esc to go back."
            }
        );
        let view = build(&root, &tui);
        let shown = text(&view);
        assert!(shown.contains("temper · Hello · talking with Porter"), "{case}: {shown}");
        assert!(
            shown.contains(&format!("Porter · a standing role · owns vsr-rs · {period}")) || case == "period",
            "{case}: {shown}"
        );
        if case == "period" {
            assert!(shown.contains("Hello's triager · this period"), "{shown}");
        }
        assert_eq!(shown.contains("Its Now, 06:58:"), now, "{case}: {shown}");
        assert!(!shown.contains("Earlier activity") || case == "unknown cost", "{case}: {shown}");
        assert_eq!(shown.contains("Fine. From now on"), conversation, "{case}: {shown}");
        assert!(view.tree().nodes().iter().any(
            |node| node.role == Role::Status && f::text(view.tree(), node).as_ref() == "Hello · Porter".as_bytes()
        ));
        assert_eq!(crate::escape(&view), Some(Event::Leave));
        assert_eq!(
            crate::offered(&root, &f::words()).as_ref(),
            &[
                d::pages::terminal::Command::New,
                d::pages::terminal::Command::Chats,
                d::pages::terminal::Command::Help,
                d::pages::terminal::Command::Exit
            ]
        );
        assert_eq!(crate::offered_own(&root, &tui).as_ref(), &[Own::Open, Own::Tell, Own::Theme]);
        assert_eq!(crate::focus(&view), Some(b"http://127.0.0.1:8367/projects/1/team?role=2".as_slice()));
    }
    let (mut root, _) = f::live(Some(f::me(&[b"Hello"])));
    f::event(&mut root, d::Event::Went { place: d::Place::Address(Address::Talk { project: 1, task: 7 }) });
    let shown = text(&build(&root, &Tui::new(f::ADDRESS)));
    assert!(shown.contains("talking with role T7"), "{shown}");
}
#[test]
fn the_roles_reply_choices_win_over_a_proposal_without_a_document() {
    let root = chat::talk_root("proposal");
    let tui = Tui::new(f::ADDRESS);
    let view = build(&root, &tui);
    assert_eq!(crate::reply_number(&view, b"1"), Some(Event::Answer { words: b"Yes, add it".as_slice().into() }));
    assert_eq!(crate::escape(&view), Some(Event::ReplyAside { turn: chat::REPLY_TURN }));
    assert!(root.pages().chat().unwrap().draft.is_none());
    let mut tui = tui;
    tui.step(Told::ThemeList { open: true });
    let view = build(&root, &tui);
    assert_eq!(crate::escape(&view), Some(Event::Theme { choice: None }));
    assert_eq!(crate::reply_number(&view, b"1"), None, "a command list wins over choices");
}
#[test]
fn a_proposal_is_accepted_or_set_aside_for_that_card() {
    let root = chat::talk_root("proposal");
    let mut tui = Tui::new(f::ADDRESS);
    tui.step(Told::ReplyAside { turn: chat::REPLY_TURN });
    let part = foot::read(&root, &tui, &f::words(), WALL);
    let list = part.list.unwrap();
    assert_eq!(list.head.as_deref(), Some(b"Use plain conditions".as_slice()));
    let object = c::ObjectKey::Proposal(c::ProposalKey { proposer: c::Proposer::Party(11), number: 13 });
    assert_eq!(list.items[0].name.as_ref(), b"1. Accept it");
    assert_eq!(list.items[0].event, Event::Accept { object, revision: None });
    assert_eq!(list.items[1].name.as_ref(), b"2. Not now");
    assert_eq!(list.items[1].event, Event::Aside { object });
    assert_eq!(crate::escape(&build(&root, &tui)), Some(Event::Aside { object }));
    tui.step(Told::ProposalAside { object });
    assert!(foot::read(&root, &tui, &f::words(), WALL).list.is_none());
    tui.step(Told::Went);
    assert!(tui.proposal_aside(&object), "returning never raises that card again");
}
#[test]
fn talk_lists_roles_in_team_order_with_areas_or_now() {
    let root = launchpad::team::root(launchpad::team::Fixture {
        roles: Some(&[b"Prover", b"Triager"]),
        areas: true,
        you: true,
    });
    let mut tui = Tui::new(f::ADDRESS);
    assert!(crate::offered_own(&root, &tui).windows(3).any(|items| items == [Own::Change, Own::Talk, Own::Ask]));
    let view = build(&root, &tui);
    assert!(text(&view).contains("/talk     a role, about its area"), "{}", text(&view));
    tui.step(Told::TalkList { open: true });
    let list = foot::read(&root, &tui, &f::words(), WALL).list.unwrap();
    assert_eq!(list.head.as_deref(), Some(b"Talk with a role on Hello".as_slice()));
    assert_eq!(list.items[0].name.as_ref(), b"Prover");
    assert_eq!(list.items[0].words.as_ref(), b"tests");
    assert_eq!(list.items[1].name.as_ref(), b"Triager");
    assert_eq!(list.items[1].words.as_ref(), b"NOW LINE");
    assert_eq!(
        list.items[0].event,
        Event::Act { act: d::pages::Act::Go { address: Address::Talk { project: 1, task: 40 } } }
    );
    assert_eq!(crate::escape(&build(&root, &tui)), Some(Event::Dismiss));
}
#[test]
fn the_talks_are_their_goldens() {
    let root = chat::talk_root("open");
    let rows = [
        (
            "talk-80x24-dark",
            Scheme::Dark,
            Size { columns: 80, rows: 24 },
            include_bytes!("golden/talk-80x24-dark.txt").as_slice(),
        ),
        (
            "talk-80x24-light",
            Scheme::Light,
            Size { columns: 80, rows: 24 },
            include_bytes!("golden/talk-80x24-light.txt").as_slice(),
        ),
        (
            "talk-110x32-dark",
            Scheme::Dark,
            Size { columns: 110, rows: 32 },
            include_bytes!("golden/talk-110x32-dark.txt").as_slice(),
        ),
    ];
    let mut views = List::uncapped();
    for (_, _, size, _) in &rows {
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Size { columns: size.columns });
        let mut part = view::chat::read(&root, &tui, &f::words(), WALL);
        part.head.as_mut().unwrap().project = b"vsr-rs".as_slice().into();
        let mut footer = foot::read(&root, &tui, &f::words(), WALL);
        footer.project = Some(b"vsr-rs".as_slice().into());
        let mut view = f::view();
        view::signed(&mut view, &view::Signed { body: view::Body::Chat(part), foot: footer }, &mut Queue::uncapped());
        views.push(view).unwrap();
    }
    let checks: Vec<_> = rows
        .iter()
        .zip(&views)
        .map(|((name, scheme, size, want), view)| goldens::Golden {
            name,
            tree: view.tree(),
            scheme: *scheme,
            background: Some(crate::theme::palette(*scheme)[0].1),
            size: *size,
            focus: Focus::default(),
            want,
        })
        .collect();
    goldens::check(&checks);
}
