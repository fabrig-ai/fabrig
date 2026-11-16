//! Places before their first words and empty sends (client/tui/pages/README.md, 2).
use super::*;

#[derive(Clone, Copy, Debug)]
enum Screen {
    Home,
    Chat,
    Talk,
}

fn at(screen: Screen) -> Host {
    match screen {
        Screen::Home => one_chat_host(false),
        Screen::Chat => {
            let mut host = home_host(false);
            person_chat(&mut host);
            host
        }
        Screen::Talk => talk::talk_host(true, false),
    }
}

#[test]
fn an_own_name_opens_its_empty_place() {
    for (screen, command, prefix) in [
        (Screen::Home, Own::Draft, None),
        (Screen::Home, Own::Change, None),
        (Screen::Home, Own::Ask, None),
        (Screen::Home, Own::Tell, None),
        (Screen::Chat, Own::Tell, None),
        (Screen::Talk, Own::Tell, None),
        (Screen::Home, Own::Change, Some("/ch")),
    ] {
        for sent in [false, true] {
            let mut host = at(screen);
            let name = core::str::from_utf8(command.name()).expect("command name");
            key_words(&mut host, prefix.unwrap_or(name));
            host.frame();
            host.tell(Told::LeftOff { task: Some(NEWEST) });
            host.tell(Told::Walk { by_key: true });
            let address = host.root.pages().address().clone();
            if sent {
                give(&mut host, Event::Send);
            } else {
                assert_eq!(host.event_of(&Key::Enter), Some(Event::Own { command }));
                host.report(Report::Key(Key::Enter));
            }
            assert_eq!(host.state().place(), Some(command), "{screen:?}, sent={sent}");
            assert_eq!(host.root.pages().address(), &address);
            assert!(host.root.pages().start().is_none());
            assert!(host.root.pages().tell().is_none());
            assert_eq!(host.words(), []);
            assert_eq!(host.state().left_off(), None);
            assert!(host.state().walk().is_none());
            assert!(text(&mut host).contains("or Esc to go back."));
            for character in "fix the README".chars() {
                host.frame();
                empty(&host.report(Report::Key(Key::Char(character))));
            }
            assert_eq!(host.words(), b"fix the README");
        }
    }
}

#[test]
fn an_offered_name_with_only_whitespace_opens_its_empty_place() {
    for (screen, words) in [
        (Screen::Home, "/change "),
        (Screen::Home, "/ask "),
        (Screen::Home, "/draft "),
        (Screen::Home, "/change   "),
        (Screen::Home, "/change\t\u{2003}"),
        (Screen::Home, "/tell "),
        (Screen::Chat, "/tell "),
        (Screen::Talk, "/tell "),
        (Screen::Chat, "/tell  \t"),
        (Screen::Talk, "/tell\u{2003}"),
    ] {
        let mut host = at(screen);
        typed(&mut host, words.as_bytes());
        host.frame();
        host.report(Report::Key(Key::End));
        host.report(Report::Key(Key::Left));
        let written = host.state().box_written();
        let address = host.root.pages().address().clone();
        host.frame();
        assert_eq!(host.event_of(&Key::Enter), Some(Event::Send));
        empty(&host.report(Report::Key(Key::Enter)));
        assert_eq!(host.root.pages().address(), &address, "{screen:?}, {words:?}: no navigation");
        assert!(host.root.pages().start().is_none(), "no StartChat");
        assert!(host.root.pages().tell().is_none(), "no Tell");
        assert!(host.words().is_empty(), "{screen:?}, {words:?}");
        assert!(host.state().place().is_some());
        assert!(host.state().box_written() > written, "cleared the box");
        assert_eq!(host.state().notice(), None, "no notice");
        host.frame();
        assert_eq!(host.focus.caret, 0, "caret at the empty box start");
    }
}

#[test]
fn an_unoffered_name_with_space_is_a_chats_message() {
    let mut host = reply_host();
    typed(&mut host, b"/change ");
    host.frame();
    assert_eq!(host.event_of(&Key::Enter), Some(Event::Send));
    let duties = host.report(Report::Key(Key::Enter));
    let duties = stored(&mut host, duties);
    let asks = sent_ask(&duties);
    let [(_, w::Ask::Message(message))] = asks.as_slice() else { panic!("chat message: {asks:?}") };
    assert_eq!(message.words(), b"/change ");
    assert_eq!(host.words(), []);
}

#[test]
fn first_words_start_the_places_chat_and_a_refusal_can_be_sent_again() {
    for (command, from, label, expected) in [
        (Own::Ask, Address::Home { project: 3 }, 7, b"fix the README".as_slice()),
        (Own::Change, Address::NewChat { draft: false }, 8, b"fix the README".as_slice()),
        (
            Own::Draft,
            Address::NewChat { draft: true },
            8,
            b"fix the README\n\nWrite this up as a draft beside us.".as_slice(),
        ),
    ] {
        let mut host = one_chat_host(false);
        give(&mut host, Event::Own { command });
        for character in "fix the README".chars() {
            host.frame();
            empty(&host.report(Report::Key(Key::Char(character))));
        }
        assert!(text(&mut host).contains("fix the README"));
        for refused in [true, false] {
            host.frame();
            assert_eq!(host.event_of(&Key::Enter), Some(Event::Send));
            let duties = host.report(Report::Key(Key::Enter));
            assert_eq!(host.root.pages().address(), &from);
            assert_eq!(host.state().place(), Some(command));
            assert_eq!(host.words(), b"fix the README");
            let mut duties = admit(&mut host, duties);
            for _ in 0..3 {
                if !sent_ask(&duties).is_empty() {
                    break;
                }
                let due = root::next_deadline(&host.root).expect("start retry");
                host.clocks(due, host.env.wall);
                let more = host.report(Report::Due);
                duties.asked.extend(admit(&mut host, more).asked);
            }
            let asks = sent_ask(&duties);
            let [(exchange, w::Ask::Create(create))] = asks.as_slice() else {
                panic!("{command:?}, refused={refused}: {asks:?}")
            };
            let bytes = skein_codec::piece::split(&mut Reader::new(create.tasks())).unwrap().unwrap();
            let task = w::NewTask::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
            assert_eq!((task.label(), task.spec().words()), (label, expected));
            if refused {
                let answer = w::Final::Refused(
                    w::Refused::new(&w::CEILINGS, w::RefusedParts { refusal: w::Refusal::NotOffered, at: None })
                        .unwrap(),
                );
                let duties = reply(&mut host, *exchange, final_bytes(&answer));
                admit(&mut host, duties);
                assert_eq!(host.state().place(), Some(command));
                assert_eq!(host.words(), b"fix the README");
                let refusal = temper_client_view::shared::refusal(&host.root, host.view.words().view()).unwrap();
                assert!(text(&mut host).contains(core::str::from_utf8(&refusal).unwrap()));
            } else {
                reply(&mut host, *exchange, created_bytes(NEWEST));
                assert_eq!(host.root.pages().address(), &Address::Chat { task: NEWEST });
                assert_eq!(host.state().place(), None);
                assert!(!text(&mut host).contains("or Esc to go back."));
            }
        }
    }
}

#[test]
fn back_ends_only_an_empty_place_and_a_list_gets_escape_first() {
    for screen in [Screen::Home, Screen::Chat, Screen::Talk] {
        let mut host = at(screen);
        let address = host.root.pages().address().clone();
        give(&mut host, Event::Own { command: Own::Tell });
        for words in [b"".as_slice(), b" \t\xe2\x80\x83"] {
            typed(&mut host, words);
            empty(&give(&mut host, Event::Send));
            assert_eq!(host.state().place(), Some(Own::Tell));
            assert_eq!(host.root.pages().address(), &address);
        }
        typed(&mut host, b"words");
        host.frame();
        assert_eq!(host.event_of(&Key::Escape), None);
        empty(&host.report(Report::Key(Key::Escape)));
        assert_eq!(host.words(), b"words");
        typed(&mut host, b"/");
        host.frame();
        assert_eq!(host.event_of(&Key::Escape), Some(Event::Dismiss));
        host.report(Report::Key(Key::Escape));
        assert_eq!(host.state().place(), Some(Own::Tell));
        host.frame();
        assert_eq!(host.event_of(&Key::Escape), Some(Event::Back));
        empty(&host.report(Report::Key(Key::Escape)));
        assert_eq!(host.state().place(), None);
        assert_eq!(host.root.pages().address(), &address, "{screen:?}: not Leave");
        assert!(host.state().notice().is_none());
    }
    let mut host = one_chat_host(false);
    give(&mut host, Event::Own { command: Own::Change });
    // A start's intermediate NewChat, as left by a refused start.
    host.start_at(Address::NewChat { draft: false });
    host.report(Report::Due);
    host.frame();
    assert_eq!(host.event_of(&Key::Escape), Some(Event::Back));
    host.report(Report::Key(Key::Escape));
    assert_eq!(host.root.pages().address(), &Address::Home { project: 3 });
    assert_eq!(host.state().place(), None);
}

#[test]
fn commands_keep_the_place_but_navigation_and_sign_out_end_it() {
    let mut host = picker_host();
    give(&mut host, Event::Own { command: Own::Ask });
    typed(&mut host, b"/model");
    give(&mut host, Event::Send);
    assert_eq!(host.root.pages().listing(), Some(root::pages::terminal::Listing::Model));
    host.frame();
    assert_eq!(
        host.event_of(&Key::Enter),
        Some(Event::Act { act: Act::Choice(root::pages::choice::Act::Model { index: 0, account: None }) })
    );
    host.report(Report::Key(Key::Enter));
    assert_eq!(host.state().place(), Some(Own::Ask));
    assert!(host.root.pages().listing().is_none());
    typed(&mut host, b"/change");
    give(&mut host, Event::Send);
    assert_eq!(host.state().place(), Some(Own::Change));
    typed(&mut host, b"/new");
    give(&mut host, Event::Send);
    assert_eq!(host.state().place(), None);
    give(&mut host, Event::Own { command: Own::Ask });
    give(&mut host, Event::Act { act: Act::Go { address: Address::Chat { task: NEWEST } } });
    assert_eq!(host.state().place(), None);
    give(&mut host, Event::Own { command: Own::Tell });
    host.report(Report::Heard(jig_client::Event::Streamed { message: jig_client::Streamed::SignedOut }));
    assert_eq!(host.state().place(), None);
}

#[test]
fn a_places_empty_box_has_no_launchpad_or_pane_keys() {
    for screen in [Screen::Home, Screen::Chat] {
        let mut host = match screen {
            Screen::Home => waiting_at(false),
            Screen::Chat => goal::host(true, false),
            Screen::Talk => unreachable!(),
        };
        give(&mut host, Event::Own { command: Own::Tell });
        for key in [Key::Up, Key::Down, Key::Control(b'p')] {
            host.frame();
            assert_eq!(host.event_of(&key), None, "{screen:?}, {key:?}");
            empty(&host.report(Report::Key(key)));
            assert_eq!(host.state().left_off(), None);
        }
        key_words(&mut host, "n");
        assert_eq!(host.words(), b"n");
        assert!(host.state().walk().is_none());
        assert!(!text(&mut host).contains("n to walk"));
        assert!(!text(&mut host).contains("Ctrl-P"));
    }
}

fn picker_host() -> Host {
    let (mut host, watch, _) = signed_host(true);
    let model = w::MeModel::new(
        &w::CEILINGS,
        w::MeModelParts {
            new_until: None,
            number: 1,
            source: w::ModelSource::Stock,
            estimated: false,
            name: b"model".as_slice().into(),
            provider: None,
            window: 8192,
            output: 1024,
            efforts: List::with_capacity(0),
            effort: None,
            prices: w::MePrices::new(
                &w::CEILINGS,
                w::MePricesParts { input: 1, cached: 1, cache_write: None, output: 1, unit: 1 },
            )
            .unwrap(),
            description: None,
            shown: None,
            accounts: List::from_boxed(Box::new([1])),
            picker: true,
        },
    )
    .unwrap();
    let mut me = launchpad_me(true).into_parts();
    let mut projects = Vec::new();
    let mut reader = Reader::new(&me.projects);
    while let Some(bytes) = skein_codec::piece::split(&mut reader).unwrap() {
        let mut project = w::MeProject::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap().into_parts();
        project.models = List::from_boxed(Box::new([model.clone()]));
        projects.push(project_bytes(&w::MeProject::new(&w::CEILINGS, project).unwrap()));
    }
    me.projects = pieces(&projects);
    let duties = delivery(&mut host, watch, &w::DeliveryValue::Me(w::Me::new(&w::CEILINGS, me).unwrap()));
    admit(&mut host, duties);
    host
}
