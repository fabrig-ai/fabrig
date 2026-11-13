//! A chat's terminal frame (temper/docs/design/client/pages/README.md).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary tests inspect semantic trees and bindings"
)]
pub(super) use super::fixtures::terminal::{
    QUESTION_OBJECT, command_root, command_root_with_watch, state_root, terminal_question_root,
};
use super::{
    document::{ProposalState, document_root},
    fixtures as f,
};
use crate::{
    look,
    words::{Shell, Words},
};
use jig_client::contract as c;
use skein_ui::{Role, State, Style, Tree};
use temper_client_domain::{
    self as d, Action, Address, Event,
    pages::{Act, chat::document::Mode, terminal},
};

#[test]
fn a_talk_uses_chat_layout_without_chat_commands_request_draft_or_picker() {
    use terminal::Command as C;
    let source = command_root(0);
    let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
    let mut offers = skein_lib::List::with_capacity(task.acts.len() + 1);
    for offer in &task.acts {
        offers.push(offer.clone()).unwrap();
    }
    offers.push(c::Offered { act: c::ActKind::Talk, admission: c::Admission::Allowed, effect: None }).unwrap();
    task.acts = offers;
    task.spec.specification.words =
        c::DocumentText::complete(c::Text::new(b"Role rules, not a request".as_slice().into(), 64).unwrap());
    let (mut root, _, watch) =
        f::root_at(Address::Talk { project: f::PROJECT, task: f::chat::TASK }, &f::domain_limits());
    f::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
    );
    let task = root.pages().chat().unwrap().task(root.client()).unwrap();
    for command in [C::Pause, C::Close, C::StartOver, C::Model, C::Effort, C::Draft, C::Goal] {
        assert!(!crate::shared::offered(&root, Some(task), command), "{command:?}");
    }
    let view = view(&root, Shell::Browser);
    assert!(styled(view.tree(), look::SESSION));
    assert!(f::find(view.tree(), Role::TextArea, b"Message").is_some());
    assert!(!styled(view.tree(), look::MODEL));
    assert!(!styled(view.tree(), look::DOC));
    for name in [b"Pause".as_slice(), b"Close", b"Start over", b"+ Draft"] {
        assert!(f::find(view.tree(), Role::Button, name).is_none(), "{name:?}");
    }
    for node in view.tree().nodes() {
        for run in &node.text {
            assert_ne!(run.text.as_ref(), b"Role rules, not a request");
        }
    }
}

fn act(root: &mut d::Domain, act: Act) {
    f::root_event(root, Event::Act { action: Action::Pages(act) });
}
pub(super) fn view(root: &d::Domain, shell: Shell) -> crate::View {
    let words = Words::english(f::FORGE, 512, shell).unwrap();
    let mut limits = f::domain_limits();
    limits.client.domain.text = 512;
    limits.client.domain.long_text = 65536;
    let bounds = crate::Limits::of(&limits, &words).unwrap();
    let mut view = crate::View::new(&bounds, &limits, words, f::FORGE);
    f::build(&mut view, root);
    view
}
fn styled(tree: &Tree, token: Style) -> bool {
    tree.nodes().iter().any(|n| n.styles.as_slice().contains(&token))
}
#[test]
fn the_terminal_draws_a_chats_pages_in_the_browser_alone() {
    for address in [
        Address::Chat { task: f::chat::TASK },
        Address::NewChat { draft: false },
        Address::NewChat { draft: true },
        Address::Ask { project: f::PROJECT },
        Address::Chats { filter: d::ChatsFilter::All },
        Address::Home { project: f::PROJECT },
    ] {
        let eligible = terminal::draws(&address);
        for on in [false, true] {
            let mut root = f::chat::messaging_root();
            if address != *root.pages().address() {
                f::root_event(&mut root, Event::Went { place: d::Place::Address(address.clone()) });
            }
            if on {
                act(&mut root, Act::Terminal(terminal::Act::Enter));
            }
            for shell in [Shell::Browser, Shell::Terminal] {
                let mut view = view(&root, shell);
                let terminal = eligible && on && shell == Shell::Browser;
                assert_eq!(
                    view.tree()
                        .nodes()
                        .iter()
                        .any(|n| n.role == Role::Main && n.styles.as_slice().contains(&look::TERMINAL)),
                    terminal,
                    "{address:?} {shell:?} {on}"
                );
                assert_eq!(styled(view.tree(), look::RAIL), !terminal, "rail: {address:?} {shell:?} {on}");
                if terminal {
                    assert!(!styled(view.tree(), look::TOP));
                }
                assert_eq!(styled(view.tree(), look::TABBAR), shell == Shell::Browser && !terminal, "tabs");
                assert!(f::find(view.tree(), Role::Button, b"Terminal, full screen").is_none());
                if !terminal && address == (Address::Ask { project: f::PROJECT }) {
                    let right = view
                        .tree()
                        .nodes()
                        .iter()
                        .find(|node| node.styles.as_slice().contains(&look::SESSION_RIGHT))
                        .unwrap();
                    assert_eq!(right.size, 1, "Ask's bar has no right-side content");
                }
                if terminal && address == (Address::NewChat { draft: true }) {
                    let id = f::find(view.tree(), Role::Button, b"Draft").unwrap();
                    assert!(view.tree().find(id).unwrap().states.as_slice().contains(&State::Pressed));
                    let decoded = crate::act(&mut view, id, skein_ui::Act::Press).unwrap();
                    let before = root.pages().chats().unwrap().words.clone();
                    f::root_event(&mut root, Event::Act { action: decoded });
                    assert_eq!(root.pages().address(), &Address::NewChat { draft: false });
                    assert_eq!(root.pages().chats().unwrap().words, before);
                }
                if terminal {
                    act(&mut root, Act::Terminal(terminal::Act::Theme));
                    act(&mut root, Act::Terminal(terminal::Act::Font));
                    let changed = self::view(&root, shell);
                    assert!(styled(changed.tree(), look::TERMINAL_LIGHT));
                    assert!(styled(changed.tree(), look::TERMINAL_PLEX));
                }
            }
        }
    }
}
fn writing_root(reply: &[u8]) -> d::Domain {
    let (mut root, _, run_watch) = observed_writing_root();
    let mut run = writing_state();
    run.writing = Some(reply_text(reply));
    watch_delivery(&mut root, run_watch, c::Delivery::RunState(Box::new(run)));
    root
}
fn writing_state() -> c::RunState {
    c::RunState { task: f::chat::TASK, attempt: Some(f::chat::ATTEMPT), committed_turn: 0, writing: None, call: None }
}
fn reply_text(reply: &[u8]) -> c::RunText {
    c::RunText {
        writing: None,
        attempt: f::chat::ATTEMPT,
        began: Some(f::NOW),
        turn: c::ProvisionalTurnName { attempt: f::chat::ATTEMPT, number: f::chat::TURN, retry: f::chat::RETRY },
        from: 0,
        through: u32::try_from(reply.len()).unwrap(),
        text: c::Text::new(reply.into(), f::chat::TEXT).unwrap(),
        cut: false,
        call: None,
    }
}

#[test]
fn the_terminal_says_where_it_is_and_what_runs() {
    for (state, running, paused, ended) in [
        (0, false, false, false),
        (1, true, false, false),
        (2, true, false, false),
        (3, false, true, false),
        (4, false, false, true),
        (5, true, false, false),
    ] {
        let mut root = state_root(state);
        let browser = view(&root, Shell::Browser);
        let status = browser
            .tree()
            .nodes()
            .iter()
            .find(|n| n.styles.as_slice().contains(&look::STATUS_LINE))
            .and_then(|node| node.name.clone());
        act(&mut root, Act::Terminal(terminal::Act::Enter));
        let mut view = view(&root, Shell::Browser);
        let tree = view.tree();
        assert!(f::find(tree, Role::Heading { level: 1 }, b"T9 \xc2\xb7 Hello").is_some());
        assert!(tree.nodes().iter().any(|n| n.text.iter().any(|r| r.text.as_ref() == "✦".as_bytes())));
        for name in [b"Shop".as_slice(), b"/", b"T9"] {
            assert!(tree.nodes().iter().any(|n| n.name.as_deref() == Some(name)), "{name:?}");
        }
        let run = tree.nodes().iter().find(|n| n.styles.as_slice().contains(&look::TERMINAL_RUN));
        assert_eq!(run.is_some(), running, "state {state}");
        if let Some(run) = run {
            assert_eq!(run.name, status);
            assert_eq!(run.role, Role::Status);
            let foot = tree.nodes().iter().find(|n| n.styles.as_slice().contains(&look::SESSION_FOOT)).unwrap();
            assert_eq!(foot.role, Role::Form);
            assert_eq!(super::connect_forge::parent(tree, run.id), Some(foot.id), "running line's form: state {state}");
            let at = tree.nodes().iter().position(|n| n.id == foot.id).unwrap();
            assert_eq!(tree.nodes()[at.checked_add(1).unwrap()].id, run.id, "running line first: state {state}");
            let next = at.checked_add(1).unwrap().checked_add(usize::try_from(run.size).unwrap()).unwrap();
            let composer = &tree.nodes()[next];
            assert!(
                composer.styles.as_slice().contains(&look::SESSION_COMPOSER),
                "composer follows running line: state {state}"
            );
            assert_eq!(super::connect_forge::parent(tree, composer.id), Some(foot.id));
        }
        assert!(styled(tree, look::SESSION_COMPOSER));
        assert!(styled(tree, look::TERMINAL_LINE));
        if !ended {
            let line = tree.nodes().iter().position(|n| n.styles.as_slice().contains(&look::TERMINAL_LINE)).unwrap();
            let size = usize::try_from(tree.nodes()[line].size).unwrap();
            let model =
                tree.nodes()[line..line.checked_add(size).unwrap()].iter().find(|n| n.role == Role::Button).unwrap();
            assert!(!model.styles.as_slice().contains(&look::TERMINAL_BTN), "plain model button: state {state}");
            let hint = if paused { "paused · a message resumes it" } else { "/ for commands" };
            assert!(f::find(tree, Role::Paragraph, hint.as_bytes()).is_some(), "state {state}");
            assert!(f::find(tree, Role::TextArea, b"Message").is_some());
            assert!(f::find(tree, Role::Paragraph, "7 spent · of 7".as_bytes()).is_some());
        }
        for name in [b"Send".as_slice(), b"Pause", b"Resume", b"Close", b"Start over", "Start goal…".as_bytes()] {
            assert!(f::find(tree, Role::Button, name).is_none(), "{name:?}");
        }
        let exit = f::find(tree, Role::Button, b"Exit").unwrap();
        assert!(tree.find(exit).unwrap().styles.as_slice().contains(&look::TERMINAL_BTN));
        assert_eq!(
            crate::act(&mut view, exit, skein_ui::Act::Press),
            Some(Action::Pages(Act::Terminal(terminal::Act::Leave)))
        );
    }
    for (body, mode, state, draft) in [
        (Some(b"# Draft\n".as_slice()), Mode::Split, ProposalState::Ordinary, Some(true)),
        (Some(b"# Draft\n".as_slice()), Mode::Chat, ProposalState::Ordinary, Some(false)),
        (None, Mode::Chat, ProposalState::DraftOffered, None),
        (None, Mode::Chat, ProposalState::Ordinary, None),
    ] {
        let mut root = document_root(body, mode, &[], state);
        act(&mut root, Act::Terminal(terminal::Act::Enter));
        let mut view = view(&root, Shell::Browser);
        if let Some(pressed) = draft {
            let id = f::find(view.tree(), Role::Button, b"Draft").unwrap();
            let node = view.tree().find(id).unwrap();
            assert_eq!(node.states.as_slice().contains(&State::Pressed), pressed);
            assert_eq!(node.shortcut, Some(skein_ui::Shortcut { key: b'\\', modifier: skein_ui::Modifier::Primary }));
            assert_eq!(
                crate::act(&mut view, id, skein_ui::Act::Press),
                Some(Action::Pages(Act::Mode { mode: if pressed { Mode::Chat } else { Mode::Split } }))
            );
        } else {
            assert!(f::find(view.tree(), Role::Button, b"Draft").is_none());
            let plus = f::find(view.tree(), Role::Button, b"+ Draft");
            assert_eq!(plus.is_some(), state == ProposalState::DraftOffered);
            if let Some(id) = plus {
                assert_eq!(crate::act(&mut view, id, skein_ui::Act::Press), Some(Action::Pages(Act::AddDraft)));
            }
        }
    }
}

#[test]
fn a_writing_run_says_working_before_the_first_answer() {
    let (mut root, task_watch, run_watch) = observed_writing_root();
    let mut run = writing_state();
    watch_delivery(&mut root, run_watch, c::Delivery::RunState(Box::new(run.clone())));
    assert!(root.pages().chat().unwrap().thread.lines().is_empty());
    let browser = view(&root, Shell::Browser);
    assert!(f::find(browser.tree(), Role::Status, b"Working").is_some(), "the status line says Working");
    act(&mut root, Act::Terminal(terminal::Act::Enter));
    assert_working_line(&root, Some(b"Working"));
    run.writing = Some(reply_text(b"Hello"));
    watch_delivery(&mut root, run_watch, c::Delivery::RunState(Box::new(run.clone())));
    assert!(skein_lib::bytes::find(&f::text(view(&root, Shell::Browser).tree()), b"Hello").is_some());
    assert_working_line(&root, Some(b"Working"));
    run.writing = None;
    run.attempt = None;
    run.committed_turn = f::chat::TURN;
    watch_delivery(&mut root, run_watch, c::Delivery::RunState(Box::new(run)));
    commit_reply(&mut root, task_watch);
    assert!(skein_lib::bytes::find(&f::text(view(&root, Shell::Browser).tree()), b"Hello").is_some());
    assert_working_line(&root, None);
}

fn assert_working_line(root: &d::Domain, expected: Option<&[u8]>) {
    assert_eq!(crate::shared::running(root, &f::browser_words()).as_deref(), expected);
    let terminal = view(root, Shell::Browser);
    let running = terminal.tree().nodes().iter().find(|node| node.styles.as_slice().contains(&look::TERMINAL_RUN));
    assert_eq!(running.and_then(|node| node.name.as_deref()), expected);
}

fn commit_reply(root: &mut d::Domain, task_watch: skein_lib::Token) {
    let mut task = root.pages().chat().unwrap().task(root.client()).unwrap().clone();
    task.activity = u64::from(f::chat::TURN);
    task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Idle);
    task.chip.details.details.observation = None;
    task.waits_on = c::WaitsOn::News;
    let mut out = watch_delivery(root, task_watch, c::Delivery::Task(Box::new(task)));
    let mut reply = f::first_run::turn(
        u64::from(f::chat::TURN),
        f::chat::ATTEMPT,
        f::chat::TURN,
        &[f::first_run::message(c::Speaker::Assistant, &[c::Block::Text(f::first_run::text(b"Hello"))])],
        f::chat::TEXT,
    );
    reply.source.task = f::chat::TASK;
    reply.decoded.as_mut().unwrap().task = f::chat::TASK;
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Activity { .. }) },
                },
        } = request
        {
            f::root_event(
                root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::ReadDone {
                            read,
                            result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(
                                c::ActivityPage {
                                    rows: skein_lib::List::from_boxed(Box::new([reply])),
                                    before: None,
                                    after: None,
                                    cut: false,
                                },
                            )))),
                        },
                    },
                },
            );
            assert_eq!(
                root.pages().chat().unwrap().thread.committed_turn,
                Some(c::TurnName { attempt: f::chat::ATTEMPT, number: f::chat::TURN })
            );
            return;
        }
    }
    panic!("the completed task asks for its committed reply");
}

fn watch_delivery(
    root: &mut d::Domain,
    watch: skein_lib::Token,
    delivery: c::Delivery,
) -> skein_lib::Queue<d::Request> {
    let mut out = skein_lib::Queue::uncapped();
    d::step(
        root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) },
            },
        },
        &mut out,
    );
    out
}

fn observed_writing_root() -> (d::Domain, skein_lib::Token, skein_lib::Token) {
    let source = state_root(1);
    let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
    task.chip.executor = c::ChipExecutor::Agent { charter: 37 };
    task.activity = 0;
    task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Running { attempt: f::chat::ATTEMPT });
    task.chip.details.details.observation = Some(c::ChipObservation {
        channel: skein_lib::Token::new(1),
        work: c::ChipWork::Writing,
        since: f::NOW,
        reported_at: f::NOW,
        started: None,
    });
    let (mut root, me_watch, task_watch, _, _) = f::root_at_with_activity(
        Address::Chat { task: f::chat::TASK },
        &f::domain_limits(),
        b"Shop",
        None,
        Some(c::ActivityPage { rows: skein_lib::List::with_capacity(0), before: None, after: None, cut: false }),
        None,
    );
    f::deliver_me(&mut root, me_watch, source.pages().me(source.client()).unwrap().clone());
    let mut out = watch_delivery(&mut root, task_watch.unwrap(), c::Delivery::Task(Box::new(task)));
    while let Some(request) = out.pop() {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Run { .. }) },
                },
        } = request
        {
            return (root, task_watch.unwrap(), watch);
        }
    }
    panic!("a running chat opens its run watch");
}

#[test]
fn the_running_line_stands_while_the_reply_is_written_without_browser_dots() {
    const REPLY: &[u8] = b"The checks are running";
    let mut root = writing_root(REPLY);
    let chat = root.pages().chat().unwrap();
    let lines = chat.thread.lines();
    let Some(jig_client::thread::Line::Provisional { text, .. }) = lines.as_slice().last() else {
        panic!("the last line holds the provisional reply")
    };
    assert_eq!(*text, REPLY);
    let browser = view(&root, Shell::Browser);
    assert!(!styled(browser.tree(), look::TYPING), "reply words replace the browser's dots");
    act(&mut root, Act::Terminal(terminal::Act::Enter));
    let terminal = view(&root, Shell::Browser);
    let running = terminal
        .tree()
        .nodes()
        .iter()
        .find(|node| node.styles.as_slice().contains(&look::TERMINAL_RUN))
        .expect("the running line stands while reply words stream");
    assert_eq!(running.name.as_deref(), Some(b"Working".as_slice()));
    assert_eq!(crate::shared::running(&root, &f::browser_words()).as_deref(), Some(b"Working".as_slice()));
    assert!(!styled(terminal.tree(), look::TYPING));
}

#[test]
#[expect(clippy::too_many_lines, reason = "the offered commands, choice lists and notes are rows of one view table")]
fn the_commands_list_what_the_page_offers() {
    use terminal::{Act as T, Command as C};
    for (state, words, expected) in [
        (0, "/", "/model /effort /draft /pause /close /new /chats /theme /font /help /exit"),
        (3, "/", "/model /effort /draft /close /new /chats /theme /font /help /exit"),
        (4, "/", "/new /chats /theme /font /help /exit"),
        (0, "/mo", "/model"),
        (0, "/model\u{2003}now", ""),
        (0, "/nope", ""),
        (0, "/model now", ""),
        (0, "hello", ""),
    ] {
        let mut root = command_root(state);
        act(&mut root, Act::Terminal(T::Enter));
        act(&mut root, Act::MessageWords { words: words.as_bytes().into() });
        let mut view = view(&root, Shell::Browser);
        let menu = f::find(view.tree(), Role::Menu, b"Commands");
        assert_eq!(menu.is_some(), !expected.is_empty(), "{state} {words}");
        let items: Vec<_> = view.tree().nodes().iter().filter(|n| n.role == Role::MenuItem).cloned().collect();
        let names: Vec<_> = items.iter().map(|n| core::str::from_utf8(n.name.as_ref().unwrap()).unwrap()).collect();
        assert_eq!(names, expected.split_whitespace().collect::<Vec<_>>(), "{state} {words}");
        if let Some(menu) = menu {
            assert!(view.tree().find(menu).unwrap().states.as_slice().contains(&State::Options));
        }
        for item in items {
            let command = C::named(item.name.as_ref().unwrap()).unwrap();
            assert_eq!(
                item.description.as_deref(),
                Some(match command {
                    C::Model => b"Change the model, from the next turn".as_slice(),
                    C::Effort => b"Change the effort",
                    C::Draft => b"Show or hide the draft",
                    C::Pause => b"Pause this chat; a message resumes it",
                    C::Close => b"Close this chat; it stays in Chats",
                    C::StartOver => b"Start a new Ask temper chat",
                    C::Goal => b"Start a goal from the draft",
                    C::New => b"Start a new chat",
                    C::Chats => b"Open another chat",
                    C::Theme => b"Dark or light",
                    C::Font => b"DejaVu Sans Mono or IBM Plex Mono, to compare",
                    C::Help => b"Commands and keys",
                    C::Exit => b"Leave the terminal",
                })
            );
            assert_eq!(
                crate::act(&mut view, item.id, skein_ui::Act::Press),
                Some(Action::Pages(Act::Terminal(T::Run { command })))
            );
        }
        let escape = f::find(view.tree(), Role::Button, b"Close the list");
        assert_eq!(escape.is_some(), menu.is_some());
        if let Some(id) = escape {
            assert_eq!(view.tree().find(id).unwrap().shortcut, Some(skein_ui::Shortcut::ESCAPE));
            assert_eq!(crate::act(&mut view, id, skein_ui::Act::Press), Some(Action::Pages(Act::Terminal(T::Dismiss))));
        }
        if state == 4 {
            assert!(
                view.tree()
                    .nodes()
                    .iter()
                    .any(|n| n.role == Role::TextArea && n.placeholder.as_deref() == Some(b"/new starts another chat"))
            );
        }
    }
    for (state, expected) in [
        (ProposalState::Ordinary, "/draft /close /new /chats /theme /font /help /exit"),
        (ProposalState::NotProposed, "/draft /close /start-over /new /chats /theme /font /help /exit"),
    ] {
        let mut root = document_root(Some(b"# Draft\nBody"), Mode::Chat, &[], state);
        act(&mut root, Act::Terminal(T::Enter));
        act(&mut root, Act::MessageWords { words: b"/".as_slice().into() });
        let v = view(&root, Shell::Browser);
        let names: Vec<_> = v
            .tree()
            .nodes()
            .iter()
            .filter(|n| n.role == Role::MenuItem)
            .map(|n| core::str::from_utf8(n.name.as_ref().unwrap()).unwrap())
            .collect();
        assert_eq!(names, expected.split_whitespace().collect::<Vec<_>>());
    }
    for (command, head) in [
        (C::Model, "Model. A change applies from the next turn."),
        (C::Effort, "Effort. A change applies from the next turn."),
        (C::Chats, "Chats"),
        (C::Close, "Close this chat? It stays in Chats."),
        (C::StartOver, "Start over? A new Ask temper chat starts. This one stays in Chats."),
    ] {
        let mut root = if command == C::StartOver {
            document_root(Some(b"# Draft\nBody"), Mode::Chat, &[], ProposalState::NotProposed)
        } else {
            command_root(0)
        };
        act(&mut root, Act::Terminal(T::Enter));
        act(&mut root, Act::Terminal(T::Run { command }));
        let mut view = view(&root, Shell::Browser);
        let tree = view.tree();
        let menu = f::find(tree, Role::Menu, head.as_bytes()).unwrap();
        assert!(tree.find(menu).unwrap().states.as_slice().contains(&State::Options));
        assert_eq!(tree.find(menu).unwrap().styles.as_slice().contains(&look::TERMINAL_CHOICES), command != C::Chats);
        assert!(
            tree.nodes()
                .iter()
                .any(|n| n.styles.as_slice().contains(&look::TERMINAL_HEAD)
                    && n.name.as_deref() == Some(head.as_bytes()))
        );
        assert!(f::find(tree, Role::Button, b"Close the list").is_some());
        assert!(!tree.nodes().iter().any(|n| n.role == Role::Dialog));
        let selected = tree
            .nodes()
            .iter()
            .filter(|n| n.role == Role::MenuItem && n.states.as_slice().contains(&State::Selected))
            .count();
        if command == C::Model || command == C::Effort {
            assert_eq!(selected, 1, "{command:?}");
        }
        if command == C::Model || command == C::Effort {
            let (name, expected) = if command == C::Model {
                (b"chosen".as_slice(), d::pages::choice::Act::Model { index: 0, account: None })
            } else {
                (b"high".as_slice(), d::pages::choice::Act::Effort { index: 0 })
            };
            let id = f::find(view.tree(), Role::MenuItem, name).unwrap();
            assert_eq!(crate::act(&mut view, id, skein_ui::Act::Press), Some(Action::Pages(Act::Choice(expected))));
        }
        if command == C::Close || command == C::StartOver {
            let restarting = command == C::StartOver;
            for (name, expected) in [
                (if restarting { b"Start over".as_slice() } else { b"Close".as_slice() }, Act::Confirm),
                (if restarting { b"Keep this one".as_slice() } else { b"Keep it open".as_slice() }, Act::Cancel),
            ] {
                let id = f::find(view.tree(), Role::MenuItem, name).unwrap();
                assert_eq!(crate::act(&mut view, id, skein_ui::Act::Press), Some(Action::Pages(expected)));
            }
            act(&mut root, Act::Confirm);
            assert!(root.pages().confirmation().unwrap().sent.is_some());
            let waiting = self::view(&root, Shell::Browser);
            assert!(!waiting.tree().nodes().iter().any(|n| n.states.as_slice().contains(&State::Options)));
            assert!(!waiting.tree().nodes().iter().any(|n| n.role == Role::Dialog));
            assert!(f::find(waiting.tree(), Role::Button, b"Close the list").is_none());
        }
    }
    let (mut root, retained_watch) = command_root_with_watch(0);
    act(&mut root, Act::Terminal(T::Enter));
    let mut out = skein_lib::Queue::uncapped();
    d::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: f::domain_limits() },
        Event::Act { action: Action::Pages(Act::Terminal(T::Run { command: C::Chats })) },
        &mut out,
    );
    let mut watch = retained_watch;
    while let Some(r) = out.pop() {
        if let d::Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down:
                        jig_client::Down::Open {
                            watch: token,
                            subject: jig_client::Watchable::Jig(c::Subject::Tasks { query, .. }),
                        },
                },
        } = r
            && query.tracked.is_none()
            && query.requester.is_some()
        {
            watch = Some(token);
        }
    }
    let mut rows = skein_lib::List::with_capacity(4);
    for (task, activity) in [(3, 1), (f::chat::TASK, 9), (2, 5), (4, 2)] {
        let mut row = super::chats::line();
        row.chip.task = task;
        row.last_activity = skein_lib::Wall::from_nanos(activity);
        rows.push(row).unwrap();
    }
    f::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.expect("chats read asked"),
                    event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false }))),
                },
            },
        },
    );
    assert_eq!(root.pages().listing(), Some(terminal::Listing::Chats));
    assert_eq!(root.pages().chats().unwrap().live(root.client()).unwrap().rows.len(), 4);
    let mut v = view(&root, Shell::Browser);
    let items: Vec<_> = v.tree().nodes().iter().filter(|n| n.role == Role::MenuItem).cloned().collect();
    assert_eq!(items.iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(), [b"T2".as_slice(), b"T4", b"T3"]);
    for (item, task) in items.iter().zip([2, 4, 3]) {
        assert_eq!(item.description.as_deref(), Some(b"Hello".as_slice()));
        assert_eq!(
            crate::act(&mut v, item.id, skein_ui::Act::Press),
            Some(Action::Pages(Act::Go { address: Address::Chat { task } }))
        );
    }
    for address in
        [Address::Chat { task: f::chat::TASK }, Address::NewChat { draft: false }, Address::Ask { project: f::PROJECT }]
    {
        let (mut root, task_watch) =
            super::choice::provider_byline_root(address.clone(), [c::Readiness::Unavailable; 2], [false; 2], false);
        if let Some(watch) = task_watch {
            let source = command_root(0);
            let task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
            f::root_event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Watched {
                            watch,
                            event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                        },
                    },
                },
            );
        }
        act(&mut root, Act::Terminal(T::Enter));
        let writing = if let Address::Chat { .. } = address {
            Act::MessageWords { words: b"/nope".as_slice().into() }
        } else {
            Act::ChatWords { words: b"/nope".as_slice().into() }
        };
        act(&mut root, writing);
        let mut v = view(&root, Shell::Browser);
        let field = v
            .tree()
            .nodes()
            .iter()
            .find(|n| n.role == Role::TextArea)
            .unwrap_or_else(|| panic!("{address:?}: command box"));
        assert!(field.states.as_slice().contains(&State::SubmitsOnEnter));
        if address != (Address::Chat { task: f::chat::TASK }) {
            assert_eq!(field.focus, Some(d::pages::shown(root.pages())));
        }
        let id = field.id;
        let expected = if let Address::Chat { .. } = address { Act::Send } else { Act::StartChat };
        assert_eq!(crate::act(&mut v, id, skein_ui::Act::Submit), Some(Action::Pages(expected)));
    }
    let mut root = command_root(0);
    act(&mut root, Act::Terminal(T::Enter));
    let mut quiet = view(&root, Shell::Browser);
    assert!(f::find(quiet.tree(), Role::Disclosure, b"Model").is_none());
    let id = f::find(quiet.tree(), Role::Button, "chosen · high".as_bytes()).unwrap();
    assert_eq!(
        crate::act(&mut quiet, id, skein_ui::Act::Press),
        Some(Action::Pages(Act::Terminal(T::Run { command: C::Model })))
    );
    let mut root = command_root(0);
    act(&mut root, Act::Terminal(T::Enter));
    act(&mut root, Act::Terminal(T::Run { command: C::Help }));
    let v = view(&root, Shell::Browser);
    let help = v
        .tree()
        .nodes()
        .iter()
        .find(|n| n.role == Role::Code && n.styles.as_slice().contains(&look::TERMINAL_NOTE))
        .unwrap();
    let (log_at, log) = v.tree().nodes().iter().enumerate().find(|(_, n)| n.role == Role::Log).unwrap();
    let note_at = v.tree().nodes().iter().position(|n| n.id == help.id).unwrap();
    assert!(
        note_at > log_at && note_at < log_at.checked_add(usize::try_from(log.size).unwrap()).unwrap(),
        "notes scroll after the conversation inside its log"
    );
    let text = core::str::from_utf8(help.name.as_ref().unwrap()).unwrap();
    assert!(text.starts_with("/model       Change the model, from the next turn\n/effort      Change the effort"));
    assert!(text.ends_with("\n\nEnter sends\nShift+Enter makes a new line\nCtrl+\\ shows or hides the draft\nPgUp and PgDn read back\nEsc closes a list"));
}

fn subtree<'a>(tree: &'a Tree, role: Role, name: &[u8]) -> &'a [skein_ui::Node] {
    let at = tree.nodes().iter().position(|n| n.role == role && n.name.as_deref() == Some(name)).unwrap();
    &tree.nodes()[at..at.checked_add(usize::try_from(tree.nodes()[at].size).unwrap()).unwrap()]
}
fn press(view: &mut crate::View, role: Role, name: &[u8]) -> Act {
    let id = f::find(view.tree(), role, name).unwrap();
    let Some(Action::Pages(act)) = crate::act(view, id, skein_ui::Act::Press) else { panic!("page binding") };
    act
}
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one table holds the terminal's questions, cards and failed fold beside their browser drawings"
)]
fn the_terminal_draws_what_a_real_chat_has_on_a_line() {
    use skein_lib::{List, Wall};
    use temper_client_domain::pages::needs;
    enum Card {
        Question(bool),
        Proposal(bool),
        Delegation(bool),
        Failure,
    }
    for row in [
        Card::Question(true),
        Card::Question(false),
        Card::Proposal(false),
        Card::Proposal(true),
        Card::Delegation(false),
        Card::Delegation(true),
        Card::Failure,
    ] {
        match row {
            Card::Question(choices) => {
                let mut root = terminal_question_root(choices);
                let mut drawn = view(&root, Shell::Browser);
                assert!(!styled(drawn.tree(), look::TERMINAL_LINE));
                let head = if choices { b"Which parser?".as_slice() } else { b"What should the parser do?" };
                let id = f::find(drawn.tree(), Role::Text, head).unwrap();
                assert!(drawn.tree().find(id).unwrap().styles.as_slice().contains(&look::TERMINAL_HEAD));
                if choices {
                    let menu = subtree(drawn.tree(), Role::Menu, head);
                    assert!(menu[0].states.as_slice().contains(&State::Options));
                    let items: Vec<_> = menu.iter().filter(|n| n.role == Role::MenuItem).collect();
                    assert_eq!(
                        items.iter().map(|n| n.name.as_deref().unwrap()).collect::<Vec<_>>(),
                        [b"Keep it".as_slice(), b"Replace it", b"Later"]
                    );
                    assert!(!items[0].states.as_slice().contains(&State::Selected));
                    assert!(items[1].states.as_slice().contains(&State::Selected));
                    assert!(!items[2].states.as_slice().contains(&State::Selected));
                    assert_eq!(
                        press(&mut drawn, Role::MenuItem, b"Later"),
                        Act::Needs(needs::Act::Later { object: QUESTION_OBJECT })
                    );
                    let choose = press(&mut drawn, Role::MenuItem, b"Replace it");
                    assert_eq!(
                        choose,
                        Act::Needs(needs::Act::Choose {
                            object: QUESTION_OBJECT,
                            words: b"Replace it".as_slice().into()
                        })
                    );
                    act(&mut root, choose);
                    assert!(root.pages().needs().held(QUESTION_OBJECT));
                    let mut answered = view(&root, Shell::Browser);
                    assert!(skein_lib::bytes::find(&f::text(answered.tree()), b"Answered: Replace it").is_some());
                    let undo = press(&mut answered, Role::Button, b"Undo");
                    assert_eq!(undo, Act::Needs(needs::Act::Undo { object: QUESTION_OBJECT }));
                    act(&mut root, undo);
                    assert!(!root.pages().needs().held(QUESTION_OBJECT));
                    assert!(f::find(view(&root, Shell::Browser).tree(), Role::MenuItem, b"Replace it").is_some());
                    act(&mut root, Act::MessageWords { words: b"/".as_slice().into() });
                    assert!(f::find(view(&root, Shell::Browser).tree(), Role::Menu, b"Commands").is_some());
                    assert!(f::find(view(&root, Shell::Browser).tree(), Role::MenuItem, b"Later").is_none());
                } else {
                    assert_eq!(
                        press(&mut drawn, Role::Link, "Answer in Needs you ›".as_bytes()),
                        Act::Needs(needs::Act::Open { object: QUESTION_OBJECT })
                    );
                    assert!(f::find(drawn.tree(), Role::MenuItem, b"Later").is_none());
                }
                act(&mut root, Act::Terminal(terminal::Act::Leave));
                assert!(root.pages().inbox().is_none());
                assert!(f::find(view(&root, Shell::Browser).tree(), Role::Text, head).is_none());
            }
            Card::Proposal(settings) => {
                let mut page = super::cards::batch_proposal();
                if settings {
                    let c::ReadValue::Proposal(proposal) = &mut page else { panic!("proposal") };
                    proposal.action = c::ProposalAction::Settings(Box::new(c::ProposalActionSettings {
                        against: 0,
                        items: List::with_capacity(0),
                        review: None,
                        made: None,
                    }));
                }
                let mut root = f::chat::proposal_in_layout(None, Some(page), settings, false);
                let browser = view(&root, Shell::Browser);
                let browser_text = f::text(browser.tree());
                assert!(f::find(browser.tree(), Role::Disclosure, b"Details").is_none());
                act(&mut root, Act::Terminal(terminal::Act::Enter));
                let mut drawn = view(&root, Shell::Browser);
                let facts = subtree(drawn.tree(), Role::Disclosure, b"Details");
                assert!(!facts[0].states.as_slice().contains(&State::Expanded));
                assert!(skein_lib::bytes::find(&f::nodes_text(facts), b"Make a goal").is_some());
                if !settings {
                    assert!(facts.iter().any(|n| n.role == Role::Term && n.name.as_deref() == Some(b"Budget")));
                }
                assert!(styled(drawn.tree(), look::TERMINAL_CARD_LINE));
                assert!(!styled(drawn.tree(), look::CARD));
                assert!(f::find(drawn.tree(), Role::Text, b"Proposal").is_some());
                let accept = press(&mut drawn, Role::Button, b"Accept");
                assert_eq!(
                    accept,
                    Act::Decide {
                        object: c::ObjectKey::Proposal(f::chat::PROPOSAL),
                        revision: None,
                        act: c::ActKind::Accept
                    }
                );
                act(&mut root, accept);
                let mut confirm = view(&root, Shell::Browser);
                assert!(f::find(confirm.tree(), Role::Dialog, b"Accept this proposal?").is_none());
                assert!(f::find(confirm.tree(), Role::Menu, b"Accept this proposal?").is_some());
                assert_eq!(press(&mut confirm, Role::MenuItem, b"Accept"), Act::Confirm);
                let cancel = press(&mut confirm, Role::MenuItem, b"Not yet");
                assert_eq!(cancel, Act::Cancel);
                act(&mut root, cancel);
                for dismiss in
                    [Act::Terminal(terminal::Act::Dismiss), Act::MessageWords { words: b"Actually".as_slice().into() }]
                {
                    let mut drawn = view(&root, Shell::Browser);
                    let accept = press(&mut drawn, Role::Button, b"Accept");
                    act(&mut root, accept);
                    act(&mut root, dismiss);
                    assert!(root.pages().confirmation().is_none());
                }
                act(&mut root, Act::Terminal(terminal::Act::Leave));
                assert_eq!(f::text(view(&root, Shell::Browser).tree()), browser_text);
                act(&mut root, Act::Terminal(terminal::Act::Enter));
                let mut drawn = view(&root, Shell::Browser);
                let accept = press(&mut drawn, Role::Button, b"Accept");
                act(&mut root, accept);
                act(&mut root, Act::Confirm);
                assert!(root.pages().confirmation().unwrap().sent.is_some());
                let sent = view(&root, Shell::Browser);
                assert!(!sent.tree().nodes().iter().any(|n| n.states.as_slice().contains(&State::Options)));
                assert!(!sent.tree().nodes().iter().any(|n| n.role == Role::Dialog));
            }
            Card::Delegation(landed) => {
                let mut task = f::task::document();
                task.chip.task = f::chat::DELEGATE;
                task.chip.title = c::DocumentText::complete(f::first_run::text(b"Fix parser"));
                task.result_contract = c::TaskContractDocument::Change { connector: f::FORGE, kind: 1, words: 256 };
                task.result = if landed {
                    Some(c::TaskResult::Ending(Box::new(c::ChipEnding::Done(c::ResultValue::Change {
                        kind: 1,
                        part: c::Part {
                            connector: f::FORGE,
                            kind: c::PartKind::Result,
                            key: c::PartKey::Number(f::chat::DELEGATE),
                        },
                        words: c::DocumentText::complete(f::first_run::text(b"1a2b3c4abcdef")),
                    }))))
                } else {
                    None
                };
                let mut b = skein_ui::Builder::new(&f::tree_limits());
                let mut bindings = crate::frame::Bindings::new(32);
                crate::cards::delegation::terminal_card(&mut b, &task, &f::browser_words(), &mut bindings);
                let tree = b.finish();
                assert!(f::find(&tree, Role::Paragraph, "T12 · CHANGE".as_bytes()).is_some());
                assert!(f::find(&tree, Role::Heading { level: 3 }, b"Fix parser").is_some());
                assert!(f::find(&tree, Role::Status, if landed { b"Landed" } else { b"Running" }).is_some());
                assert!(!styled(&tree, look::STEPS));
                assert!(!styled(&tree, look::DETAILS));
                let link = tree
                    .nodes()
                    .iter()
                    .find(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Task 12"))
                    .unwrap();
                assert_eq!(
                    bindings.decode(link.binding.unwrap().token, skein_ui::Act::Press),
                    Some(Action::Pages(Act::Go { address: Address::Task { task: 12 } }))
                );
                assert!(skein_lib::bytes::find(&f::text(&tree), b"Open").is_some());
                let mut b = skein_ui::Builder::new(&f::tree_limits());
                crate::cards::delegation::card(
                    &mut b,
                    &task,
                    &f::browser_words(),
                    &mut crate::frame::Bindings::new(32),
                );
                let browser = b.finish();
                assert!(styled(&browser, look::STEPS));
                assert!(!styled(&browser, look::TERMINAL_CARD));
                assert_eq!(styled(&browser, look::DETAILS), landed);
            }
            Card::Failure => {
                let c::Block::Call(mut call) =
                    f::first_run::call(1, b"shell", br#"{"command":"check"}"#, Some(b"parser rejected\nmore detail"))
                else {
                    panic!("call")
                };
                call.failed = true;
                for (terminal, running, expected) in [
                    (false, false, Some(b"1 failed".as_slice())),
                    (false, true, None),
                    (true, false, Some(b"failed: parser rejected".as_slice())),
                    (true, true, Some(b"failed: parser rejected".as_slice())),
                ] {
                    let mut b = skein_ui::Builder::new(&f::tree_limits());
                    let mut run = crate::pages::chat_line::Run::with_capacity(1);
                    run.call(&call, Wall::EPOCH, None);
                    run.flush(&mut b, None, running, &f::browser_words(), terminal);
                    let tree = b.finish();
                    let fold = tree.nodes().iter().find(|n| n.role == Role::Disclosure).unwrap();
                    assert_eq!(fold.description.as_deref(), expected);
                    assert!(!fold.states.as_slice().contains(&State::Expanded));
                }
            }
        }
    }
}
