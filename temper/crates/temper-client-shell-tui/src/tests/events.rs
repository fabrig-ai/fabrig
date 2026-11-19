//! Host event translations (temper/docs/design/client/tui/README.md, 3).
use super::*;

#[test]
fn an_own_composer_opens_and_advances_its_write() {
    for command in [Own::Draft, Own::Change, Own::Ask] {
        let (mut host, _, _) = signed_host(true);
        host.tell(Told::Walk { by_key: true });
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
    host.tell(Told::Walk { by_key: true });
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
            host.report(Report::Key(Key::Down));
        }
        assert_eq!(give(&mut host, Event::Own { command: Own::Open }).open.as_deref(), address);
    }
}

#[test]
fn n_walks_only_from_an_empty_launchpad() {
    for (chat, words, after, event, walking, expected) in [
        (false, b"".as_slice(), None, Event::Own { command: Own::Needs }, true, b"".as_slice()),
        (false, b"a".as_slice(), None, Event::Typed { value: b"an".as_slice().into() }, false, b"an".as_slice()),
        (true, b"".as_slice(), None, Event::Typed { value: b"n".as_slice().into() }, false, b"n".as_slice()),
        (false, b"".as_slice(), Some('o'), Event::Own { command: Own::Needs }, false, b"no".as_slice()),
    ] {
        let mut host = waiting_at(chat);
        typed(&mut host, words);
        host.frame();
        assert_eq!(host.event_of(&Key::Char('n')), Some(event.clone()), "chat={chat}, words={words:?}");
        host.report(Report::Key(Key::Char('n')));
        if let Some(character) = after {
            host.frame();
            host.report(Report::Key(Key::Char(character)));
        }
        assert_eq!(host.words(), expected, "chat={chat}, words={words:?}, after={after:?}");
        assert_eq!(host.state().walk().is_some_and(|walk| walk.by_key() && !walk.taken()), walking);
    }
}

#[test]
fn typing_restores_only_an_untaken_walk_key() {
    for (by_key, taken, expected) in
        [(false, false, b"o".as_slice()), (true, true, b"o".as_slice()), (true, false, b"no".as_slice())]
    {
        let mut host = waiting_at(false);
        host.tell(Told::Walk { by_key });
        if taken {
            host.tell(Told::Walked { object: shared_question(&host) });
        }
        host.typed(b"o".as_slice().into());
        assert_eq!(host.words(), expected, "by_key={by_key}, taken={taken}");
        assert!(host.state().walk().is_none(), "typing ends the prompt");
        assert!(host.state().field().is_empty(), "walking keys belong only to the box");
    }
}

fn check_walk_step(step: &WalkStep, label: Option<&root::pages::needs::Label>, open: Option<&[u8]>) {
    let mut host = waiting_host();
    let object = shared_question(&host);
    host.tell(Told::Walk { by_key: false });
    let duties = give(&mut host, Event::Walk { object, step: step.clone() });
    let answered = host.root.pages().needs().answered(Wall::EPOCH, root::pages::needs::ANSWERED_ON_HOME, Some(3));
    assert_eq!(
        answered.iter().find(|line| line.object == object).map(|line| &line.label),
        label,
        "{step:?}: only a choice answers, in its words"
    );
    assert_eq!(duties.open.as_deref(), open, "{step:?}: page opened");
    assert!(host.state().walk().is_some_and(|walk| walk.shown(&object) && walk.taken()), "{step:?}: need shown");
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
        assert_eq!(host.state().walk().is_some_and(|walk| !walk.by_key()), walk, "{own:?}");
    }
}

#[test]
fn arrows_select_and_draw_the_left_off_chat() {
    let passed = {
        let mut host = left_off_host();
        let mut passed = true;
        for (key, task, title) in
            [(Key::Down, NEWEST, "Newest"), (Key::Down, OLDER, "Older"), (Key::Up, NEWEST, "Newest")]
        {
            passed &= host.event_of(&key) == Some(Event::LeftOff { task: Some(task) });
            host.report(Report::Key(key));
            passed &= host.state().left_off() == Some(task)
                && text(&mut host).lines().any(|line| line.starts_with(&format!(" ❯ Project   {title}, a chat")));
        }
        passed
    };
    assert!(passed);
}

#[test]
fn enter_opens_and_watches_the_left_off_chat() {
    let passed = {
        let mut host = left_off_host();
        let event = host.event_of(&Key::Down).expect("first chat");
        give(&mut host, event);
        host.frame();
        let selected = host.state().left_off() == Some(NEWEST)
            && host.event_of(&Key::Enter)
                == Some(Event::Act { act: Act::Go { address: Address::Chat { task: NEWEST } } });
        let duties = host.report(Report::Key(Key::Enter));
        read_exchange(&duties, "activity");
        watch(
            &duties,
            &w::Subject::Task(
                w::SubjectTask::new(&w::CEILINGS, w::SubjectTaskParts { task: NEWEST }).expect("task subject"),
            ),
        );
        selected && host.root.pages().address() == &Address::Chat { task: NEWEST }
    };
    assert!(passed);
}

#[test]
fn escape_clears_the_left_off_selection() {
    let mut host = selected_chat();
    let event = host.event_of(&Key::Escape);
    host.report(Report::Key(Key::Escape));
    assert_eq!(event, Some(Event::LeftOff { task: None }));
    assert_eq!(host.state().left_off(), None);
}

#[test]
fn typing_ends_left_off_selection_and_its_arrows() {
    let mut host = selected_chat();
    host.report(Report::Key(Key::Char('a')));
    assert_eq!(host.state().left_off(), None);
    assert_eq!(host.event_of(&Key::Down), None);
}

#[test]
fn a_command_list_takes_arrows_before_left_off_chats() {
    let mut host = selected_chat();
    host.report(Report::Key(Key::Char('/')));
    host.frame();
    let event = host.event_of(&Key::Down);
    host.report(Report::Key(Key::Down));
    assert_eq!(host.state().left_off(), Some(NEWEST));
    assert_eq!(event, None);
    assert!(text(&mut host).lines().any(|line| line.starts_with("❯ /change")));
}

fn selected_chat() -> Host {
    let mut host = left_off_host();
    host.report(Report::Key(Key::Down));
    host.frame();
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
    assert!(matches!(&asks[0].1, w::Ask::Create(create) if create.project() == 3 && one_asking_task(create.tasks())));
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
            && matches!(&asks[0].1, w::Ask::Message(message) if message.task() == 7 && message.words() == b"Continue")
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
                && matches!(&asks[0].1, w::Ask::Answer(answer) if answer.project() == 3 && matches!(answer.on(), w::AnswerOn::Question(question) if question.words() == if words { b"Use a comma".as_slice() } else { b"Comma".as_slice() }))
                && host.root.pages().chat().is_some_and(|chat| chat.words.is_empty())
                && host.state().aside().is_none()
        };
        assert!(passed, "words={words}");
    }
}
