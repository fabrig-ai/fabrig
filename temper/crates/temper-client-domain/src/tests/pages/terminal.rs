use super::*;
use crate::pages::{
    chat::document::Mode,
    terminal::{self, Command, Font, Listing, Look, Note, Theme},
};

#[test]
fn the_look_moves_by_its_acts() {
    let limits = fixtures::limits();
    let root = crate::unlinked(&limits, fixtures::SEED);
    let dark = Look::default();
    let light = Look { terminal: true, theme: Theme::Light, font: Font::Plex };
    for (before, act, after) in [
        (dark, terminal::Act::Enter, Look { terminal: true, ..dark }),
        (dark, terminal::Act::Leave, dark),
        (dark, terminal::Act::Theme, Look { theme: Theme::Light, ..dark }),
        (dark, terminal::Act::Font, Look { font: Font::Plex, ..dark }),
        (light, terminal::Act::Enter, light),
        (light, terminal::Act::Leave, Look { terminal: false, ..light }),
        (light, terminal::Act::Theme, Look { theme: Theme::Dark, ..light }),
        (light, terminal::Act::Font, Look { font: Font::DejaVu, ..light }),
    ] {
        let mut pages = Pages::new();
        assert_eq!(pages.look(), dark);
        let env = fixtures::env(&limits);
        let mut out = Queue::uncapped();
        pages.restore_look(before);
        pages::step(&mut pages, &env, root.client(), Event::Went { address: Address::Chat { task: 9 } }, &mut out);
        pages::step(&mut pages, &env, root.client(), Event::Act { act: Act::Mode { mode: Mode::Split } }, &mut out);
        pages::step(
            &mut pages,
            &env,
            root.client(),
            Event::Act { act: Act::MessageWords { words: Box::from(b"Keep these words".as_slice()) } },
            &mut out,
        );
        let mut domain = crate::unlinked(&limits, fixtures::SEED);
        domain.pages = pages;
        let mut acts = Queue::uncapped();
        crate::step(
            &mut domain,
            &env,
            crate::Event::Act { action: Action::Pages(Act::Terminal(act)) },
            &mut acts,
        );
        assert_eq!(acts.pop(), Some(crate::Request::KeepLook { look: after }));
        assert!(acts.is_empty());
        let pages = domain.pages();
        assert_eq!(pages.look(), after);
        let font_note = if before.font == after.font { None } else { Some(Note::Font { font: after.font }) };
        assert_eq!(pages.notes().last(), font_note.as_ref());
        assert_eq!(pages.address(), &Address::Chat { task: 9 });
        let chat = pages.chat().unwrap();
        assert_eq!(chat.mode, Mode::Split);
        assert_eq!(chat.words.as_ref(), b"Keep these words");
    }
    for look in [dark, light] {
        let mut root = crate::unlinked(&limits, fixtures::SEED);
        let mut out = Queue::uncapped();
        crate::step(&mut root, &fixtures::env(&limits), crate::Event::LookKept { look }, &mut out);
        assert_eq!(root.pages().look(), look);
        assert!(out.is_empty(), "a restored look asks for nothing");
    }
    for (address, expected) in [
        (Address::Chat { task: 9 }, true),
        (Address::NewChat { draft: false }, true),
        (Address::Configure { project: 1 }, true),
        (Address::Chats { filter: ChatsFilter::All }, false),
        (Address::Home { project: 1 }, false),
    ] {
        assert_eq!(terminal::draws(&address), expected, "{address:?}");
    }
    for (mode, draft) in [(Mode::Chat, false), (Mode::Split, true)] {
        let mut pages = Pages::new();
        let env = fixtures::env(&limits);
        let mut out = Queue::uncapped();
        pages::step(
            &mut pages,
            &env,
            root.client(),
            Event::Went { address: Address::NewChat { draft: !draft } },
            &mut out,
        );
        pages::step(
            &mut pages,
            &env,
            root.client(),
            Event::Act { act: Act::ChatWords { words: Box::from(b"Keep the opening".as_slice()) } },
            &mut out,
        );
        pages::step(&mut pages, &env, root.client(), Event::Act { act: Act::Terminal(terminal::Act::Enter) }, &mut out);
        pages::step(&mut pages, &env, root.client(), Event::Act { act: Act::Mode { mode } }, &mut out);
        assert_eq!(pages.address(), &Address::NewChat { draft });
        assert_eq!(pages.chats().unwrap().words.as_ref(), b"Keep the opening");
        assert_eq!(pages.look(), Look { terminal: true, ..dark });
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "the command table and its send and dismiss cases share one fixture")]
fn a_command_runs_the_pages_own_act() {
    let limits = fixtures::limits();
    let root = crate::unlinked(&limits, fixtures::SEED);
    let env = fixtures::env(&limits);
    for command in Command::ALL {
        let mut pages = Pages::new();
        let mut out = Queue::uncapped();
        for event in [
            Event::Went { address: Address::Chat { task: 9 } },
            Event::Act { act: Act::Terminal(terminal::Act::Enter) },
            Event::Act { act: Act::ShowSection { line: 1 } },
            Event::Act { act: Act::MessageWords { words: b"/words".as_slice().into() } },
            Event::Act { act: Act::Terminal(terminal::Act::Run { command }) },
        ] {
            pages::step(&mut pages, &env, root.client(), event, &mut out);
        }
        assert!(pages.chat().unwrap().words.is_empty(), "{command:?}");
        assert_eq!(pages.chat().unwrap().shown, None, "the command box owns focus");
        let listing = match command {
            Command::Model => Some(Listing::Model),
            Command::Effort => Some(Listing::Effort),
            Command::Chats => Some(Listing::Chats),
            Command::Draft
            | Command::Pause
            | Command::Close
            | Command::StartOver
            | Command::Goal
            | Command::New
            | Command::Theme
            | Command::Font
            | Command::Help
            | Command::Exit => None,
        };
        assert_eq!(pages.listing(), listing, "{command:?}");
        match command {
            Command::Theme => assert_eq!(pages.look().theme, Theme::Light),
            Command::Font => assert_eq!(pages.notes(), &[Note::Font { font: Font::Plex }]),
            Command::Help => assert_eq!(pages.notes(), &[Note::Help]),
            Command::Exit => assert!(!pages.look().terminal),
            Command::New => assert!({
                let mut found = false;
                while let Some(r) = out.pop() {
                    found |= r == (Request::Show { address: Address::NewChat { draft: false } });
                }
                found
            }),
            Command::Model
            | Command::Effort
            | Command::Draft
            | Command::Pause
            | Command::Close
            | Command::StartOver
            | Command::Goal
            | Command::Chats => {}
        }
        if listing.is_some() {
            pages::step(
                &mut pages,
                &env,
                root.client(),
                Event::Act { act: Act::MessageWords { words: b"hello".as_slice().into() } },
                &mut out,
            );
            assert_eq!(pages.listing(), None, "typed words close {command:?}");
        }
    }
    for (words, note, listing) in [
        (b"/model".as_slice(), None, Some(Listing::Model)),
        (b"/nope", Some(Note::Unknown { words: b"/nope".as_slice().into() }), None),
        (b"/model now", None, None),
    ] {
        let mut pages = Pages::new();
        let mut out = Queue::uncapped();
        for event in [
            Event::Went { address: Address::Chat { task: 9 } },
            Event::Act { act: Act::Terminal(terminal::Act::Enter) },
            Event::Act { act: Act::MessageWords { words: words.into() } },
            Event::Act { act: Act::Send },
        ] {
            pages::step(&mut pages, &env, root.client(), event, &mut out);
        }
        assert_eq!(pages.notes().last(), note.as_ref());
        assert_eq!(pages.listing(), listing);
        assert_eq!(pages.chat().unwrap().words.is_empty(), words != b"/model now");
    }
    for (words, expected) in [(b"/mo".as_slice(), b"".as_slice()), (b"hello", b"hello")] {
        let mut pages = Pages::new();
        let mut out = Queue::uncapped();
        for event in [
            Event::Went { address: Address::Chat { task: 9 } },
            Event::Act { act: Act::Terminal(terminal::Act::Run { command: Command::Model }) },
            Event::Act { act: Act::MessageWords { words: words.into() } },
        ] {
            pages::step(&mut pages, &env, root.client(), event, &mut out);
        }
        pages::step(
            &mut pages,
            &env,
            root.client(),
            Event::Act { act: Act::Terminal(terminal::Act::Run { command: Command::Model }) },
            &mut out,
        );
        assert_eq!(pages.listing(), Some(Listing::Model));
        pages::step(
            &mut pages,
            &env,
            root.client(),
            Event::Act { act: Act::Terminal(terminal::Act::Dismiss) },
            &mut out,
        );
        assert_eq!(pages.listing(), None);
        pages::step(
            &mut pages,
            &env,
            root.client(),
            Event::Act { act: Act::MessageWords { words: words.into() } },
            &mut out,
        );
        pages::step(
            &mut pages,
            &env,
            root.client(),
            Event::Act { act: Act::Terminal(terminal::Act::Dismiss) },
            &mut out,
        );
        assert_eq!(pages.listing(), None);
        assert_eq!(pages.chat().unwrap().words.as_ref(), expected);
    }
    for (command, own, configuring) in [
        (Command::Draft, Act::Mode { mode: Mode::Split }, false),
        (Command::Pause, Act::Stop, false),
        (Command::Close, Act::Close, false),
        (Command::StartOver, Act::StartOver, true),
        (
            Command::Goal,
            Act::OpenStart {
                draft: Some(b"docs/design/draft/draft.md".as_slice().into()),
                chat: Some(fixtures::chat::TASK),
            },
            false,
        ),
    ] {
        let (mut commanded, f) = loaded_pages(false, configuring);
        let (mut direct, _) = loaded_pages(false, configuring);
        let mut actual = Queue::uncapped();
        let mut expected = Queue::uncapped();
        for (p, out) in [(&mut commanded, &mut actual), (&mut direct, &mut expected)] {
            pages::step(
                p,
                &env,
                &f.client,
                Event::Act { act: Act::MessageWords { words: b"/words".as_slice().into() } },
                out,
            );
        }
        pages::step(
            &mut commanded,
            &env,
            &f.client,
            Event::Act { act: Act::Terminal(terminal::Act::Run { command }) },
            &mut actual,
        );
        pages::step(&mut direct, &env, &f.client, Event::Act { act: own }, &mut expected);
        assert!(commanded.chat().unwrap().words.is_empty());
        assert_eq!(commanded.chat().unwrap().mode, direct.chat().unwrap().mode, "{command:?}");
        assert_eq!(commanded.confirmation(), direct.confirmation(), "{command:?}");
        assert_eq!(commanded.start().is_some(), direct.start().is_some(), "{command:?}");
        while let Some(r) = actual.pop() {
            assert_eq!(Some(r), expected.pop(), "{command:?}");
        }
        assert!(expected.is_empty(), "{command:?}");
        match command {
            Command::Close | Command::StartOver => assert!(commanded.confirmation().is_some()),
            Command::Goal => assert!(commanded.start().is_some()),
            Command::Pause => assert!(!commanded.chat().unwrap().pending.is_empty()),
            Command::Model
            | Command::Effort
            | Command::Draft
            | Command::New
            | Command::Chats
            | Command::Theme
            | Command::Font
            | Command::Help
            | Command::Exit => {}
        }
    }
    for (ended, words, sent, note) in [
        (false, b"/model now".as_slice(), true, None),
        (false, "/model\u{2003}now".as_bytes(), true, None),
        (false, b"/model", false, None),
        (false, b"/nope", false, Some(Note::Unknown { words: b"/nope".as_slice().into() })),
        (true, b"hello", false, Some(Note::Closed)),
        (true, b"/help", false, Some(Note::Help)),
    ] {
        let (mut pages, f) = loaded_pages(ended, false);
        let mut out = Queue::uncapped();
        for act in [Act::MessageWords { words: words.into() }, Act::Send] {
            pages::step(&mut pages, &env, &f.client, Event::Act { act }, &mut out);
        }
        assert_eq!(!pages.chat().unwrap().pending.is_empty(), sent, "{words:?}");
        assert_eq!(pages.notes().last(), note.as_ref());
    }
    for following in [
        Act::Terminal(terminal::Act::Dismiss),
        Act::MessageWords { words: b"hello".as_slice().into() },
        Act::Mode { mode: Mode::Chat },
    ] {
        let (mut cancelling, f) = loaded_pages(false, false);
        let mut out = Queue::uncapped();
        pages::step(
            &mut cancelling,
            &env,
            &f.client,
            Event::Act { act: Act::Terminal(terminal::Act::Run { command: Command::Close }) },
            &mut out,
        );
        assert!(cancelling.confirmation().is_some());
        pages::step(&mut cancelling, &env, &f.client, Event::Act { act: following }, &mut out);
        assert!(cancelling.confirmation().is_none());
        assert!(cancelling.chat().unwrap().closing.is_none());
    }
    let mut fresh = Pages::new();
    pages::step(
        &mut fresh,
        &env,
        root.client(),
        Event::Went { address: Address::NewChat { draft: false } },
        &mut Queue::uncapped(),
    );
    let mut opening = Queue::uncapped();
    pages::step(
        &mut fresh,
        &env,
        root.client(),
        Event::Act { act: Act::Terminal(terminal::Act::Run { command: Command::Draft }) },
        &mut opening,
    );
    assert_eq!(opening.pop(), Some(Request::Replace { address: Address::NewChat { draft: true } }));
    assert_eq!(fresh.address(), &Address::NewChat { draft: false }, "navigation follows the same Go as + Draft");
    for address in [Address::NewChat { draft: false }, Address::Configure { project: fixtures::PROJECT }] {
        let mut pages = Pages::new();
        let mut out = Queue::uncapped();
        for event in [
            Event::Went { address },
            Event::Act { act: Act::Terminal(terminal::Act::Enter) },
            Event::Act { act: Act::ChatWords { words: b"/model".as_slice().into() } },
            Event::Act { act: Act::StartChat },
        ] {
            pages::step(&mut pages, &env, root.client(), event, &mut out);
        }
        assert_eq!(pages.listing(), Some(Listing::Model));
        assert!(pages.chats().unwrap().words.is_empty());
    }
    let mut pages = Pages::new();
    let mut out = Queue::uncapped();
    for _ in 0u8..10 {
        pages::step(
            &mut pages,
            &env,
            root.client(),
            Event::Act { act: Act::Terminal(terminal::Act::Run { command: Command::Help }) },
            &mut out,
        );
    }
    assert_eq!(pages.notes().len(), 8);
    pages::step(&mut pages, &env, root.client(), Event::Went { address: Address::NewChat { draft: false } }, &mut out);
    assert!(pages.notes().is_empty());
    pages::step(
        &mut pages,
        &env,
        root.client(),
        Event::Act { act: Act::Terminal(terminal::Act::Run { command: Command::Help }) },
        &mut out,
    );
    pages::step(&mut pages, &env, root.client(), Event::Link { link: Link::SignedOut }, &mut out);
    assert!(pages.notes().is_empty());
}

fn loaded_pages(ended: bool, configuring: bool) -> (Pages, fixtures::chat::Fixture) {
    use jig_client::contract as c;
    let mut fixture = fixtures::chat::opened();
    let mut task = fixtures::chat::task();
    task.chip.executor = c::ChipExecutor::Agent { charter: 1 };
    task.acts = skein_lib::List::with_capacity(4);
    if ended {
        fixtures::chat::phase(&mut task, c::ChipPhase::Ended);
    } else {
        for act in [c::ActKind::Message, c::ActKind::Stop, c::ActKind::Cancel, c::ActKind::Amend] {
            task.acts.push(c::Offered { act, admission: c::Admission::Allowed, effect: None }).unwrap();
        }
    }
    fixtures::chat::deliver(&mut fixture.client, fixture.watch, task);
    let mut me = fixtures::chats::me();
    if configuring {
        me.projects.get_mut(0).unwrap().templates.get_mut(0).unwrap().number = 3;
    }
    for event in [
        jig_client::Event::Watch { handle: pages::me_handle(), subject: jig_client::Watchable::Jig(c::Subject::Me) },
        jig_client::Event::Up {
            up: jig_client::Up::Watched {
                watch: fixture.me,
                event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
            },
        },
    ] {
        fixture.client.step_unlinked(skein_lib::Time::ZERO, skein_lib::Wall::EPOCH, event, &mut Queue::uncapped());
    }
    let mut pages = Pages::new();
    let env = fixtures::env(&fixtures::limits());
    let mut out = Queue::uncapped();
    for event in [
        Event::Went { address: Address::Chat { task: fixtures::chat::TASK } },
        Event::Told { told: Told::Changed { handle: fixture.chat.document } },
        Event::Act { act: Act::Terminal(terminal::Act::Enter) },
    ] {
        pages::step(&mut pages, &env, &fixture.client, event, &mut out);
    }
    let mut notes = None;
    while let Some(r) = out.pop() {
        if let Request::Client(jig_client::Event::Read {
            handle,
            read: jig_client::Readable::Jig(c::Read::Notes { .. }),
        }) = r
        {
            notes = Some(handle);
        }
    }
    let row = c::Note {
        number: 1,
        revision: c::Revision { value: 1 },
        scope: c::NoteScope::Task { project: fixtures::PROJECT, task: fixtures::chat::TASK },
        description: c::DocumentText::complete(c::Text::new(b"Draft".as_slice().into(), 32).unwrap()),
        body: c::DocumentText::complete(c::Text::new(b"# Draft\nBody".as_slice().into(), 32).unwrap()),
        writer: c::NoteAuthor::Task { task: fixtures::chat::TASK, attempt: 1 },
        tasks: skein_lib::List::with_capacity(0),
        changed: None,
        acts: skein_lib::List::with_capacity(0),
        cut: false,
    };
    pages::step(
        &mut pages,
        &env,
        &fixture.client,
        Event::Told {
            told: Told::Page {
                handle: notes.unwrap(),
                page: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(c::NotePage {
                    cut: false,
                    rows: skein_lib::List::from_boxed(Box::new([row])),
                    before: None,
                    after: None,
                })))),
            },
        },
        &mut out,
    );
    (pages, fixture)
}
