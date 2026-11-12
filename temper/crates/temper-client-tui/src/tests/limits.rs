use super::fixtures as f;
use crate::words::Word;
use crate::{Refusal, Told, Tui, View, look};
use skein_ui::{Role, Tree, phrases::Hole};

// temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema, BranchPath: list 24 Stop.
const WIRE_STOPS: u32 = 24;

pub(super) fn fits(tree: &Tree, limits: &skein_ui::Limits) {
    assert_eq!(skein_ui::outline::check(tree), None);
    assert!(tree.len() <= limits.nodes);
    let mut total = 0_u32;
    let mut bindings = 0_u32;
    let mut parents = skein_lib::Stack::with_capacity(limits.depth);
    for (index, node) in tree.nodes().iter().enumerate() {
        let index = u32::try_from(index).unwrap();
        while match parents.top() {
            Some(end) => *end <= index,
            None => false,
        } {
            parents.pop();
        }
        let depth = parents.len().checked_add(1).unwrap();
        assert!(depth <= limits.depth);
        parents.push(index.checked_add(node.size).unwrap()).unwrap();
        assert!(node.text_bytes() <= limits.text_per_node);
        total = total.checked_add(node.text_bytes()).unwrap();
        bindings = bindings.checked_add(u32::from(node.binding.is_some())).unwrap();
        assert!(node.styles.len() <= limits.styles_per_node);
    }
    assert!(total <= limits.text_bytes);
    assert!(bindings <= limits.bindings);
    assert_eq!(f::count(tree, Role::Page), 1);
    assert_eq!(f::count(tree, Role::Main), 1);
    let mut top = false;
    let mut bottom = false;
    let mut feet = 0_u32;
    for node in tree.nodes() {
        top |= node.region == Some(look::TOP);
        bottom |= node.region == Some(look::BOTTOM) || node.region == Some(look::LIST);
        if node.role == Role::ContentInfo || node.region == Some(look::LIST) {
            feet = feet.checked_add(1).unwrap();
        }
    }
    assert!((top || f::count(tree, Role::Log) == 1) && bottom);
    assert_eq!(feet, 1, "a quiet line or list is the foot");
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks all own phrases and both sources of the tree bound")]
#[expect(clippy::disallowed_macros, reason = "boundary fixtures format full notice holes and their expected phrases")]
fn the_trees_fit_their_limits() {
    let domain = f::limits();
    let bounds = crate::limits(&domain, &skein_ui::Limits::EMPTY, 512).expect("fixture limits fit");
    assert!(skein_ui::valid(&bounds));
    let chat = skein_ui::Limits {
        nodes: 1024,
        depth: 64,
        text_per_node: 1_048_576,
        text_bytes: 2_097_152,
        drawing_bytes: 512,
        styles_per_node: 5,
        patches: 4096,
        bindings: 128,
        slots: 32,
        markdown_depth: 5,
        scene_marks: 2,
        scene_bytes: 256,
    };
    let front_text = 50 * 512 + 8192 + 4096;
    let with_front = skein_ui::Limits {
        nodes: chat.nodes + 192,
        text_bytes: chat.text_bytes + front_text,
        bindings: chat.bindings + 1,
        patches: chat.patches + 192 * 4,
        ..chat
    };
    assert_eq!(crate::limits(&domain, &chat, 512), Some(with_front));
    let words = f::words();
    assert!(bounds.text_per_node >= u32::try_from(crate::BoxLine::MOST).unwrap());
    assert!(bounds.nodes >= 610);
    assert!(bounds.bindings >= 15);
    assert_eq!((crate::limits::BOX_LOG, crate::limits::LOG_LINE, crate::limits::STEP), (50, 512, 64));
    let step = "s".repeat(64);
    let title = "t".repeat(usize::try_from(domain.client.domain.text).unwrap());
    let label = "l".repeat(32);
    let step_words = skein_lib::bytes::concat(&[b"18446744073709551615 ", label.as_bytes()]);
    let quit_one = skein_lib::bytes::concat(&[b"Stop the box? ", title.as_bytes(), b"'s running task stops with it."]);
    let restarting = skein_lib::bytes::concat(&[step.as_bytes(), b" restarting"]);
    let gave_up = skein_lib::bytes::concat(&[step.as_bytes(), b" gave up"]);
    let gave_up_note =
        skein_lib::bytes::concat(&[b"The box gave up: ", step.as_bytes(), b". /log shows why; Ctrl+C closes."]);
    let change_start = skein_lib::bytes::concat(&[b"one change, run as ", title.as_bytes(), b"'s way says"]);
    // Seven scheme bytes, 64 authority bytes, seven path bytes and twenty task digits.
    let page_address = skein_lib::bytes::concat(&[b"http://", &[b'a'; 64], b"/tasks/18446744073709551615"]);
    let open_at = skein_lib::bytes::concat(&[b"Open ", &page_address, b" in your browser."]);
    assert_eq!(crate::limits::NOTICE, skein_lib::Duration::from_secs(6));
    // A multibyte scalar crossing the hole's byte bound is left out whole.
    let bound = usize::try_from(domain.client.domain.text).unwrap();
    for (raw, cut) in [
        (
            format!("/{}", "x".repeat(bound.checked_sub(1).unwrap())),
            format!("/{}", "x".repeat(bound.checked_sub(1).unwrap())),
        ),
        (format!("/{}", "x".repeat(bound)), format!("/{}", "x".repeat(bound.checked_sub(1).unwrap()))),
        (
            format!("/{}界", "x".repeat(bound.checked_sub(2).unwrap())),
            format!("/{}", "x".repeat(bound.checked_sub(2).unwrap())),
        ),
    ] {
        let notice = crate::Notice::Unknown { words: raw.as_bytes().into() };
        assert_eq!(words.notice(&notice).as_ref(), format!("Unknown command {cut} · /help lists them").as_bytes());
    }
    let mut failures = skein_lib::List::with_capacity(91);
    for (word, holes, want) in [
        (Word::UnknownNotice, &[Hole::Name(title.as_bytes().into())][..], skein_lib::bytes::concat(&[b"Unknown command ", title.as_bytes(), " · /help lists them".as_bytes()]).as_ref()),
        (Word::SigningInNotice, &[][..], b"Still signing in".as_slice()),
        (Word::KindChat, &[][..], ", a chat".as_bytes()),
        (Word::KindGoal, &[][..], ", a goal".as_bytes()),
        (Word::WalksThem, &[][..], "n walks them".as_bytes()),
        (Word::MoneyNeed, &[][..], "$".as_bytes()),
        (Word::SignOffNeed, &[][..], "✓".as_bytes()),
        (Word::StartHead, &[][..], "Start".as_bytes()),
        (Word::ChangeCommand, &[][..], "One change, run as the project's way says".as_bytes()),
        (Word::BriefCommand, &[][..], "The brief, whole".as_bytes()),
        (Word::NeedsCommand, &[][..], "Walk what needs you".as_bytes()),
        (Word::OpenCommand, &[][..], "Open this on the web".as_bytes()),
        (Word::BriefMore, &[][..], " · /brief".as_bytes()),
        (Word::BoxStart, &[][..], "Ask for a change, or ask temper.  / for commands".as_bytes()),
        (Word::ToWalk, &[][..], "n to walk".as_bytes()),
        (Word::Next, &[][..], "Next".as_bytes()),
        (Word::OpenOnWeb, &[][..], "Open it on the web".as_bytes()),
        (Word::WalkDone, &[][..], "Nothing else needs you.".as_bytes()),
        (Word::KeyWalk, &[][..], "n walks what needs you".as_bytes()),
        (Word::ThemeCommand, &[][..], b"Auto, light or dark".as_slice()),
        (Word::ThemeHead, &[][..], b"Theme. Auto follows your terminal.".as_slice()),
        (Word::ThemeAuto, &[][..], b"Auto".as_slice()),
        (Word::ThemeLight, &[][..], b"Light".as_slice()),
        (Word::ThemeDark, &[][..], b"Dark".as_slice()),
        (Word::ThemeAutoLight, &[][..], b"light, as your terminal says".as_slice()),
        (Word::ThemeAutoDark, &[][..], b"dark, as your terminal says".as_slice()),
        (Word::ThemeAutoNone, &[][..], b"dark: your terminal doesn't say".as_slice()),
        (Word::ThemeNoticeAutoLight, &[][..], b"Theme: auto, light as your terminal says.".as_slice()),
        (Word::ThemeNoticeAutoDark, &[][..], b"Theme: auto, dark as your terminal says.".as_slice()),
        (Word::ThemeNoticeAutoNone, &[][..], b"Theme: auto, dark: your terminal doesn't say.".as_slice()),
        (Word::ThemeNoticeLight, &[][..], b"Theme: light.".as_slice()),
        (Word::ThemeNoticeDark, &[][..], b"Theme: dark.".as_slice()),
        (Word::ChatNumber, &[Hole::Number(u64::MAX)][..], "/chats 18446744073709551615".as_bytes()),
        (Word::ChangeStart, &[Hole::Name(title.as_bytes().into())][..], &change_start),
        (Word::OpenAt, &[Hole::Name(page_address)][..], &open_at),
        (Word::BoxPurpose, &[][..], "on this machine, to try it".as_bytes()),
        (Word::WelcomeOpen, &[][..], "Welcome. Set temper up in your browser: open this link on this machine.".as_bytes()),
        (Word::LinkOnce, &[][..], "It signs you in as this temper's owner, once, within 2 hours.".as_bytes()),
        (Word::InTheBrowser, &[][..], "In the browser".as_bytes()),
        (Word::Step, &[Hole::Number(u64::MAX), Hole::Name(label.as_bytes().into())][..], &step_words),
        (Word::Following, &[][..], "Setting temper up in your browser".as_bytes()),
        (Word::SignedInOwner, &[][..], "Signed in as the owner".as_bytes()),
        (Word::StepsFollowed, &[][..], "The steps are the browser's; this screen follows them.".as_bytes()),
        (Word::Fetching, &[][..], "fetching Forgejo…".as_bytes()),
        (Word::SettingUp, &[][..], "setting up Forgejo…".as_bytes()),
        (Word::ConnectingForgejo, &[][..], "connecting Forgejo…".as_bytes()),
        (Word::ConnectingGitHub, &[][..], "connecting GitHub…".as_bytes()),
        (Word::FirstRepository, &[][..], "making its first repository…".as_bytes()),
        (Word::SetUpStopped, &[][..], "stopped: the browser says why".as_bytes()),
        (Word::Playground, &[][..], "playground".as_bytes()),
        (Word::PlaygroundBanner, &[][..], "This is the playground: a scripted model and in-memory forge. Set temper up in your browser when you're ready to leave it.".as_bytes()),
        (Word::QuitOne, &[Hole::Name(title.as_bytes().into())][..], &quit_one),
        (Word::QuitMany, &[Hole::Number(u64::MAX)][..], "Stop the box? 18446744073709551615 running tasks stop with it.".as_bytes()),
        (Word::QuitNone, &[][..], "Stop the box?".as_bytes()),
        (Word::StopBox, &[][..], "Stop the box".as_bytes()),
        (Word::DontQuit, &[][..], "Don't quit".as_bytes()),
        (Word::Restarting, &[Hole::Name(step.as_bytes().into())][..], &restarting),
        (Word::GaveUp, &[Hole::Name(step.as_bytes().into())][..], &gave_up),
        (Word::GaveUpNote, &[Hole::Name(step.as_bytes().into())][..], &gave_up_note),
        (Word::LogCommand, &[][..], "The box's recent log".as_bytes()),
        (Word::LogHead, &[][..], "/log".as_bytes()),
        (Word::LogEmpty, &[][..], "Nothing yet.".as_bytes()),
        (Word::ExitFront, &[][..], "Close temper and stop the box".as_bytes()),
        (Word::KeyCloseFront, &[][..], "Ctrl+C closes temper and stops the box".as_bytes()),
        (Word::Separator, &[][..], " · ".as_bytes()),
        (Word::Between, &[][..], " / ".as_bytes()),
        (Word::Byline, &[Hole::Name(Box::from(*b"00:00"))][..], "written by the status agent 00:00".as_bytes()),
        (Word::LeftOff, &[][..], "Where you left off".as_bytes()),
        (Word::NeedsYou, &[][..], "Needs you".as_bytes()),
        (Word::Question, &[][..], "?".as_bytes()),
        (Word::OtherNeed, &[][..], "•".as_bytes()),
        (
            Word::NoProject,
            &[Hole::Name(Box::from(f::ADDRESS))][..],
            "Setting temper up makes your first project: open http://127.0.0.1:8367 in a browser.".as_bytes(),
        ),
        (Word::Needing, &[Hole::Number(0)][..], "0 need you".as_bytes()),
        (Word::Needing, &[Hole::Number(1)][..], "1 needs you".as_bytes()),
        (Word::Needing, &[Hole::Number(2)][..], "2 need you".as_bytes()),
        (Word::Needing, &[Hole::Number(u64::MAX)][..], "18446744073709551615 need you".as_bytes()),
        (Word::NeedingCut, &[Hole::Number(3)][..], "3+ need you".as_bytes()),
        (Word::AskTemper, &[][..], "Ask temper".as_bytes()),
        (Word::Message, &[][..], "Message".as_bytes()),
        (Word::Reply, &[][..], "Reply".as_bytes()),
        (Word::Ended, &[][..], "/new starts another chat".as_bytes()),
        (Word::Commands, &[][..], "Commands".as_bytes()),
        (Word::Exit, &[][..], "Close temper".as_bytes()),
        (Word::KeyEnter, &[][..], "Enter sends".as_bytes()),
        (Word::KeyTab, &[][..], "Tab completes a command".as_bytes()),
        (Word::KeyPage, &[][..], "PgUp and PgDn read back".as_bytes()),
        (Word::KeyEsc, &[][..], "Esc closes a list".as_bytes()),
        (Word::KeyClose, &[][..], "Ctrl+C closes temper".as_bytes()),
        (Word::Else, &[][..], "Tell it something else".as_bytes()),
        (Word::Dots, &[][..], "…".as_bytes()),
        (Word::Numbered, &[Hole::Number(3)][..], "3. ".as_bytes()),
    ] {
        let said = words.say(word, holes);
        if said.as_ref() != want {
            failures.push(word).unwrap();
        }
        assert!(u32::try_from(said.len()).unwrap() <= word.capacity(&domain).unwrap(), "{word:?}");
    }
    assert!(failures.is_empty(), "word rows: {failures:?}");
    let name = [b'n'; 32];
    let address = *b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:65535";
    let me = f::me(&[&name]);
    let root = f::live(Some(me)).0;
    let mut tui = Tui::new(&address);
    tui.step(Told::Kept { written: false });
    let mut view = View::new(&bounds, f::words());
    f::build(&mut view, &tui, &root);
    fits(view.tree(), &bounds);
    f::find(view.tree(), Role::Text, b"nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn");
    let full = f::refused(f::full());
    let approval = super::approval::asked(Some(temper_client_domain::asked::State::Asked {
        code: super::approval::code(),
        expires: skein_lib::Wall::from_nanos(600_000_000_000),
    }));
    for (refusal, browser) in [
        (Some(Refusal::OtherAddress { address: Box::from(address) }), false),
        (None, false),
        (Some(Refusal::OtherAddress { address: Box::from(address) }), true),
    ] {
        let mut tui = Tui::new(&address);
        tui.step(Told::Link { link: jig_client::Link::Live });
        tui.step(Told::Link { link: jig_client::Link::SignedOut });
        if browser {
            tui.step(Told::Browse);
            tui.step(Told::Opened { opened: false });
        } else {
            tui.step(Told::Asked);
        }
        tui.step(Told::Typed { value: Box::new([b'x'; 512]) });
        match refusal {
            Some(refusal) => tui.step(Told::Refused { refusal }),
            None => tui.step(Told::Answered { lost: false }),
        }
        f::build(&mut view, &tui, if browser { &approval } else { &full });
        fits(view.tree(), &bounds);
        let field = f::find(view.tree(), Role::TextField, b"Link");
        assert_eq!(field.value.as_ref().unwrap().text.len(), 512);
        assert_eq!(view.tree().len(), if browser { 28 } else { 18 });
    }
    // Three left-off rows, five needs, Start and four whole briefs coexist at their bounds.
    for bytes in [200_usize, 600] {
        use crate::view::{self, launchpad};
        let root = super::launchpad::home(None, &[]);
        let mut part = launchpad::read(&root, &tui, &words, f::NOW);
        part.project = Some(name.into());
        part.lines
            .push(launchpad::Line {
                role: Role::Paragraph,
                words: b"Four changes landed across four projects.".as_slice().into(),
                style: None,
            })
            .unwrap();
        part.lines
            .push(launchpad::Line {
                role: Role::Paragraph,
                words: "s".repeat((32 + 2 + bytes) * 4 + 3).as_bytes().into(),
                style: None,
            })
            .unwrap();
        part.lines
            .push(launchpad::Line {
                role: Role::Paragraph,
                words: b"written by the status agent 07:42 \xc2\xb7 /brief".as_slice().into(),
                style: Some(look::BYLINE),
            })
            .unwrap();
        for task in 0..3 {
            part.left_off
                .push(launchpad::LeftOff {
                    task,
                    project: name.into(),
                    title: title.as_bytes().into(),
                    kind: launchpad::LeftKind::Chat { number: u32::try_from(task + 1).unwrap(), session: false },
                    at: b"Fri 18:40".as_slice().into(),
                    state: b"waiting on you".as_slice().into(),
                })
                .unwrap();
        }
        let mut needs = skein_lib::List::with_capacity(5);
        for _row in 0_u32..5 {
            needs
                .push(launchpad::NeedRow {
                    since: None,
                    mark: temper_client_view::shared::NeedMark::SignOff,
                    project: name.into(),
                    words: Box::new([b'w'; 512]),
                })
                .unwrap();
        }
        part.needs = needs.into_boxed();
        for _project in 0_u32..4 {
            for length in [32 + 2 + 200, bytes] {
                part.brief
                    .push(launchpad::Line {
                        role: Role::Paragraph,
                        words: "b".repeat(length).as_bytes().into(),
                        style: Some(look::NOTE),
                    })
                    .unwrap();
            }
        }
        part.brief
            .push(launchpad::Line {
                role: Role::Paragraph,
                words: b"written by the status agent 07:42".as_slice().into(),
                style: Some(look::NOTE),
            })
            .unwrap();
        let foot = view::foot::read(&root, &tui, &words, f::NOW);
        view::signed(
            &mut view,
            &view::Signed { body: view::Body::Launchpad(part), foot },
            &mut skein_lib::Queue::uncapped(),
        );
        fits(view.tree(), &bounds);
    }
    // Each retained view can replace an address with two full printed paths.
    let page = temper_client_view::Limits::of(&domain, words.view()).unwrap();
    let chat_bounds = crate::limits(&domain, &page.tree, 512).unwrap();
    let part = chat_views();
    let mut chat_view = View::new(&chat_bounds, words);
    let foot = crate::view::foot::read(&root, &tui, &f::words(), f::NOW);
    crate::view::signed(
        &mut chat_view,
        &crate::view::Signed { body: crate::view::Body::Chat(part), foot },
        &mut skein_lib::Queue::uncapped(),
    );
    fits(chat_view.tree(), &chat_bounds);
    let stops = temper_client_domain::Limits::chat_views() * 2 * WIRE_STOPS;
    assert_eq!(f::count(chat_view.tree(), Role::Description), usize::try_from(stops * 2).unwrap());
    assert_eq!(crate::limits(&domain, &skein_ui::Limits::EMPTY, u32::MAX), None);
    let mut huge = domain;
    huge.client.domain.text = u32::MAX;
    assert_eq!(crate::limits(&huge, &skein_ui::Limits::EMPTY, 512), None);
}

#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "the boundary fixture builds maximum wire paths and collects the admitted views"
)]
fn chat_views() -> crate::view::chat::Chat {
    use crate::view::{
        chat::{Chat, Entry, Piece},
        foot,
        printed::Printed,
    };
    use temper_client_protocol_wire::v1 as w;
    let words = vec![b'w'; usize::try_from(temper_client_view::paths::WORDS).unwrap()];
    let stops: Vec<_> = (0..WIRE_STOPS).map(|_| (w::Actor::Code, w::PathStep::Check, words.as_slice())).collect();
    let path = super::printed::path(&stops);
    let pieces = (0..temper_client_domain::Limits::chat_views())
        .map(|_| {
            Piece::View(Printed::Drawn(Box::new(w::View::Proposal(
                w::ProposalView::new(&w::CEILINGS, w::ProposalViewParts { before: path.clone(), after: path.clone() })
                    .unwrap(),
            ))))
        })
        .collect();
    Chat {
        pane: None,
        to_end: 1,
        entries: Box::new([Entry::Viewed(pieces)]),
        notes: foot::Notes { lines: Box::default(), gave_up: None, log: None },
    }
}
