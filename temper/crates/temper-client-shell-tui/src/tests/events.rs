//! Host event translations (temper/docs/design/client/tui/host.md, 1).
use super::*;

#[test]
fn an_own_composer_opens_and_advances_its_write() {
    for command in [Own::Draft, Own::Change, Own::Ask] {
        let (mut host, _, _) = signed_host(true);
        host.tell(Told::Walk { first: None });
        let before = host.state().box_written();
        let mut duties = Duties::default();
        host.event(Event::Own { command }, &mut duties);
        assert_eq!(host.state().place(), Some(command));
        assert!(host.state().box_written() > before);
        assert_eq!(host.words(), b"");
        assert!(!host.state().brief_shown());
        assert!(host.state().walk().is_none());
        assert!(duties.open.is_none());
        assert_eq!(duties.asked.len(), 0);
    }
}

fn check_own_view(event: &Event, brief: bool, walking: bool, open: Option<&[u8]>) {
    let (mut host, _, _) = signed_host(true);
    host.tell(Told::Walk { first: None });
    let mut duties = Duties::default();
    host.event(event.clone(), &mut duties);
    assert_eq!(host.words(), b"", "{event:?}");
    assert_eq!(host.state().brief_shown(), brief, "{event:?}");
    assert_eq!(host.state().walk().is_some(), walking, "{event:?}");
    assert_eq!(duties.open.as_deref(), open, "{event:?}");
    assert!(duties.asked.is_empty(), "{event:?}");
}

#[test]
fn brief_ends_the_walk_and_shows_the_brief() {
    check_own_view(&Event::Own { command: Own::Brief }, true, false, None);
}

#[test]
fn needs_begins_a_walk_without_the_brief() {
    check_own_view(&Event::Own { command: Own::Needs }, false, true, None);
}

#[test]
fn own_open_opens_the_project_and_ends_the_walk() {
    check_own_view(&Event::Own { command: Own::Open }, false, false, Some(b"http://127.0.0.1:8367/projects/3"));
}

#[test]
fn opening_a_page_keeps_the_walk() {
    let page = b"http://127.0.0.1:8367/projects/3".as_slice();
    check_own_view(&Event::Open { address: page.into() }, false, true, Some(page));
}

#[test]
fn end_walk_clears_the_walk() {
    check_own_view(&Event::EndWalk, false, false, None);
}

#[test]
fn open_uses_the_selected_chat_or_linked_goal() {
    for (projects, chat, selected, linked, address) in [
        (false, false, false, false, None),
        (true, false, false, false, Some(b"http://127.0.0.1:8367/projects/3".as_slice())),
        (true, true, false, false, Some(b"http://127.0.0.1:8367/chats/9".as_slice())),
        (true, false, true, false, Some(b"http://127.0.0.1:8367/chats/9".as_slice())),
        (true, true, false, true, Some(b"http://127.0.0.1:8367/tasks/25".as_slice())),
    ] {
        let mut host = if linked {
            goal::host(true, false)
        } else if projects {
            left_off_host()
        } else {
            signed_host(false).0
        };
        if chat && !linked {
            give(&mut host, Event::Act { act: Act::Go { address: Address::Chat { task: NEWEST } } });
        }
        if selected {
            host.report(Report::Key(Key::Up));
        }
        assert_eq!(give(&mut host, Event::Own { command: Own::Open }).open.as_deref(), address);
    }
}

#[test]
fn n_types_instead_of_walking() {
    let mut home = waiting_at(false);
    for (words, expected) in [(b"".as_slice(), b"n".as_slice()), (b"a".as_slice(), b"an".as_slice())] {
        check_n_types(&mut home, words, expected);
    }
}

#[test]
fn n_types_on_a_chat_with_a_question() {
    check_n_types(&mut waiting_at(true), b"", b"n");
}

fn check_n_types(host: &mut Host, words: &[u8], expected: &[u8]) {
    typed(host, words);
    host.frame();
    assert_eq!(host.event_of(&Key::Char('n')), Some(Event::Typed { value: expected.into() }));
    host.report(Report::Key(Key::Char('n')));
    assert_eq!(host.words(), expected);
    assert!(host.state().walk().is_none());
}

#[test]
fn typing_ends_the_walk_without_a_prefix() {
    let mut host = waiting_at(false);
    let object = shared_question(&host);
    for (from_row, taken) in [(false, false), (true, false), (true, true)] {
        host.tell(Told::Walk { first: from_row.then_some(object) });
        if taken {
            host.tell(Told::Walked { object });
        }
        host.typed(b"o".as_slice().into());
        assert_eq!(host.words(), b"o", "from_row={from_row}, taken={taken}");
        assert!(host.state().walk().is_none(), "typing ends the prompt");
        assert!(host.state().field().is_empty(), "walking keys belong only to the box");
    }
}

fn check_walk_step(step: &WalkStep, label: Option<&root::pages::needs::Label>, open: Option<&[u8]>) {
    let mut host = waiting_host();
    let object = shared_question(&host);
    host.tell(Told::Walk { first: None });
    let duties = give(&mut host, Event::Walk { object, step: step.clone() });
    let answered = host.root.pages().needs().answered(Wall::EPOCH, root::pages::needs::ANSWERED_ON_HOME, Some(3));
    assert_eq!(
        answered.iter().find(|line| line.object == object).map(|line| &line.label),
        label,
        "{step:?}: only a choice answers, in its words"
    );
    assert_eq!(duties.open.as_deref(), open, "{step:?}: page opened");
    assert!(host.state().walk().is_some_and(|walk| walk.shown(&object)), "{step:?}: need shown");
}

#[test]
fn a_walk_choice_answers_in_its_words() {
    check_walk_step(
        &WalkStep::Choose { words: b"Comma".as_slice().into() },
        Some(&root::pages::needs::Label::Choice(b"Comma".as_slice().into())),
        None,
    );
}

#[test]
fn a_walk_open_opens_without_answering() {
    let page = b"http://127.0.0.1:8367/projects/3".as_slice();
    check_walk_step(&WalkStep::Open { address: page.into() }, None, Some(page));
}

#[test]
fn a_walk_next_advances_without_answering() {
    check_walk_step(&WalkStep::Next, None, None);
}

#[test]
fn own_commands_clear_the_composer_and_standing_list() {
    for (own, brief, walk) in [(Own::Brief, true, false), (Own::Needs, false, true), (Own::Open, false, false)] {
        let mut host = waiting_at(false);
        give(&mut host, Event::Command { command: Command::Model });
        typed(&mut host, own.name());
        let before = host.state().box_written();
        give(&mut host, Event::Own { command: own });
        assert!(host.words().is_empty(), "{own:?}: command box cleared");
        assert!(host.root.pages().listing().is_none(), "{own:?}: standing list dismissed");
        assert!(host.state().box_written() > before, "{own:?}: box write advances");
        assert_eq!(host.state().brief_shown(), brief, "{own:?}");
        assert_eq!(host.state().walk().is_some_and(|walk| walk.first().is_none()), walk, "{own:?}");
    }
}

#[test]
fn arrows_and_emacs_aliases_select_and_draw_launchpad_rows() {
    let mut host = left_off_host();
    let first = Row::Chat { task: NEWEST };
    let second = Row::Chat { task: OLDER };
    for (key, row, title) in [
        (Key::Up, first, "Newest"),
        (Key::Control(b'n'), second, "Older"),
        (Key::BackTab, first, "Newest"),
        (Key::Down, second, "Older"),
        (Key::Control(b'p'), first, "Newest"),
    ] {
        assert_eq!(host.event_of(&key), Some(Event::Picked { row: Some(row) }));
        host.report(Report::Key(key));
        assert_eq!(host.state().picked(), Some((row, true)));
        let frame = host.frame();
        assert_eq!(frame.cursor(), None);
        let text = String::from_utf8(frame.text().into_vec()).expect("frame text");
        assert!(text.lines().any(|line| line.starts_with(&format!(" ❯ Project   {title}, a chat"))));
    }
}

#[test]
fn enter_and_right_open_and_watch_the_left_off_chat() {
    let mut host = selected_chat();
    assert_eq!(host.state().picked(), Some((Row::Chat { task: NEWEST }, true)));
    let address = Address::Chat { task: NEWEST };
    for key in [Key::Enter, Key::Right] {
        assert_eq!(host.event_of(&key), Some(Event::Act { act: Act::Go { address: address.clone() } }));
    }
    // The cross-project chat test gives Enter; here Right follows the same address.
    let duties = host.report(Report::Key(Key::Right));
    read_exchange(&duties, "activity");
    watch(&duties, &w::Subject::Task(w::SubjectTask { task: NEWEST }));
    assert_eq!(host.root.pages().address(), &address);
    assert_eq!(host.state().picked(), None);
    assert!(host.frame().cursor().is_some());
}

#[test]
fn escape_returns_to_the_box_with_its_words() {
    let mut host = left_off_host();
    host.frame();
    check_return_to_box(&mut host, Key::Escape, b"half", Event::Picked { row: None });
}

#[test]
fn typing_returns_to_the_box_and_appends_at_the_end() {
    let mut host = left_off_host();
    host.frame();
    check_return_to_box(&mut host, Key::Char('x'), b"halfx", Event::Typed { value: b"halfx".as_slice().into() });
}

#[test]
fn paste_returns_to_the_box_and_appends_at_the_end() {
    let mut host = left_off_host();
    host.frame();
    check_return_to_box(
        &mut host,
        Key::Paste(b"x".as_slice().into()),
        b"halfx",
        Event::Typed { value: b"halfx".as_slice().into() },
    );
}

#[test]
fn delete_keys_return_to_the_box_and_edit_at_the_end() {
    let mut host = left_off_host();
    host.frame();
    check_return_to_box(&mut host, Key::Backspace, b"hal", Event::Typed { value: b"hal".as_slice().into() });
    check_return_to_box(&mut host, Key::Delete, b"half", Event::Picked { row: None });
}

fn check_return_to_box(host: &mut Host, key: Key, words: &[u8], event: Event) {
    typed(host, b"half");
    host.report(Report::Key(Key::Home));
    assert_eq!(host.event_of(&Key::Down), None);
    host.report(Report::Key(Key::Up));
    assert_eq!(host.state().picked(), Some((Row::Chat { task: NEWEST }, true)));
    assert_eq!(host.words(), b"half");
    assert_eq!(host.event_of(&key), Some(event));
    host.report(Report::Key(key));
    assert_eq!(host.words(), words);
    assert_eq!(host.state().picked(), None);
    assert!(host.frame().cursor().is_some());
}

#[test]
fn a_commands_list_takes_arrows_before_the_launchpad_list() {
    let mut host = selected_chat();
    host.report(Report::Key(Key::Char('/')));
    host.frame();
    assert_eq!(host.state().picked(), Some((Row::Chat { task: NEWEST }, false)));
    assert_eq!(host.event_of(&Key::Down), None);
    host.report(Report::Key(Key::Down));
    assert_eq!(host.event_of(&Key::Enter), Some(Event::Own { command: Own::Change }));
    assert_eq!(host.event_of(&Key::Up), None);
    host.report(Report::Key(Key::Up));
    assert_eq!(host.event_of(&Key::Enter), Some(Event::Own { command: Own::Draft }));
    host.report(Report::Key(Key::Escape));
    assert_eq!(host.state().picked(), None);
}

#[test]
fn a_closed_picked_chat_returns_to_the_box() {
    let mut host = left_off_host();
    give(&mut host, Event::Picked { row: Some(Row::Chat { task: NEWEST }) });
    host.tell(Told::Closed { task: NEWEST });
    host.report(Report::Due);
    assert_eq!(host.state().picked(), None);
    assert!(host.frame().cursor().is_some());
    assert_eq!(host.event_of(&Key::Up), Some(Event::Picked { row: Some(Row::Chat { task: OLDER }) }));
}

#[test]
fn a_need_row_starts_the_walk_before_up_moves_the_prompt() {
    let mut host = waiting_at(false);
    let object = shared_question(&host);
    for key in [Key::Right, Key::Enter] {
        give(&mut host, Event::Picked { row: Some(Row::Need { object }) });
        assert_eq!(host.event_of(&key), Some(Event::WalkFrom { object }));
        host.report(Report::Key(key));
        assert_eq!(host.state().picked(), None);
        assert_eq!(host.state().walk().unwrap().first(), Some(object));
        host.frame();
        host.report(Report::Key(Key::Down));
        assert_eq!(
            host.event_of(&Key::Enter),
            Some(Event::Walk { object, step: WalkStep::Choose { words: b"Semicolon".as_slice().into() } })
        );
        assert_eq!(host.event_of(&Key::Up), None);
        host.report(Report::Key(Key::Up));
        assert_eq!(
            host.event_of(&Key::Enter),
            Some(Event::Walk { object, step: WalkStep::Choose { words: b"Comma".as_slice().into() } })
        );
        give(&mut host, Event::EndWalk);
    }
}

#[test]
fn a_goal_row_opens_its_page() {
    let mut host = goal_host();
    let row = Row::Goal { task: NEWEST };
    let address = b"http://127.0.0.1:8367/tasks/9".as_slice();
    for key in [Key::Enter, Key::Right] {
        assert_eq!(host.event_of(&Key::Up), Some(Event::Picked { row: Some(row) }));
        give(&mut host, Event::Picked { row: Some(row) });
        assert_eq!(host.event_of(&key), Some(Event::Open { address: address.into() }));
        assert_eq!(host.report(Report::Key(key)).open.as_deref(), Some(address));
        assert_eq!(host.state().picked(), None);
        assert!(host.frame().cursor().is_some());
    }
}

#[test]
fn a_change_row_opens_its_place() {
    let (mut host, _, _) = signed_host(true);
    for key in [Key::Enter, Key::Right] {
        give(&mut host, Event::Picked { row: Some(Row::Start { command: Own::Change }) });
        assert_eq!(host.event_of(&key), Some(Event::Own { command: Own::Change }));
        host.report(Report::Key(key));
        assert_eq!(host.state().place(), Some(Own::Change));
        assert_eq!(host.state().picked(), None);
        assert!(host.frame().cursor().is_some());
        give(&mut host, Event::Back);
    }
}

#[test]
fn start_lists_roles_and_a_role_row_opens_a_talk() {
    let mut host = talk::roles_host();
    host.frame();
    let role = Row::Role { project: 3, task: 7 };
    let first = Event::Picked { row: Some(role) };
    assert_eq!(host.event_of(&Key::Control(b'p')), Some(first.clone()));
    give(&mut host, first.clone());
    let draft = Row::Start { command: Own::Draft };
    assert_eq!(host.event_of(&Key::Tab), Some(Event::Picked { row: Some(draft) }));
    host.report(Report::Key(Key::Tab));
    assert_eq!(host.state().picked(), Some((draft, true)));
    let talk = Row::Start { command: Own::Talk };
    give(&mut host, Event::Picked { row: Some(talk) });
    assert_eq!(host.event_of(&Key::Enter), Some(Event::Own { command: Own::Talk }));
    host.report(Report::Key(Key::Enter));
    assert!(host.state().talk_list());
    assert_eq!(host.state().picked(), None);
    assert!(host.frame().cursor().is_some());
    host.report(Report::Key(Key::Escape));
    give(&mut host, first);
    let address = Address::Talk { project: 3, task: 7 };
    assert_eq!(host.event_of(&Key::Right), Some(Event::Act { act: Act::Go { address: address.clone() } }));
    host.report(Report::Key(Key::Right));
    assert_eq!(host.root.pages().address(), &address);
    assert!(host.frame().cursor().is_some());
}

fn selected_chat() -> Host {
    let mut host = left_off_host();
    host.report(Report::Key(Key::Up));
    host
}

#[test]
fn stop_closes_a_bare_host() {
    let (mut host, _, _) = signed_host(true);
    assert!(give(&mut host, Event::Stop).close);
}

#[test]
fn stay_lowers_the_quit_question() {
    let (mut host, _, _) = signed_host(true);
    let passed = {
        host.tell(temper_client_tui::Told::Quitting);
        !give(&mut host, Event::Stay).close && !host.state().quitting()
    };
    assert!(passed);
}

#[test]
fn log_is_a_front_only_command() {
    for front in [false, true] {
        let (mut host, _, _) = signed_host(true);
        let passed = {
            if front {
                host.tell(temper_client_tui::Told::Front { set_up: false, box_log: Box::default() });
            }
            typed(&mut host, b"/log");
            let before = host.state().box_written();
            let duties = give(&mut host, Event::Log);
            duties.asked.is_empty()
                && host.state().log_shown() == front
                && host.root.pages().chats().is_some_and(|chats| chats.words.is_empty() == front)
                && (host.state().box_written() > before) == front
        };
        assert!(passed, "front={front}");
    }
}

#[test]
fn typed_home_words_reach_the_composer() {
    let (mut host, _, _) = signed_host(true);
    typed(&mut host, b"Please help");
    assert!(host.root.pages().chats().is_some_and(|chats| chats.words.as_ref() == b"Please help"));
}

#[test]
fn sending_home_words_creates_one_asking_task() {
    let (mut host, _, _) = signed_host(true);
    typed(&mut host, b"Please help");
    let duties = give(&mut host, Event::Send);
    let asks = sent_ask(&duties);
    assert_eq!(asks.len(), 1);
    assert!(
        matches!(&asks[0].1, w::Ask::Create(create) if create.project == 3 && one_asking_task(create.tasks.as_ref()))
    );
    assert!(host.state().box_written() > 0);
}

#[test]
fn typed_chat_words_reach_the_composer() {
    let (mut host, _, _) = signed_host(true);
    let passed = {
        host.step(root::Event::Went { place: Place::Address(Address::Chat { task: 9 }) });
        host.report(Report::Due);
        typed(&mut host, b"Chat words");
        host.root.pages().chat().is_some_and(|chat| chat.words.as_ref() == b"Chat words")
    };
    assert!(passed);
}

#[test]
fn sending_chat_words_emits_one_message() {
    let passed = {
        let mut host = waiting_host();
        let object =
            temper_client_view::shared::question(&host.root, host.view.words().view()).expect("question").object;
        give(
            &mut host,
            Event::Act {
                act: Act::Needs(root::pages::needs::Act::Choose { object, words: b"Comma".as_slice().into() }),
            },
        );
        typed(&mut host, b"Continue");
        let before = host.state().box_written();
        let duties = give(&mut host, Event::Send);
        let asks = sent_ask(&duties);
        asks.len() == 1
            && matches!(&asks[0].1, w::Ask::Message(message) if message.task == 7 && message.words.as_ref() == b"Continue")
            && host.state().box_written() > before
    };
    assert!(passed);
}

#[test]
fn an_unknown_command_is_noticed_and_cleared() {
    let (mut host, _, _) = signed_host(true);
    let passed = {
        typed(&mut host, b"/zz");
        let before = host.state().box_written();
        let duties = give(&mut host, Event::Send);
        duties.asked.is_empty()
            && host.state().notice() == Some(&temper_client_tui::Notice::Unknown { words: b"/zz".as_slice().into() })
            && host.root.pages().chats().is_some_and(|chat| chat.words.is_empty())
            && host.state().box_written() > before
    };
    assert!(passed);
}

#[test]
fn sending_without_a_project_does_not_write() {
    let (mut host, _, _) = signed_host(false);
    let passed = {
        typed(&mut host, b"Please help");
        let before = host.state().box_written();
        give(&mut host, Event::Send).asked.is_empty() && host.state().box_written() == before
    };
    assert!(passed);
}

#[test]
fn help_clears_the_box_and_adds_its_note() {
    let (mut host, _, _) = signed_host(true);
    let passed = {
        typed(&mut host, b"/help");
        let before = host.state().box_written();
        give(&mut host, Event::Command { command: Command::Help });
        host.root.pages().notes().last() == Some(&terminal::Note::Help)
            && host.root.pages().chats().is_some_and(|chats| chats.words.is_empty())
            && host.state().box_written() > before
    };
    assert!(passed);
}

#[test]
fn dismiss_clears_the_composer() {
    let (mut host, _, _) = signed_host(true);
    let passed = {
        typed(&mut host, b"/");
        let before = host.state().box_written();
        give(&mut host, Event::Dismiss);
        host.root.pages().chats().is_some_and(|chats| chats.words.is_empty()) && host.state().box_written() > before
    };
    assert!(passed);
}

#[test]
fn escape_dispatches_the_views_event() {
    let (mut host, _, _) = signed_host(true);
    let passed = {
        typed(&mut host, b"/");
        host.frame();
        let expected = temper_client_tui::escape(&host.view);
        let event = host.event_of(&Key::Escape);
        let mut direct = host;
        let (mut keyed, _, _) = signed_host(true);
        typed(&mut keyed, b"/");
        keyed.frame();
        if let Some(event) = expected.clone() {
            give(&mut direct, event);
        } else {
            direct.report(Report::Due);
        }
        keyed.report(Report::Key(Key::Escape));
        event == expected
            && keyed.root.pages().chats().map(|chats| &chats.words)
                == direct.root.pages().chats().map(|chats| &chats.words)
            && keyed.state().box_written() == direct.state().box_written()
            && keyed.state().aside() == direct.state().aside()
    };
    assert!(passed);
}

#[test]
fn left_is_only_a_caret_key_on_the_launchpad() {
    let (mut host, _, _) = signed_host(true);
    host.frame();
    for words in ["", "hi"] {
        if !words.is_empty() {
            host.report(Report::Key(Key::Paste(words.as_bytes().into())));
            host.frame();
            host.report(Report::Key(Key::Home));
        }
        assert_eq!(host.event_of(&Key::Left), None);
        empty(&host.report(Report::Key(Key::Left)));
        assert_eq!(host.root.pages().address(), &Address::Home { project: 3 });
        assert_eq!(host.words(), words.as_bytes());
    }
}

#[test]
fn left_leaves_a_chat_while_its_question_stands() {
    let mut host = waiting_host();
    let object = shared_question(&host);
    left_leaves_with_prompt(&mut host, Event::Aside { object });
}

#[test]
fn left_leaves_a_chat_while_its_reply_choices_stand() {
    left_leaves_with_prompt(&mut reply_host(), Event::ReplyAside { turn: REPLY_TURN });
}

fn left_leaves_with_prompt(host: &mut Host, esc: Event) {
    host.frame();
    assert_eq!(host.words(), []);
    assert_eq!(host.event_of(&Key::Escape), Some(esc));
    assert_eq!(host.event_of(&Key::Left), Some(Event::Command { command: Command::New }));
    host.report(Report::Key(Key::Left));
    assert_eq!(host.root.pages().address(), &Address::Home { project: 3 });
}

#[test]
fn escape_sets_aside_the_chats_question() {
    let mut host = waiting_host();
    let object = jig_client::contract::ObjectKey::Question { task: 7, message: 7 };
    assert!(has_option(&host, b"1. Comma"));
    assert_eq!(host.event_of(&Key::Escape), Some(Event::Aside { object }));
    empty(&host.report(Report::Key(Key::Escape)));
    assert_eq!(host.state().aside(), Some(object));
    assert_eq!(host.root.pages().address(), &Address::Chat { task: 7 });
    assert!(!has_option(&host, b"1. Comma"));
}

#[test]
fn an_aside_without_a_question_expires() {
    let (mut host, _, _) = signed_host(true);
    let passed = {
        let object = jig_client::contract::ObjectKey::Task(9);
        host.event(Event::Aside { object }, &mut Duties::default());
        let aside = host.state().aside() == Some(object);
        // No question stands: an aside expires in the same report.
        host.report(Report::Due);
        aside && host.state().aside().is_none()
    };
    assert!(passed);
}

#[test]
fn a_navigation_act_changes_the_address() {
    let (mut host, _, _) = signed_host(true);
    let passed = {
        give(&mut host, Event::Act { act: Act::Go { address: Address::Chat { task: 9 } } });
        host.root.pages().address() == &Address::Chat { task: 9 }
    };
    assert!(passed);
}

#[test]
fn resize_tells_the_tui_its_columns() {
    let (mut host, _, _) = signed_host(true);
    let passed = {
        let at_start = host.state().columns() == 80;
        host.report(Report::Resize(Size { columns: 40, rows: 12 }));
        at_start && host.state().columns() == 40
    };
    assert!(passed);
}

#[test]
fn send_and_navigation_keep_an_unknown_command_notice() {
    for send in [false, true] {
        let (mut host, _, _) = signed_host(true);
        let passed = {
            host.tell(temper_client_tui::Told::Notice {
                notice: Some(temper_client_tui::Notice::Unknown { words: b"/zz".as_slice().into() }),
            });
            if send {
                give(&mut host, Event::Send);
            } else {
                host.out.push(root::Request::Show { address: Address::Chat { task: 9 }, replace: false });
                host.report(Report::Due);
            }
            host.state().notice().is_some()
        };
        assert!(passed);
    }
}

#[test]
fn answers_wait_for_the_hold_then_send_their_words() {
    for words in [false, true] {
        let passed = {
            let mut host = waiting_host();
            let object =
                temper_client_view::shared::question(&host.root, host.view.words().view()).expect("question").object;
            let mut aside = true;
            let duties = if words {
                let mut duties = Duties::default();
                host.event(Event::Aside { object }, &mut duties);
                aside = host.state().aside() == Some(object);
                typed(&mut host, b"Use a comma");
                give(&mut host, Event::Send)
            } else {
                give(
                    &mut host,
                    Event::Act {
                        act: Act::Needs(root::pages::needs::Act::Choose { object, words: b"Comma".as_slice().into() }),
                    },
                )
            };
            assert!(duties.asked.is_empty(), "answers wait through the undo hold");
            host.clocks(Time::from_nanos(root::pages::needs::HOLD.as_nanos()), Wall::EPOCH);
            let duties = host.report(Report::Due);
            let duties = stored(&mut host, duties);
            let asks = sent_ask(&duties);
            aside
                && asks.len() == 1
                && matches!(&asks[0].1, w::Ask::Answer(answer) if answer.project == 3 && matches!( &answer.on, w::AnswerOn::Question(question) if question.words.as_ref() == if words { b"Use a comma".as_slice() } else { b"Comma".as_slice() }))
                && host.root.pages().chat().is_some_and(|chat| chat.words.is_empty())
                && host.state().aside().is_none()
        };
        assert!(passed, "words={words}");
    }
}

#[test]
fn open_on_a_named_chat_uses_its_https_origin() {
    let mut host = left_off_host();
    give(&mut host, Event::Act { act: Act::Go { address: Address::Chat { task: NEWEST } } });
    host.tell(Told::Address {
        address: temper_client_tui::Origin::Https { name: b"dev.fabrig.ai".as_slice().into(), port: 443 },
    });
    typed(&mut host, b"/open");
    host.frame();
    let duties = host.report(Report::Key(Key::Enter));
    assert_eq!(duties.open.as_deref(), Some(b"https://dev.fabrig.ai/chats/9".as_slice()));
}
