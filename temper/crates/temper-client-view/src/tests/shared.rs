//! Shared words match the browser's trees (temper/docs/design/client/tui/pages/README.md).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "ordinary tests inspect semantic trees and bindings"
)]
use super::fixtures as f;
use crate::{look, shared, words::Words};
use jig_client::contract as c;
use skein_lib::{List, Queue, Wall};
use skein_ui::{Builder, Role, Tree};
use temper_client_domain::{
    self as d, Address, Event,
    pages::{
        Act,
        terminal::{self, Command, Note},
    },
};

fn act(root: &mut d::Domain, act: Act) {
    f::root_event(root, Event::Act { action: d::Action::Pages(act) });
}
fn terminal_root(running: bool) -> d::Domain {
    let mut task = super::chat::asking_task();
    task.chip.details.phase = c::ChipPhase::Active(if running { c::RunStage::Due } else { c::RunStage::Idle });
    task.chip.phase = task.chip.details.phase.compact();
    task.waits_on = if running { c::WaitsOn::Nothing } else { c::WaitsOn::News };
    task.chip.details.details.live_delegates = 0;
    task.requester = c::Requester::Person(f::chat::PARTY);
    task.wake.words = c::WakeRule::Immediate;
    task.delegates.clear();
    task.run_wait = None;
    let mut root = super::chat::asking_task_root(task, true, 1);
    act(&mut root, Act::Terminal(terminal::Act::Enter));
    root
}
#[derive(Debug, PartialEq, Eq)]
enum Value {
    Words(Option<Box<[u8]>>),
    Tick(Option<Wall>),
    Flag(bool),
    Kind(shared::NeedKind),
    Count(u64),
    Choices(Box<[shared::Choice]>),
    Confirmation(Option<shared::Confirmation>),
    Question(Option<shared::Question>),
    Delegation(shared::DelegationLine),
}
#[derive(Debug, PartialEq, Eq)]
struct Row {
    name: &'static str,
    shared: Value,
    browser: Value,
}
fn words(name: &'static str, shared: Box<[u8]>, browser: Option<Box<[u8]>>) -> Row {
    Row { name, shared: Value::Words(Some(shared)), browser: Value::Words(browser) }
}
fn named(tree: &Tree, role: Role, name: &[u8]) -> Option<Box<[u8]>> {
    f::find(tree, role, name).and_then(|id| tree.find(id)).and_then(|node| node.name.clone())
}
fn quiet(w: &Words) -> [Row; 3] {
    let root = terminal_root(false);
    let view = f::browser_page_view(&root);
    let said = shared::say(w, shared::TerminalWord::Commands);
    let closed = shared::closed_line(&root, root.pages().chat().unwrap().task(root.client()), w);
    [
        words("commands hint", said.clone(), named(view.tree(), Role::Paragraph, b"/ for commands")),
        words("commands words", said, Some(b"/ for commands".as_slice().into())),
        words("closed line", closed.clone(), named(view.tree(), Role::Button, &closed)),
    ]
}
fn running(w: &Words, active: bool) -> [Row; 2] {
    let root = terminal_root(active);
    let view = f::browser_page_view(&root);
    let node = view.tree().nodes().iter().find(|node| node.styles.as_slice().contains(&look::TERMINAL_RUN));
    [
        Row {
            name: "running words",
            shared: Value::Words(shared::running(&root, w)),
            browser: Value::Words(node.and_then(|node| node.name.clone())),
        },
        Row {
            name: "running offered",
            shared: Value::Flag(shared::running(&root, w).is_some()),
            browser: Value::Flag(active),
        },
    ]
}
fn commands(rows: &mut List<Row>, new: bool) {
    let mut root = terminal_root(false);
    if new {
        f::root_event(&mut root, Event::Went { place: d::Place::Address(Address::NewChat { draft: false }) });
    }
    act(
        &mut root,
        if new {
            Act::ChatWords { words: b"/".as_slice().into() }
        } else {
            Act::MessageWords { words: b"/".as_slice().into() }
        },
    );
    let view = f::browser_page_view(&root);
    let task = root.pages().chat().and_then(|chat| chat.task(root.client()));
    for command in Command::ALL {
        let node = view
            .tree()
            .nodes()
            .iter()
            .find(|node| node.role == Role::MenuItem && node.name.as_deref() == Some(command.name()));
        let offered = shared::offered(&root, task, command);
        rows.push(Row { name: "command offered", shared: Value::Flag(offered), browser: Value::Flag(node.is_some()) })
            .unwrap();
        rows.push(Row {
            name: "command description",
            shared: Value::Words(if offered { Some(shared::description(command).into()) } else { None }),
            browser: Value::Words(node.and_then(|node| node.description.clone())),
        })
        .unwrap();
    }
}
fn note(note: Note) -> Row {
    let mut root = terminal_root(false);
    match &note {
        Note::Unknown { words } => {
            act(&mut root, Act::MessageWords { words: words.clone() });
            act(&mut root, Act::Send);
        }
        Note::Closed => {
            let mut task = super::chat::asking_task();
            task.chip.details.phase = c::ChipPhase::Ended;
            task.chip.phase = c::Phase::Ended;
            task.acts.clear();
            root = super::chat::asking_task_root(task, true, 1);
            act(&mut root, Act::Terminal(terminal::Act::Enter));
            act(&mut root, Act::Send);
        }
        Note::Help => act(&mut root, Act::Terminal(terminal::Act::Run { command: Command::Help })),
        Note::Font { .. } => act(&mut root, Act::Terminal(terminal::Act::Font)),
    }
    let view = f::browser_page_view(&root);
    let text = shared::note(&note);
    let role = if note == Note::Help { Role::Code } else { Role::Paragraph };
    words("note", text.clone(), named(view.tree(), role, &text))
}
fn home(w: &Words) -> [Row; 4] {
    let page = f::ReadPage::new(Address::Home { project: f::PROJECT });
    let facts = d::pages::home::facts(&page.root, f::PROJECT, f::NOW);
    let (headline, next) = shared::home_headline(w, b"Shop", &facts, f::NOW);
    let view = f::browser_page_view(&page.root);
    let first = words("home headline", headline.clone(), named(view.tree(), Role::Heading { level: 1 }, &headline));
    let tick = Row {
        name: "home minute tick",
        shared: Value::Tick(next),
        browser: Value::Tick(Some(Wall::from_nanos(60_000_000_000))),
    };
    let mut view = f::view_of(f::browser_words());
    let at = next.unwrap_or(f::NOW);
    crate::build(&mut view, &page.root, at, &mut Queue::uncapped());
    let (headline, _) = shared::home_headline(w, b"Shop", &facts, at);
    [
        first,
        tick,
        words("home after tick", headline.clone(), named(view.tree(), Role::Heading { level: 1 }, &headline)),
        Row { name: "home without a brief", shared: Value::Flag(facts.brief().is_some()), browser: Value::Flag(false) },
    ]
}
fn state(w: &Words) -> Row {
    let mut page = f::ReadPage::new(Address::Home { project: f::PROJECT });
    let line = super::chats::line();
    page.live_chats(core::slice::from_ref(&line));
    let me = page.root.pages().me(page.root.client()).unwrap();
    let state = shared::chat_state(&line, me, w);
    let view = f::browser_page_view(&page.root);
    let browser = view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.styles.as_slice().contains(&look::HOME_WHEN))
        .and_then(|node| node.text.last())
        .map(|run| {
            let end = skein_lib::bytes::find(&run.text, " · ".as_bytes()).unwrap_or(run.text.len());
            run.text.get(..end).unwrap().into()
        });
    words("home chat state", state, browser)
}
fn need(w: &Words) -> [Row; 2] {
    let question = c::Question {
        task: f::task::document().chip,
        from: None,
        message: f::INBOX_QUESTION,
        words: c::DocumentText::complete(f::first_run::text(b"Which path?")),
        since: Wall::EPOCH,
        acts: List::with_capacity(0),
        ending: None,
    };
    let root = f::reply_composer((
        c::Read::Question { task: f::INBOX_TASK, message: f::INBOX_QUESTION },
        c::ReadValue::Question(Box::new(question)),
    ));
    let card = root.pages().inbox().unwrap().cards.iter().find(|card| card.page.is_some()).unwrap();
    let line = shared::need_line(&d::pages::needs::Need::Card(card), w);
    let view = f::browser_page_view(&root);
    let nodes = view.tree().nodes();
    let from = nodes.iter().position(|node| node.styles.as_slice().contains(&look::NEED_FROM)).unwrap();
    let end = from.checked_add(usize::try_from(nodes[from].size).unwrap()).unwrap();
    let title = nodes[from..end].iter().find(|node| node.role == Role::Link).and_then(|node| node.name.as_deref());
    let question = nodes
        .iter()
        .find(|node| node.styles.as_slice().contains(&look::ASK_QUESTION))
        .and_then(|node| node.name.as_deref());
    let browser = title.zip(question).map(|(title, question)| skein_lib::bytes::concat(&[title, b": ", question]));
    [
        words("home need parts", line.words, browser),
        Row { name: "need kind", shared: Value::Kind(line.kind), browser: Value::Kind(shared::NeedKind::Question) },
    ]
}
fn heading(w: &Words) -> Row {
    let root = terminal_root(false);
    let task = root.pages().chat().unwrap().task(root.client()).unwrap();
    let heading = shared::heading(task.chip.title.bytes());
    let expected = w.say(
        crate::words::Word::ChatTitle,
        &[skein_ui::phrases::Hole::Number(task.chip.task), skein_ui::phrases::Hole::Name(heading)],
    );
    let view = f::browser_page_view(&root);
    let browser = view
        .tree()
        .nodes()
        .iter()
        .find(|node| node.role == Role::Heading { level: 1 })
        .and_then(|node| node.name.clone());
    words("chat heading", expected, browser)
}
fn run(w: &Words) -> [Row; 6] {
    let c::Block::Call(call) = f::first_run::call(1, b"read", b"{}", None) else { unreachable!() };
    let mut run = shared::Run::with_capacity(1);
    run.call(&call, Wall::EPOCH, None);
    let has_call = run.has_call(1);
    let line = run.line(None, false, w, false).unwrap();
    let empty = run.is_empty() && run.line(None, false, w, false).is_none();
    let mut run = shared::Run::with_capacity(1);
    run.call(&call, Wall::EPOCH, None);
    let mut b = Builder::new(&f::tree_limits());
    run.flush(&mut b, None, false, w, false);
    let tree = b.finish();
    let node = tree.nodes().iter().find(|node| node.role == Role::Disclosure).unwrap();
    [
        words("run name", line.name, node.name.clone()),
        Row {
            name: "run description",
            shared: Value::Words(line.description),
            browser: Value::Words(node.description.clone()),
        },
        Row { name: "run failed", shared: Value::Count(line.failed), browser: Value::Count(0) },
        Row { name: "run running", shared: Value::Flag(line.running), browser: Value::Flag(false) },
        Row { name: "run has call", shared: Value::Flag(has_call), browser: Value::Flag(true) },
        Row { name: "run empties", shared: Value::Flag(empty), browser: Value::Flag(true) },
    ]
}
#[expect(clippy::wildcard_enum_match_arm, reason = "the fixture finds the run watch among public requests")]
fn working(rows: &mut List<Row>, w: &Words) {
    for (name, active, writing) in [
        ("working before words", true, false),
        ("working ends when words begin", true, true),
        ("idle is not working", false, false),
    ] {
        let mut root = terminal_root(active);
        if writing {
            let mut task = super::chat::asking_task();
            task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Running { attempt: f::chat::ATTEMPT });
            task.chip.phase = c::Phase::Active;
            task.waits_on = c::WaitsOn::Nothing;
            task.result = None;
            root = super::chat::asking_task_root(task, true, 1);
            let handle = root.pages().chat().unwrap().run.unwrap();
            f::root_event(&mut root, Event::Client { event: jig_client::Event::Unwatch { handle } });
            let mut out = Queue::uncapped();
            d::step(
                &mut root,
                &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
                Event::Client {
                    event: jig_client::Event::Watch {
                        handle,
                        subject: jig_client::Watchable::Jig(c::Subject::Run {
                            task: f::chat::TASK,
                            turn: None,
                            offset: 0,
                        }),
                    },
                },
                &mut out,
            );
            let watch = out
                .iter()
                .find_map(|request| match request {
                    d::Request::Unlinked {
                        request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, .. } },
                    } => Some(*watch),
                    _ => None,
                })
                .unwrap();
            let text = b"Hello".to_vec();
            f::root_event(
                &mut root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: jig_client::Up::Watched {
                            watch,
                            event: c::WatchEvent::Delivered(c::Delivery::RunState(Box::new(c::RunState {
                                task: f::chat::TASK,
                                attempt: Some(f::chat::ATTEMPT),
                                committed_turn: 0,
                                writing: Some(c::RunText {
                                    writing: None,
                                    attempt: f::chat::ATTEMPT,
                                    began: None,
                                    turn: c::ProvisionalTurnName { attempt: f::chat::ATTEMPT, number: 1, retry: 0 },
                                    from: 0,
                                    through: u32::try_from(text.len()).unwrap(),
                                    text: c::Text::new(text.into_boxed_slice(), f::chat::TEXT).unwrap(),
                                    cut: false,
                                    call: None,
                                }),
                                call: None,
                            }))),
                        },
                    },
                },
            );
        }
        let view = f::browser_page_view(&root);
        let browser = view.tree().nodes().iter().any(|node| {
            node.styles.as_slice().contains(&look::TYPING) || node.styles.as_slice().contains(&look::WORK_RUNNING)
        });
        rows.push(Row { name, shared: Value::Flag(shared::working(&root, w)), browser: Value::Flag(browser) }).unwrap();
    }
}

#[test]
fn shared_words_are_the_browsers() {
    let w = f::browser_words();
    let mut rows = List::uncapped();
    for row in quiet(&w)
        .into_iter()
        .chain(running(&w, false))
        .chain(running(&w, true))
        .chain([note(Note::Unknown { words: b"/nope".as_slice().into() }), note(Note::Closed)])
        .chain(home(&w))
        .chain([state(&w)])
        .chain(need(&w))
        .chain([heading(&w)])
        .chain(run(&w))
    {
        rows.push(row).unwrap();
    }
    working(&mut rows, &w);
    lists(&mut rows, &w);
    confirmations(&mut rows);
    questions(&mut rows, &w);
    card_lines(&mut rows, &w);
    refusals(&mut rows, &w);
    failure_words(&mut rows, &w);
    commands(&mut rows, false);
    commands(&mut rows, true);
    let mut failed = List::uncapped();
    for row in &rows {
        if row.shared != row.browser {
            failed.push(row).unwrap();
        }
    }
    let no_failures: &[&Row] = &[];
    assert_eq!(failed.as_slice(), no_failures, "every shared row");
}

fn browser_choices(view: &mut crate::View) -> Box<[shared::Choice]> {
    let mut choices = List::uncapped();
    let nodes = view.tree().nodes().to_vec();
    for node in &nodes {
        if node.role != Role::MenuItem {
            continue;
        }
        let Some(d::Action::Pages(act)) = crate::act(view, node.id, skein_ui::Act::Press) else { continue };
        if let Act::Go { address: Address::ConnectModel { .. } | Address::Accounts } = act {
            continue;
        }
        choices
            .push(shared::Choice {
                name: node.name.clone().unwrap(),
                words: node.description.clone().unwrap_or_default(),
                selected: node.states.as_slice().contains(&skein_ui::State::Selected),
                act,
            })
            .unwrap();
    }
    choices.into_boxed()
}
fn list_row(rows: &mut List<Row>, root: &d::Domain, choices: Box<[shared::Choice]>, listing: terminal::Listing) {
    let mut view = super::terminal::view(root, crate::words::Shell::Browser);
    let head = view
        .tree()
        .nodes()
        .iter()
        .find(|n| n.styles.as_slice().contains(&look::TERMINAL_HEAD))
        .and_then(|n| n.name.clone());
    rows.push(words("list head", shared::list_head(listing).into(), head)).unwrap();
    rows.push(Row {
        name: "list options, descriptions, selection and acts",
        shared: Value::Choices(choices),
        browser: Value::Choices(browser_choices(&mut view)),
    })
    .unwrap();
}
fn lists(rows: &mut List<Row>, w: &Words) {
    for two in [false, true] {
        let (mut root, _, _) = super::choice::picker_fixture(two, false, true);
        act(&mut root, Act::Terminal(terminal::Act::Enter));
        act(&mut root, Act::Terminal(terminal::Act::Run { command: Command::Model }));
        let choices = shared::models(&root, None, w);
        rows.push(Row {
            name: "models leave out hidden models and provider links",
            shared: Value::Count(u64::try_from(choices.len()).unwrap()),
            browser: Value::Count(if two { 2 } else { 1 }),
        })
        .unwrap();
        let view = super::terminal::view(&root, crate::words::Shell::Browser);
        rows.push(Row {
            name: "browser keeps provider connection row",
            shared: Value::Flag(f::find(view.tree(), Role::MenuItem, b"OpenAI").is_some()),
            browser: Value::Flag(true),
        })
        .unwrap();
        list_row(rows, &root, choices, terminal::Listing::Model);
    }
    let mut root = super::terminal::command_root(0);
    act(&mut root, Act::Terminal(terminal::Act::Enter));
    act(&mut root, Act::Terminal(terminal::Act::Run { command: Command::Effort }));
    let task = root.pages().chat().unwrap().task(root.client());
    list_row(rows, &root, shared::efforts(&root, task, w), terminal::Listing::Effort);
    let (mut root, watch) = super::terminal::command_root_with_watch(0);
    act(&mut root, Act::Terminal(terminal::Act::Enter));
    act(&mut root, Act::Terminal(terminal::Act::Run { command: Command::Chats }));
    let mut chats = List::with_capacity(4);
    for (task, activity) in [(3, 1), (f::chat::TASK, 9), (2, 5), (4, 2)] {
        let mut line = super::chats::line();
        line.chip.task = task;
        line.last_activity = Wall::from_nanos(activity);
        chats.push(line).unwrap();
    }
    f::root_event(
        &mut root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                        rows: chats,
                        more: false,
                    }))),
                },
            },
        },
    );
    let choices = shared::chats(&root, w);
    let names = crate::accounts::words::join(&choices.iter().map(|c| c.name.as_ref()).collect::<Vec<_>>());
    rows.push(words("chats newest first without the open chat", names, Some(b"T2T4T3".as_slice().into()))).unwrap();
    list_row(rows, &root, choices, terminal::Listing::Chats);
}
fn confirmations(rows: &mut List<Row>) {
    for restarting in [false, true] {
        let mut root = if restarting {
            super::document::document_root(
                Some(b"# Draft\nBody"),
                d::pages::chat::document::Mode::Chat,
                &[],
                super::document::ProposalState::NotProposed,
            )
        } else {
            super::terminal::command_root(0)
        };
        act(&mut root, Act::Terminal(terminal::Act::Enter));
        act(
            &mut root,
            Act::Terminal(terminal::Act::Run { command: if restarting { Command::StartOver } else { Command::Close } }),
        );
        let mut view = super::terminal::view(&root, crate::words::Shell::Browser);
        let (head, take, keep): (&'static [u8], &'static [u8], &'static [u8]) = if restarting {
            (b"Start over? A new Ask temper chat starts. This one stays in Chats.", b"Start over", b"Keep this one")
        } else {
            (b"Close this chat? It stays in Chats.", b"Close", b"Keep it open")
        };
        let browser = if f::find(view.tree(), Role::Menu, head).is_some() {
            Some(shared::Confirmation { head, take, keep })
        } else {
            None
        };
        rows.push(Row {
            name: "confirmation head and words",
            shared: Value::Confirmation(shared::confirmation(&root)),
            browser: Value::Confirmation(browser),
        })
        .unwrap();
        let choices: Box<[shared::Choice]> = match shared::confirmation(&root) {
            Some(c) => Box::new([
                shared::Choice { name: c.take.into(), words: Box::default(), selected: false, act: Act::Confirm },
                shared::Choice { name: c.keep.into(), words: Box::default(), selected: false, act: Act::Cancel },
            ]),
            None => Box::default(),
        };
        rows.push(Row {
            name: "confirmation acts",
            shared: Value::Choices(choices),
            browser: Value::Choices(browser_choices(&mut view)),
        })
        .unwrap();
        act(&mut root, Act::Confirm);
        rows.push(Row {
            name: "sent confirmation disappears",
            shared: Value::Confirmation(shared::confirmation(&root)),
            browser: Value::Confirmation(None),
        })
        .unwrap();
    }
}
fn questions(rows: &mut List<Row>, w: &Words) {
    for choices in [false, true] {
        let mut root = super::terminal::terminal_question_root(choices);
        let view = super::terminal::view(&root, crate::words::Shell::Browser);
        let head = view
            .tree()
            .nodes()
            .iter()
            .find(|n| n.styles.as_slice().contains(&look::TERMINAL_HEAD))
            .unwrap()
            .name
            .clone()
            .unwrap();
        let mut parsed = List::uncapped();
        let mut primary = None;
        for node in
            view.tree().nodes().iter().filter(|n| n.role == Role::MenuItem && n.name.as_deref() != Some(b"Later"))
        {
            if node.states.as_slice().contains(&skein_ui::State::Selected) {
                primary = Some(usize::try_from(parsed.len()).unwrap());
            }
            parsed
                .push(shared::QuestionChoice {
                    words: node.name.clone().unwrap(),
                    recommended: node.description.is_some(),
                })
                .unwrap();
        }
        let object = root.pages().inbox().unwrap().cards.as_slice().first().unwrap().object;
        rows.push(Row {
            name: "question head, choices, recommendation and primary",
            shared: Value::Question(shared::question(&root, w)),
            browser: Value::Question(Some(shared::Question { object, head, choices: parsed.into_boxed(), primary })),
        })
        .unwrap();
        // The data reads the card whatever look its consumer uses.
        let mut look = root.pages().look();
        look.terminal = false;
        f::root_event(&mut root, Event::LookKept { look });
        rows.push(Row {
            name: "question does not depend on look",
            shared: Value::Flag(shared::question(&root, w).is_some()),
            browser: Value::Flag(true),
        })
        .unwrap();
    }
    let root = super::terminal::command_root(0);
    rows.push(Row {
        name: "no open question",
        shared: Value::Question(shared::question(&root, w)),
        browser: Value::Question(None),
    })
    .unwrap();
}
fn card_lines(rows: &mut List<Row>, w: &Words) {
    let task = super::delegation::change();
    let mut b = Builder::new(&f::tree_limits());
    let mut bindings = crate::frame::Bindings::new(32);
    crate::cards::delegation::terminal_card(&mut b, &task, w, &mut bindings);
    let tree = b.finish();
    let name = |role| tree.nodes().iter().find(|n| n.role == role).unwrap().name.clone().unwrap();
    rows.push(Row {
        name: "delegation kind, title and state",
        shared: Value::Delegation(shared::delegation_line(&task, w)),
        browser: Value::Delegation(shared::DelegationLine {
            kind: name(Role::Paragraph),
            title: name(Role::Heading { level: 3 }),
            state: name(Role::Status),
        }),
    })
    .unwrap();
    let mut root = f::chat::proposal_in_layout(None, Some(super::cards::batch_proposal()), false, false);
    act(&mut root, Act::Terminal(terminal::Act::Enter));
    let view = super::terminal::view(&root, crate::words::Shell::Browser);
    let card = root.pages().cards().unwrap().as_slice().first().unwrap();
    let title = view.tree().nodes().iter().find(|n| n.role == Role::Heading { level: 3 }).and_then(|n| n.name.clone());
    rows.push(Row {
        name: "proposal title",
        shared: Value::Words(shared::proposal_line(card, w)),
        browser: Value::Words(title),
    })
    .unwrap();
    let root = super::terminal::terminal_question_root(true);
    let card = root.pages().inbox().unwrap().cards.as_slice().first().unwrap();
    rows.push(Row {
        name: "question has no proposal line",
        shared: Value::Words(shared::proposal_line(card, w)),
        browser: Value::Words(None),
    })
    .unwrap();
}
fn answer_send(root: &mut d::Domain, send: Act, answer: c::Answer) {
    let mut out = Queue::uncapped();
    for event in [Event::Act { action: d::Action::Pages(send) }, Event::Client { event: jig_client::Event::Stored }] {
        d::step(
            root,
            &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
            event,
            &mut out,
        );
    }
    let env = skein_lib::Env {
        now: skein_lib::Time::ZERO.saturating_add(d::pages::needs::HOLD),
        wall: f::NOW,
        limits: f::domain_limits(),
    };
    d::fire(root, &env, &mut out);
    d::step(root, &env, Event::Client { event: jig_client::Event::Stored }, &mut out);
    let attempt = out
        .iter()
        .find_map(|r| {
            if let d::Request::Unlinked {
                request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { attempt, .. } },
            } = r
            {
                Some(*attempt)
            } else {
                None
            }
        })
        .unwrap();
    f::root_event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Jig(answer)),
                },
            },
        },
    );
}
fn refusal_row(rows: &mut List<Row>, root: &d::Domain, w: &Words, name: &'static str) {
    let view = f::browser_page_view(root);
    rows.push(Row {
        name,
        shared: Value::Words(shared::refusal(root, w)),
        browser: Value::Words(named(view.tree(), Role::Status, b"This act is not offered")),
    })
    .unwrap();
}
fn refusals(rows: &mut List<Row>, w: &Words) {
    let refused = c::Answer::Refused { refusal: c::Refusal::NotOffered, at: None };
    let mut root = f::playground_root(Address::Ask { project: f::PROJECT }, f::PROJECT, true);
    act(&mut root, Act::ChatWords { words: b"My way of working".as_slice().into() });
    answer_send(&mut root, Act::StartChat, refused.clone());
    refusal_row(rows, &root, w, "refused start on Ask");
    let root = super::chat::answered_message(refused.clone());
    refusal_row(rows, &root, w, "refused message");
    let root = super::chat::answered_message(c::Answer::Done(c::Outcome::Document(c::DocumentOutcome::Said {
        task: f::chat::TASK,
        message: 1,
    })));
    refusal_row(rows, &root, w, "accepted message has no refusal");
    let mut root = super::terminal::terminal_question_root(true);
    let object = root.pages().inbox().unwrap().cards.as_slice().first().unwrap().object;
    answer_send(
        &mut root,
        Act::Needs(d::pages::needs::Act::Choose { object, words: b"Replace it".as_slice().into() }),
        refused,
    );
    refusal_row(rows, &root, w, "refused waiting answer");
}

fn feedback_words(line: &shared::Feedback) -> Box<[u8]> {
    match &line.after {
        Some(after) => crate::accounts::words::join(&[&line.head, b" ", after]),
        None => line.head.clone(),
    }
}
fn failure_words(rows: &mut List<Row>, w: &Words) {
    use jig_client::{Answered, Outcome};
    for outcome in [
        Outcome::Unknown,
        Outcome::Answered(Answered::Jig(c::Answer::Refused { refusal: c::Refusal::Role { held: None }, at: None })),
        Outcome::Answered(Answered::Jig(c::Answer::Refused {
            refusal: c::Refusal::Moved { now: c::Revision { value: 1 }, what: c::Moved::Waiting },
            at: None,
        })),
    ] {
        let mut b = Builder::new(&f::tree_limits());
        crate::pages::chats::outcome_text(&mut b, w, &outcome, 0);
        let tree = b.finish();
        let parts = tree
            .nodes()
            .iter()
            .filter(|n| n.role == Role::Status || n.role == Role::Paragraph)
            .map(|n| n.name.as_deref().unwrap())
            .collect::<Vec<_>>();
        let browser = parts.join(b" ".as_slice());
        let line = shared::outcome_feedback(&outcome, w, false).unwrap();
        rows.push(words("send failure and explanation", feedback_words(&line), Some(browser.into_boxed_slice())))
            .unwrap();
        if let Outcome::Answered(Answered::Jig(c::Answer::Refused { refusal: c::Refusal::Moved { .. }, .. })) = outcome
        {
            let answer = shared::outcome_feedback(&outcome, w, true).unwrap();
            rows.push(words(
                "moved answer asks to look again",
                feedback_words(&answer),
                Some(crate::accounts::words::join(&[&line.head, b" ", &w.say_card(crate::cards::P::LookAgain, &[])])),
            ))
            .unwrap();
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum NeedAction {
    Answer(shared::Question),
    Review { words: Box<[u8]>, address: Box<[u8]> },
    Web,
}
#[derive(Debug, PartialEq, Eq)]
struct NeedValue {
    object: c::ObjectKey,
    project: Option<u32>,
    mark: shared::NeedMark,
    line: shared::NeedLine,
    act: NeedAction,
}
fn need_value(object: c::ObjectKey, mark: shared::NeedMark, words: &[u8], act: NeedAction) -> NeedValue {
    NeedValue {
        object,
        project: Some(f::PROJECT),
        mark,
        line: shared::NeedLine {
            kind: if mark == shared::NeedMark::Question { shared::NeedKind::Question } else { shared::NeedKind::Other },
            words: words.into(),
        },
        act,
    }
}
fn prompt_question(message: u64, admitted: bool) -> (c::ObjectKey, c::Read, c::ReadValue) {
    let mut chip = f::task::document().chip;
    chip.task = f::INBOX_TASK;
    chip.title = f::task::document_words(b"Parser");
    (
        c::ObjectKey::Question { task: f::INBOX_TASK, message },
        c::Read::Question { task: f::INBOX_TASK, message },
        c::ReadValue::Question(Box::new(c::Question {
            task: chip,
            from: None,
            message,
            words: c::DocumentText::complete(f::first_run::text(
                b"Which parser?\nChoice: Keep it\nChoice: Replace it (recommended)",
            )),
            since: Wall::EPOCH,
            acts: if admitted {
                List::from_boxed(Box::new([c::Offered {
                    act: c::ActKind::Answer,
                    admission: c::Admission::Allowed,
                    effect: None,
                }]))
            } else {
                List::with_capacity(0)
            },
            ending: None,
        })),
    )
}
fn question_need(message: u64) -> NeedValue {
    let object = c::ObjectKey::Question { task: f::INBOX_TASK, message };
    need_value(
        object,
        shared::NeedMark::Question,
        b"Parser: Which parser?",
        NeedAction::Answer(shared::Question {
            object,
            head: b"Which parser?".as_slice().into(),
            choices: Box::new([
                shared::QuestionChoice { words: b"Keep it".as_slice().into(), recommended: false },
                shared::QuestionChoice { words: b"Replace it".as_slice().into(), recommended: true },
            ]),
            primary: Some(1),
        }),
    )
}
fn held_needs(reason: c::HoldCategory) -> d::Domain {
    let held = super::needs::held(Some(reason));
    let task = held.task.chip.task;
    super::needs::kinds(&[(
        c::ObjectKey::Escalation(task),
        c::Read::Escalation { task },
        c::ReadValue::Escalation(Box::new(held)),
    )])
}
fn approval_needs(row: &d::forge::approvals::Row) -> d::Domain {
    let mut root = super::needs::kinds(&[]);
    super::needs::approvals(&mut root, core::slice::from_ref(row), 0);
    root
}
fn review_need(row: &d::forge::approvals::Row) -> NeedValue {
    need_value(
        c::ObjectKey::Task(row.task),
        shared::NeedMark::SignOff,
        b"A change: Approve A change on Forgejo",
        NeedAction::Review {
            words: "Review on Forgejo ↗".as_bytes().into(),
            address: b"https://forge.example/work/hello/pulls/24/files".as_slice().into(),
        },
    )
}
struct NeedCase {
    name: &'static str,
    root: Box<d::Domain>,
    expected: Box<[NeedValue]>,
}
fn need_case(name: &'static str, root: d::Domain, expected: Box<[NeedValue]>) -> NeedCase {
    NeedCase { name, root: Box::new(root), expected }
}
#[test]
#[expect(clippy::too_many_lines, reason = "one table checks each need's mark, prompt and merged order")]
fn need_prompts_say_each_kind() {
    use shared::NeedMark;
    let w = f::browser_words();
    let approval = super::needs::approval_row();
    let no_account = d::forge::approvals::Row { yours: d::forge::approvals::Yours::NoAccount, ..approval.clone() };
    let mut proposal = super::needs::proposal(c::Proposer::Task(f::INBOX_TASK));
    proposal.action = c::ProposalAction::Settings(Box::new(c::ProposalActionSettings {
        against: 1,
        items: List::with_capacity(0),
        review: None,
        made: None,
    }));
    let proposal_object = c::ObjectKey::Proposal(proposal.key);
    let proposal_read = c::Read::Proposal { proposal: proposal.key };
    let mut ordered = super::needs::kinds(&[prompt_question(1, true), prompt_question(2, true)]);
    super::needs::approvals(&mut ordered, core::slice::from_ref(&approval), 0);
    act(
        &mut ordered,
        Act::Needs(d::pages::needs::Act::Later { object: c::ObjectKey::Question { task: f::INBOX_TASK, message: 1 } }),
    );
    let mut no_inbox = d::unlinked(&f::domain_limits(), f::SEED);
    super::needs::approvals(&mut no_inbox, core::slice::from_ref(&approval), 0);
    let rows = [
        need_case(
            "question choices and primary",
            super::needs::kinds(&[prompt_question(1, true)]),
            Box::new([question_need(1)]),
        ),
        need_case(
            "question not admitted",
            super::needs::kinds(&[prompt_question(1, false)]),
            Box::new([need_value(
                c::ObjectKey::Question { task: f::INBOX_TASK, message: 1 },
                NeedMark::Question,
                b"Parser: Which parser?",
                NeedAction::Web,
            )]),
        ),
        need_case(
            "budget hold",
            held_needs(c::HoldCategory::Budget),
            Box::new([need_value(
                c::ObjectKey::Escalation(f::INBOX_TASK),
                NeedMark::Money,
                b"first-run-projects: 31 of 40 used",
                NeedAction::Web,
            )]),
        ),
        need_case(
            "other hold",
            held_needs(c::HoldCategory::Stalled),
            Box::new([need_value(
                c::ObjectKey::Escalation(f::INBOX_TASK),
                NeedMark::Other,
                b"first-run-projects: first-run-projects",
                NeedAction::Web,
            )]),
        ),
        need_case(
            "settings proposal",
            super::needs::kinds(&[(proposal_object, proposal_read, c::ReadValue::Proposal(Box::new(proposal)))]),
            Box::new([need_value(
                proposal_object,
                NeedMark::Other,
                b"Task 9: Accept how temper will run your work? The agent follows this plan.",
                NeedAction::Web,
            )]),
        ),
        need_case("counting approval", approval_needs(&approval), Box::new([review_need(&approval)])),
        need_case(
            "approval without account",
            approval_needs(&no_account),
            Box::new([need_value(
                c::ObjectKey::Task(approval.task),
                NeedMark::SignOff,
                b"A change: Approve A change on Forgejo",
                NeedAction::Web,
            )]),
        ),
        need_case(
            "cards then approvals then later",
            ordered,
            Box::new([question_need(2), review_need(&approval), question_need(1)]),
        ),
        need_case("no inbox", no_inbox, Box::default()),
    ];
    let mut failed = Vec::new();
    for NeedCase { name, root, expected } in rows {
        let prompts = shared::need_prompts(&root, &w)
            .into_vec()
            .into_iter()
            .map(|prompt| NeedValue {
                object: prompt.object,
                project: prompt.project,
                mark: prompt.mark,
                line: prompt.line,
                act: match prompt.act {
                    shared::NeedAct::Answer(question) => NeedAction::Answer(question),
                    shared::NeedAct::Review { words, address } => NeedAction::Review { words, address },
                    shared::NeedAct::Web => NeedAction::Web,
                },
            })
            .collect::<Vec<_>>();
        if prompts.as_slice() != expected.as_ref() {
            failed.push((name, prompts, expected));
        }
    }
    assert!(failed.is_empty(), "need prompt rows: {failed:?}");
}
