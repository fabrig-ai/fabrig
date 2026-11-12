use super::*;
use crate::pages::{
    chat::document::Mode,
    terminal::{self, Command, Font, Listing, Look, Note, Theme},
};

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "look acts, restoration and full-screen departure share the terminal fixture"
)]
fn the_look_moves_by_its_acts() {
    let limits = fixtures::limits();
    let root = crate::unlinked(&limits, fixtures::SEED);
    let dark = Look::default();
    let light = Look { terminal: true, theme: Theme::Light, font: Font::Plex, night: true };
    assert!(!dark.night, "a first visit is day");
    let mut stale_acts = Queue::uncapped();
    let mut stale_restores = Queue::uncapped();
    let mut stale_departures = Queue::uncapped();
    let mut stale_list_restores = Queue::uncapped();
    let mut stale_list_departures = Queue::uncapped();
    for (before, act, after, screen) in [
        (dark, terminal::Act::Enter, Look { terminal: true, ..dark }, Some(true)),
        (dark, terminal::Act::Leave, dark, Some(false)),
        (dark, terminal::Act::Theme, Look { theme: Theme::Light, ..dark }, None),
        (dark, terminal::Act::Font, Look { font: Font::Plex, ..dark }, None),
        (light, terminal::Act::Enter, light, Some(true)),
        (light, terminal::Act::Leave, Look { terminal: false, ..light }, Some(false)),
        (light, terminal::Act::Theme, Look { theme: Theme::Dark, ..light }, None),
        (light, terminal::Act::Font, Look { font: Font::DejaVu, ..light }, None),
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
        let shown = crate::shown(&domain);
        crate::step(
            &mut domain,
            &env,
            crate::Event::Act { action: Action::Pages(Act::Terminal(act.clone())) },
            &mut acts,
        );
        if crate::shown(&domain) <= shown {
            stale_acts.push((before, act));
        }
        assert_eq!(acts.pop(), Some(crate::Request::KeepLook { look: after }));
        let screen = match screen {
            Some(on) => Some(crate::Request::Screen { on }),
            None => None,
        };
        assert_eq!(acts.pop(), screen);
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
    let mut domain = crate::unlinked(&limits, fixtures::SEED);
    let env = fixtures::env(&limits);
    let home = Address::Home { project: 1 };
    let mut out = Queue::uncapped();
    crate::step(&mut domain, &env, crate::Event::Went { place: crate::Place::Address(home.clone()) }, &mut out);
    out.clear();
    for night in [true, false] {
        let shown = crate::shown(&domain);
        crate::step(
            &mut domain,
            &env,
            crate::Event::Act { action: Action::Pages(Act::Terminal(terminal::Act::Night)) },
            &mut out,
        );
        assert_eq!(domain.pages().address(), &home);
        assert_eq!(domain.pages().look(), Look { night, ..dark });
        assert_eq!(out.pop(), Some(crate::Request::KeepLook { look: Look { night, ..dark } }));
        assert!(out.is_empty(), "night asks no full screen");
        assert!(crate::shown(&domain) > shown, "night redraws Home");
    }
    for look in [dark, light] {
        let mut root = crate::unlinked(&limits, fixtures::SEED);
        let mut out = Queue::uncapped();
        let shown = crate::shown(&root);
        crate::step(&mut root, &fixtures::env(&limits), crate::Event::LookKept { look }, &mut out);
        if crate::shown(&root) <= shown {
            stale_restores.push(look);
        }
        assert_eq!(root.pages().look(), look);
        assert!(out.is_empty(), "a restored look asks for nothing");
    }
    for (command, words, asking) in [
        (None, b"hello".as_slice(), false),
        (None, b"/m".as_slice(), false),
        (Some(Command::Model), b"".as_slice(), false),
        (Some(Command::Effort), b"".as_slice(), false),
        (Some(Command::Close), b"".as_slice(), false),
        (Some(Command::StartOver), b"".as_slice(), true),
    ] {
        let (mut pages, fixture) = loaded_pages(false, asking);
        let env = fixtures::env(&limits);
        let mut out = Queue::uncapped();
        pages::step(
            &mut pages,
            &env,
            &fixture.client,
            Event::Act { act: Act::MessageWords { words: words.into() } },
            &mut out,
        );
        if let Some(command) = command {
            pages::step(
                &mut pages,
                &env,
                &fixture.client,
                Event::Act { act: Act::Terminal(terminal::Act::Run { command }) },
                &mut out,
            );
        }
        let mut domain = crate::unlinked(&limits, fixtures::SEED);
        domain.pages = pages;
        domain.client = fixture.client;
        let mut out = Queue::uncapped();
        let shown = crate::shown(&domain);
        crate::step(&mut domain, &env, crate::Event::ScreenLeft, &mut out);
        if crate::shown(&domain) <= shown {
            stale_departures.push((command, words, asking));
        }
        assert_eq!(domain.pages().look(), Look { terminal: true, ..dark });
        assert_eq!(domain.pages().listing(), None);
        assert!(domain.pages().confirmation().is_none());
        let chat = domain.pages().chat().unwrap();
        assert!(chat.closing.is_none());
        assert_eq!(chat.words.as_ref(), if words.starts_with(b"/") { b"" } else { words });
        assert!(out.is_empty());
    }
    for command in [None, Some(Command::Model)] {
        let mut domain = crate::unlinked(&limits, fixtures::SEED);
        let env = fixtures::env(&limits);
        let mut out = Queue::uncapped();
        crate::step(
            &mut domain,
            &env,
            crate::Event::Act { action: Action::Pages(Act::Terminal(terminal::Act::Enter)) },
            &mut out,
        );
        if let Some(command) = command {
            crate::step(
                &mut domain,
                &env,
                crate::Event::Act { action: Action::Pages(Act::Terminal(terminal::Act::Run { command })) },
                &mut out,
            );
        }
        let listing = domain.pages().listing();
        let mut out = Queue::uncapped();
        let shown = crate::shown(&domain);
        crate::step(&mut domain, &env, crate::Event::LookKept { look: light }, &mut out);
        if crate::shown(&domain) <= shown {
            stale_list_restores.push(command);
        }
        assert_eq!(domain.pages().look(), light);
        assert_eq!(domain.pages().listing(), listing, "restoring a look leaves its list alone");
        assert!(out.is_empty());
        let shown = crate::shown(&domain);
        crate::step(&mut domain, &env, crate::Event::ScreenLeft, &mut out);
        if crate::shown(&domain) <= shown {
            stale_list_departures.push(command);
        }
        assert_eq!(domain.pages().look(), light);
        assert_eq!(domain.pages().listing(), None);
        assert!(out.is_empty());
    }
    for (address, expected) in [
        (Address::Chat { task: 9 }, true),
        (Address::NewChat { draft: false }, true),
        (Address::Ask { project: 1 }, true),
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
    assert!(
        stale_acts.is_empty()
            && stale_restores.is_empty()
            && stale_departures.is_empty()
            && stale_list_restores.is_empty()
            && stale_list_departures.is_empty(),
        "rows that did not move shown: acts {stale_acts:#?}; LookKept {stale_restores:#?}; \
         ScreenLeft {stale_departures:#?}; LookKept after commands {stale_list_restores:#?}; \
         ScreenLeft after restored look and commands {stale_list_departures:#?}"
    );
}

#[test]
#[expect(clippy::too_many_lines, reason = "the command table and its send and dismiss cases share one fixture")]
fn a_command_runs_the_pages_own_act() {
    let limits = fixtures::limits();
    let root = crate::unlinked(&limits, fixtures::SEED);
    let env = fixtures::env(&limits);
    for (terminal, words, note, sent) in [
        (true, b"/nope".as_slice(), Some(Note::Unknown { words: b"/nope".as_slice().into() }), false),
        (true, b"/help", Some(Note::Help), false),
        (true, b"/help now", None, true),
        (false, b"/help", None, true),
    ] {
        let mut page = chats::Page::new(chats::asking_me(), Address::Home { project: fixtures::PROJECT });
        page.pages.restore_look(Look { terminal, ..Look::default() });
        page.words(words);
        page.act(Act::StartChat);
        assert_eq!(page.pages.notes().last(), note.as_ref(), "{terminal}: {words:?}");
        let asks = fixtures::acts::sent(&mut page.out);
        let expected = usize::from(sent);
        assert_eq!(asks.len(), expected, "{terminal}: {words:?}");
        for ask in asks.as_ref() {
            let jig_client::contract::Ask::Create { tasks, .. } = ask else { panic!("create: {ask:?}") };
            let [task] = tasks.as_slice() else { panic!("one task: {tasks:?}") };
            assert_eq!(task.spec.words.bytes(), words);
        }
        assert!(!terminal::draws(page.pages.address()), "Home keeps the browser's frame");
    }
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
    for (command, own, asking) in [
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
        let (mut commanded, f) = loaded_pages(false, asking);
        let (mut direct, _) = loaded_pages(false, asking);
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
    for address in [Address::NewChat { draft: false }, Address::Ask { project: fixtures::PROJECT }] {
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
    assert_eq!((pages.notes()).len(), 0);
    pages::step(
        &mut pages,
        &env,
        root.client(),
        Event::Act { act: Act::Terminal(terminal::Act::Run { command: Command::Help }) },
        &mut out,
    );
    pages::step(&mut pages, &env, root.client(), Event::Link { link: Link::SignedOut }, &mut out);
    assert_eq!((pages.notes()).len(), 0);
}

fn loaded_pages(ended: bool, asking: bool) -> (Pages, fixtures::chat::Fixture) {
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
    if asking {
        me.projects.get_mut(0).unwrap().templates.get_mut(0).unwrap().number = 5;
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
