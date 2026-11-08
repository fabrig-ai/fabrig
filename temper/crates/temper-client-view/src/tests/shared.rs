//! Shared words match the browser's trees (temper/docs/design/client/README.md, 6.14).
#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect semantic trees")]
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
    let mut task = super::chat::configuring_task();
    task.chip.details.phase = c::ChipPhase::Active(if running { c::RunStage::Due } else { c::RunStage::Idle });
    task.chip.phase = task.chip.details.phase.compact();
    task.waits_on = if running { c::WaitsOn::Nothing } else { c::WaitsOn::News };
    task.chip.details.details.live_delegates = 0;
    task.requester = c::Requester::Person(f::chat::PARTY);
    task.wake.words = c::WakeRule::Immediate;
    task.delegates.clear();
    task.run_wait = None;
    let mut root = super::chat::configuring_task_root(task, true, 1);
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
            let mut task = super::chat::configuring_task();
            task.chip.details.phase = c::ChipPhase::Ended;
            task.chip.phase = c::Phase::Ended;
            task.acts.clear();
            root = super::chat::configuring_task_root(task, true, 1);
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
    let browser = view
        .tree()
        .nodes()
        .iter()
        .flat_map(|node| node.text.iter())
        .find(|run| run.text == line.words)
        .map(|run| run.text.clone());
    [
        words("home need line", line.words, browser),
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
