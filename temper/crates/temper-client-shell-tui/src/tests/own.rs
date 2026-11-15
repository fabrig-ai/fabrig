//! Names left in the box and empty sends (client/tui/pages/README.md, 2).
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
fn an_own_name_leaves_the_caret_after_its_space() {
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
            let requested = host.state().focus();
            if sent {
                give(&mut host, Event::Send);
            } else {
                assert_eq!(host.event_of(&Key::Enter), Some(Event::Own { command }));
                host.report(Report::Key(Key::Enter));
            }
            let words = format!("{name} ");
            assert_eq!(host.words(), words.as_bytes(), "{screen:?}, {command:?}, sent={sent}");
            assert_eq!(usize::try_from(host.focus.caret).unwrap(), words.len(), "caret after space");
            assert_eq!(host.state().focus(), requested, "no new focus request");
            for character in "fix the README".chars() {
                host.frame();
                empty(&host.report(Report::Key(Key::Char(character))));
            }
            assert_eq!(host.words(), format!("{name} fix the README").as_bytes());
        }
    }
}

#[test]
fn an_offered_name_with_only_whitespace_sends_nothing() {
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
        let caret = host.focus.caret;
        let written = host.state().box_written();
        let address = host.root.pages().address().clone();
        host.frame();
        assert_eq!(host.event_of(&Key::Enter), Some(Event::Send));
        empty(&host.report(Report::Key(Key::Enter)));
        assert_eq!(host.root.pages().address(), &address, "{screen:?}, {words:?}: no navigation");
        assert!(host.root.pages().start().is_none(), "no StartChat");
        assert!(host.root.pages().tell().is_none(), "no Tell");
        assert_eq!(host.words(), words.as_bytes(), "{screen:?}, {words:?}: words kept");
        assert_eq!(host.focus.caret, caret, "caret kept, even before the end");
        assert_eq!(host.state().box_written(), written, "no box write");
        assert_eq!(host.state().notice(), None, "no notice");
        host.frame();
        assert_eq!(host.focus.caret, caret, "render keeps the caret too");
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
