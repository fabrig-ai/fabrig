//! Chat entries and hand-written frames (temper/docs/design/client/README.md, 6.14 and 6.15).
use super::{fixtures as f, goldens, printed as p};
use crate::{
    Scheme, Told, Tui, look,
    view::{
        self,
        chat::{self, Chat, Entry, Piece},
        foot, printed,
    },
};
use jig_client::{Down, UnlinkedRequest, contract as c};
use skein_lib::{Env, List, Queue, Time, Wall};
use skein_ui::{Role, Style};
use temper_client_domain::pages::chat::Shown;
use temper_client_domain::{self as d, Address, Domain, Event};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_client_view::shared;

const TASK: u64 = 2;
const DELEGATE: u64 = 7;
const ATTEMPT: u64 = 44;
const SIX_SECONDS: Wall = Wall::from_nanos(6_000_000_000);
const REQUEST: &[u8] = b"What landed overnight?";
const ANSWER: &[u8] = b"Three changes landed between 22:10 and 06:40:\n\n1. csv-export: the report's columns, the first of its three tasks.\n2. csv-export: the export button on the report page.\n3. fix-login: a session no longer ends when the forge restarts.\n\ncsv-export's last task waits on you: whether the file separates its\nvalues with a comma or a";
const COMMITTED: &[u8] = b"Three changes landed between 22:10 and 06:40:\n\n1. csv-export: the report's columns, the first of its three tasks.\n2. csv-export: the export button on the report page.\n3. fix-login: a session no longer ends when the forge restarts.\n\ncsv-export's last task waits on you: whether the file separates its\nvalues with a comma or a semicolon. Most spreadsheets expect a comma.";

fn text(bytes: &[u8]) -> c::Text {
    c::Text::new(bytes.into(), chat_limits().client.domain.long_text).unwrap()
}
fn line(position: u64, activity: c::Activity) -> c::DecodedActivityLine {
    c::DecodedActivityLine {
        source: Box::new(c::ActivityLine { task: TASK, position, at: Wall::EPOCH, line: activity }),
        decoded: None,
    }
}
fn words(by: c::Requester, bytes: &[u8]) -> c::DecodedActivityLine {
    line(1, c::Activity::Words { message: 1, by, kind: c::MessageKind::Words, words: text(bytes) })
}
fn call(number: u64, tool: &[u8], input: &[u8], failed: bool) -> c::Block {
    c::Block::Call(c::Call {
        number,
        tool: text(tool),
        input: text(input),
        outcome: Some(text(b"denied\nprivate details")),
        failed,
        objects: List::with_capacity(0),
        parts: List::with_capacity(0),
    })
}
fn turn(blocks: &[c::Block]) -> c::DecodedActivityLine {
    let mut turn = jig_client_protocol_link::translate::pages::activity_page(
        jig_client_test_records::decoded_activity_record(),
        &link(),
    )
    .unwrap()
    .rows
    .swap_remove(0)
    .unwrap()
    .decoded
    .unwrap();
    let c::TurnRead::Decoded(ref mut value) = turn.turn else { unreachable!() };
    value.messages = List::from_boxed(Box::new([c::Message {
        speaker: c::Speaker::Assistant,
        blocks: List::from_boxed(blocks.into()),
    }]));
    turn.task = TASK;
    turn.name = c::TurnName { attempt: ATTEMPT, number: 1 };
    turn.at = SIX_SECONDS;
    let mut row = line(1, c::Activity::Turn { reply: 1, attempt: ATTEMPT, turn: 1, read: None });
    row.source.at = SIX_SECONDS;
    row.decoded = Some(turn);
    row
}
fn chat_limits() -> d::Limits {
    let mut limits = f::limits();
    limits.client.domain.long_text = 2048;
    limits.client.domain.turn_text = 2048;
    limits
}
fn chat_event(root: &mut Domain, event: Event) -> Queue<d::Request> {
    let mut out = Queue::uncapped();
    d::step(root, &Env { now: Time::ZERO, wall: f::NOW, limits: chat_limits() }, event, &mut out);
    out
}
fn link() -> jig_client_protocol_link::Limits {
    jig_client_protocol_link::Limits::new(chat_limits().client.domain, 8).unwrap()
}
fn task() -> c::Task {
    let mut task =
        jig_client_protocol_link::translate::documents::task(jig_client_test_records::task_record(), &link()).unwrap();
    task.project = 1;
    task.chip.task = TASK;
    task.chip.title = c::DocumentText::complete(text(REQUEST));
    task.chip.executor = c::ChipExecutor::Agent { charter: 1 };
    task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Running { attempt: ATTEMPT });
    task.chip.phase = c::Phase::Active;
    task.chip.details.details.live_delegates = 0;
    task.requester = c::Requester::Person(73);
    task.result = None;
    task.run_wait = None;
    task.waits_on = c::WaitsOn::Nothing;
    task.delegates.clear();
    task.proposals.clear();
    task.flags.conversation = true;
    task.activity = 1;
    task.spec.specification.words = c::DocumentText::complete(text(REQUEST));
    task
}
fn cancelled_task() -> c::Task {
    let mut task = task();
    task.chip.details.phase = c::ChipPhase::Ended;
    task.chip.phase = c::Phase::Cancelled;
    task.result = Some(c::TaskResult::Ending(Box::new(c::ChipEnding::Cancelled {
        reason: c::DocumentText::complete(text(b"closed")),
        result: None,
        by: c::Requester::Person(73),
        at: Wall::EPOCH,
    })));
    task
}
struct Input {
    task: Option<Box<c::Task>>,
    rows: Box<[c::DecodedActivityLine]>,
    writing: Option<&'static [u8]>,
    current: bool,
    working: bool,
    older: bool,
    shown: Option<Shown>,
}
impl Input {
    fn empty() -> Self {
        Self {
            task: None,
            rows: Box::default(),
            writing: None,
            current: false,
            working: false,
            older: false,
            shown: None,
        }
    }
}
fn activity_page(input: &Input, before: Option<c::Cursor>, answered: &mut bool) -> c::ActivityPage {
    let rows = if before.is_some() {
        assert!(input.older && *answered, "only the older fixture continues its first page");
        List::with_capacity(0)
    } else {
        assert!(!*answered, "fixture requested its latest activity page twice");
        *answered = true;
        List::from_boxed(input.rows.clone())
    };
    c::ActivityPage {
        rows,
        cut: false,
        before: if input.older { Some(c::Cursor::new(b"older".as_slice().into(), 32).unwrap()) } else { None },
        after: None,
    }
}
#[expect(clippy::wildcard_enum_match_arm, reason = "the fixture answers its task, activity and run documents alone")]
#[expect(clippy::too_many_lines, reason = "the fixture answers chat and view reads through the root's events")]
#[expect(clippy::manual_map, reason = "the read answer uses an explicit match rather than a closure")]
fn root(input: &Input) -> Domain {
    let mut root = d::unlinked(&chat_limits(), f::SEED);
    chat_event(&mut root, Event::Client { event: jig_client::Event::StartUnlinked { kept: List::with_capacity(0) } });
    let mut opened = chat_event(
        &mut root,
        Event::Start {
            address: Address::Chat { task: TASK },
            kept: List::with_capacity(0),
            with: Some(jig_client::With::Transport),
        },
    );
    chat_event(
        &mut root,
        Event::Client { event: jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } } },
    );
    let mut out = Queue::uncapped();
    while let Some(request) = opened.pop() {
        match request {
            d::Request::Unlinked {
                request:
                    UnlinkedRequest::Down {
                        down: Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Me) },
                    },
            } => {
                let mut replies = chat_event(
                    &mut root,
                    Event::Client {
                        event: jig_client::Event::Up {
                            up: jig_client::Up::Watched {
                                watch,
                                event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(f::me(&[b"Hello"])))),
                            },
                        },
                    },
                );
                while let Some(reply) = replies.pop() {
                    out.push(reply);
                }
            }
            request => {
                out.push(request);
            }
        }
    }
    let mut task = match &input.task {
        Some(task) => task.as_ref().clone(),
        None => task(),
    };
    task.activity = match input.rows.last() {
        Some(row) => row.source.position,
        None => 0,
    };
    if !input.working && task.result.is_none() {
        task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Idle);
        task.waits_on = c::WaitsOn::News;
    }
    let mut run = jig_client_test_records::run_state_record().into_parts();
    run.task = TASK;
    run.attempt = Some(ATTEMPT);
    run.committed_turn = 0;
    if let Some(bytes) = input.writing {
        let mut writing = jig_client_test_records::run_text_record().into_parts();
        writing.attempt = ATTEMPT;
        writing.turn = jig_client_protocol_wire::ProvisionalTurnName::new(
            &jig_client_protocol_wire::CEILINGS,
            jig_client_protocol_wire::ProvisionalTurnNameParts { attempt: ATTEMPT, number: 2, retry: 0 },
        )
        .unwrap();
        writing.from = 0;
        writing.through = u32::try_from(bytes.len()).unwrap();
        writing.text = bytes.into();
        run.writing =
            Some(jig_client_protocol_wire::RunText::new(&jig_client_protocol_wire::CEILINGS, writing).unwrap());
    }
    if input.current {
        let mut call = jig_client_test_records::current_call_record().into_parts();
        call.tool = b"see".as_slice().into();
        call.input = b"{\"address\":\"change 3\"}".as_slice().into();
        run.call = Some(jig_client_protocol_wire::CurrentCall::new(&jig_client_protocol_wire::CEILINGS, call).unwrap());
    }
    let run = jig_client_protocol_link::translate::live::run_state(
        jig_client_protocol_wire::RunState::new(&jig_client_protocol_wire::CEILINGS, run).unwrap(),
        &link(),
    )
    .unwrap();
    let mut activity_answered = false;
    while let Some(request) = out.pop() {
        let up = match request {
            d::Request::Unlinked {
                request:
                    UnlinkedRequest::Down {
                        down: Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Task { task: TASK }) },
                    },
            } => Some(jig_client::Up::Watched {
                watch,
                event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task.clone()))),
            }),
            d::Request::Unlinked {
                request:
                    UnlinkedRequest::Down {
                        down:
                            Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Run { task: TASK, .. }) },
                    },
            } => Some(jig_client::Up::Watched {
                watch,
                event: c::WatchEvent::Delivered(c::Delivery::RunState(Box::new(run.clone()))),
            }),
            d::Request::Unlinked {
                request:
                    UnlinkedRequest::Down {
                        down: Down::Read { read, what: jig_client::Readable::Jig(c::Read::Activity { before, .. }) },
                    },
            } => Some(jig_client::Up::ReadDone {
                read,
                result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(
                    activity_page(input, before, &mut activity_answered),
                )))),
            }),
            d::Request::Unlinked {
                request:
                    UnlinkedRequest::Down { down: Down::Read { read, what: jig_client::Readable::Family { record, .. } } },
            } => {
                let reading = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap();
                if let w::Read::View(view) = reading.read() {
                    assert_eq!(view.address(), p::ADDRESS);
                    let result = match &input.shown {
                        None | Some(Shown::Asking) => None,
                        Some(Shown::Failed) => {
                            Some(jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Refused)))
                        }
                        Some(Shown::Drawn(view)) => {
                            let answer =
                                w::ViewAnswer::new(&w::CEILINGS, w::ViewAnswerParts { view: *view.clone() }).unwrap();
                            Some(jig_client::Readout::Family {
                                family: wire::FORGE,
                                record: Some(wire::view_answer(&answer)),
                            })
                        }
                    };
                    match result {
                        Some(result) => Some(jig_client::Up::ReadDone { read, result }),
                        None => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some(up) = up {
            let mut replies = chat_event(&mut root, Event::Client { event: jig_client::Event::Up { up } });
            while let Some(reply) = replies.pop() {
                out.push(reply);
            }
        }
    }
    root
}
fn view() -> crate::View {
    let words = f::words();
    let page = temper_client_view::Limits::of(&f::limits(), words.view()).unwrap();
    crate::View::new(&crate::limits(&f::limits(), &page.tree, 512).unwrap(), words)
}
fn draw(part: Chat, list: Option<foot::Options>, columns: u16) -> crate::View {
    draw_request(part, list, columns, REQUEST)
}
fn draw_request(part: Chat, list: Option<foot::Options>, columns: u16, request: &[u8]) -> crate::View {
    let mut view = view();
    let root = f::live(Some(f::me(&[b"Hello"]))).0;
    let mut foot = foot::read(&root, &Tui::new(f::ADDRESS), &f::words(), f::NOW);
    foot.columns = columns;
    foot.launchpad = false;
    foot.rule = Some(foot::Rule {
        project: b"Hello".as_slice().into(),
        title: chat::cut(&shared::heading(request), &f::words()),
        task: TASK,
        place: f::ADDRESS.into(),
    });
    foot.closed = Some("Claude Opus 5.5 · high".as_bytes().into());
    foot.needs = d::pages::needs::Count { count: 1, cut: false };
    foot.composer.as_mut().unwrap().placeholder = b"Reply".as_slice().into();
    foot.composer.as_mut().unwrap().focus = 1;
    foot.running = if list.is_some() { None } else { Some(b"working".as_slice().into()) };
    foot.list = list;
    view::signed(&mut view, &view::Signed { body: view::Body::Chat(part), foot }, &mut Queue::uncapped());
    view
}
fn data(entries: Box<[Entry]>) -> Chat {
    Chat { to_end: 1, entries, notes: foot::Notes { lines: Box::default(), gave_up: None, log: None } }
}
struct Row {
    name: &'static str,
    input: Option<Input>,
    data: Option<Chat>,
    entries: &'static [(Style, &'static str)],
    end: usize,
    ordered: Option<u32>,
    failed: bool,
    code: usize,
}
fn row(name: &'static str, input: Input, entries: &'static [(Style, &'static str)]) -> Row {
    Row { name, input: Some(input), data: None, entries, end: 0, ordered: None, failed: false, code: 0 }
}
#[test]
#[expect(
    clippy::too_many_lines,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "one table inspects semantic trees and formats per-row failure diagnostics"
)]
fn the_chat_draws_each_entry() {
    for (title, expected) in [
        ("", ""),
        ("A short title", "A short title"),
        ("12345678901234567890", "12345678901234567890"),
        ("What can you tell me about Hello?", "What can you tell…"),
        ("123456789 123456789 next", "123456789 123456789…"),
        ("123456789 123456789  next", "123456789 123456789…"),
        ("123456789012345678901", "1234567890123456789…"),
        ("ééééééééé ééééééééé next", "ééééééééé ééééééééé…"),
        ("界界界界界界界界界界界界界界界界界界界界界", "界界界界界界界界界界界界界界界界界界界…"),
    ] {
        assert_eq!(chat::cut(title.as_bytes(), &f::words()).as_ref(), expected.as_bytes(), "{title}");
    }
    request_rule();
    let mut rows = [
        row(
            "request and later words",
            Input {
                rows: Box::new([words(c::Requester::Person(73), b"Later words\nNext paragraph")]),
                ..Input::empty()
            },
            &[(look::SAID, "What landed overnight?"), (look::SAID, "Later wordsNext paragraph")],
        ),
        row(
            "task words",
            Input { rows: Box::new([words(c::Requester::Task(DELEGATE), b"A delegate's words")]), ..Input::empty() },
            &[(look::SAID, "What landed overnight?"), (look::AGENT, "T7 · A delegate's words")],
        ),
        row(
            "two calls then text and time",
            Input {
                rows: Box::new([
                    line(1, c::Activity::RunBegan { attempt: ATTEMPT, why: c::RunWhy::Made, resumed: false }),
                    {
                        let mut t = turn(&[
                            call(1, b"settings", b"{}", false),
                            call(2, b"settings", b"{}", false),
                            c::Block::Text(text(b"The answer")),
                        ]);
                        t.source.position = 2;
                        t
                    },
                ]),
                ..Input::empty()
            },
            &[
                (look::SAID, "What landed overnight?"),
                (look::RUN, "Checked settings 2×  6 s"),
                (look::AGENT, "The answer"),
            ],
        ),
        row(
            "see phrases",
            Input {
                rows: Box::new([turn(&[
                    call(1, b"see", b"{\"address\":\"changes\"}", false),
                    call(2, b"see", b"{\"address\":\"change 1\"}", false),
                    call(3, b"see", b"{\"address\":\"change 2\"}", false),
                    call(4, b"see", b"{\"address\":\"change 3\"}", false),
                ])]),
                ..Input::empty()
            },
            &[(look::SAID, "What landed overnight?"), (look::RUN, "Listed changes, read 3 changes")],
        ),
        row(
            "failed run description",
            Input { rows: Box::new([turn(&[call(1, b"settings", b"{}", true)])]), ..Input::empty() },
            &[(look::SAID, "What landed overnight?"), (look::RUN, "Checked the inventory  failed: denied")],
        ),
        row(
            "ended call and current call",
            Input {
                working: true,
                current: true,
                rows: Box::new([turn(&[call(1, b"see", b"{\"address\":\"changes\"}", false)])]),
                ..Input::empty()
            },
            &[(look::SAID, "What landed overnight?"), (look::RUN, "Listed changes  reading change 3…")],
        ),
        row(
            "three dots",
            Input { working: true, ..Input::empty() },
            &[(look::SAID, "What landed overnight?"), (look::DOTS, "…")],
        ),
        row(
            "growing blocks",
            Input { working: true, writing: Some(b"A paragraph\n\n1. First\n2. Last"), ..Input::empty() },
            &[(look::SAID, "What landed overnight?"), (look::AGENT, "A paragraphFirstLast ")],
        ),
        row(
            "committed blocks",
            Input {
                rows: Box::new([turn(&[c::Block::Text(text(b"A paragraph\n\n1. First\n2. Last"))])]),
                ..Input::empty()
            },
            &[(look::SAID, "What landed overnight?"), (look::AGENT, "A paragraphFirstLast")],
        ),
        row(
            "a reply with a view between two paragraphs",
            Input {
                rows: Box::new([turn(&[c::Block::Text(text(p::MIXED))])]),
                shown: Some(Shown::Drawn(Box::new(p::way(false)))),
                ..Input::empty()
            },
            &[
                (look::SAID, "What landed overnight?"),
                (
                    look::AGENT,
                    "Under You approve:You approveAn agent makes each change; nothing reaches main until you approve it.○ you ask✦ produce▪ check✦ repair▪ check○ you approve▪ update✦ resolve▪ merge┃ mainNothing lands without you.",
                ),
            ],
        ),
        row(
            "a reply with a view still asking",
            Input { rows: Box::new([turn(&[c::Block::Text(text(p::MIXED))])]), ..Input::empty() },
            &[
                (look::SAID, "What landed overnight?"),
                (look::AGENT, "Under You approve:Drawing way You approve…Nothing lands without you."),
            ],
        ),
        row(
            "provisional text with a view block",
            Input { working: true, writing: Some(p::MIXED), ..Input::empty() },
            &[
                (look::SAID, "What landed overnight?"),
                (look::AGENT, "Under You approve:way You approve\nNothing lands without you. "),
            ],
        ),
        row(
            "hidden blocks",
            Input {
                rows: Box::new([turn(&[
                    c::Block::Opaque { dialect: text(b"reason"), size: 10 },
                    call(1, b"note", "{}".as_bytes(), false),
                ])]),
                older: true,
                ..Input::empty()
            },
            &[(look::RUN, "note")],
        ),
        Row {
            name: "card lines",
            input: None,
            data: Some(data(Box::new([
                Entry::Delegation(shared::DelegationLine {
                    kind: "T7 · Change".as_bytes().into(),
                    title: b"csv-export".as_slice().into(),
                    state: b"Landed".as_slice().into(),
                }),
                Entry::Proposal(b"Change the settings".as_slice().into()),
            ]))),
            entries: &[
                (look::AGENT, "T7 · Change · csv-export · Landed"),
                (look::AGENT, "Proposal Change the settings"),
            ],
            end: 0,
            ordered: None,
            failed: false,
            code: 0,
        },
        row(
            "requester cancellation is hidden",
            Input {
                task: Some(Box::new(cancelled_task())),
                rows: Box::new([line(1, c::Activity::Decided { revision: 1, proposal: None })]),
                ..Input::empty()
            },
            &[(look::SAID, "What landed overnight?")],
        ),
    ];
    rows[4].failed = true;
    rows[7].end = 1;
    rows[7].ordered = Some(1);
    rows[8].ordered = Some(1);
    rows[11].end = 1;
    rows[11].code = 1;
    let words = f::words();
    let mut failures = List::uncapped();
    for mut row in rows {
        let part = if let Some(input) = row.input {
            let root = root(&input);
            let tui = Tui::new(f::ADDRESS);
            chat::read(&root, &tui, &words, f::NOW)
        } else {
            row.data.take().unwrap()
        };
        let view = draw(part, None, 80);
        let tree = view.tree();
        let entries: Vec<_> = tree
            .nodes()
            .iter()
            .filter(|n| {
                n.role == Role::Item
                    && n.styles.as_slice().iter().any(|s| [look::SAID, look::AGENT, look::RUN, look::DOTS].contains(s))
            })
            .map(|n| (n.styles.as_slice()[0], f::text(tree, n)))
            .collect();
        let expected: Vec<_> =
            row.entries.iter().map(|(style, words)| (*style, Box::<[u8]>::from(words.as_bytes()))).collect();
        let end = tree.nodes().iter().filter(|n| n.styles.as_slice().contains(&look::END)).count();
        let ordered = tree.nodes().iter().find_map(|n| n.ordered);
        let failed = tree.nodes().iter().any(|n| n.styles.as_slice().contains(&look::RUN_FAILED));
        let code = tree.nodes().iter().filter(|n| n.role == Role::Code).count();
        let log = tree.nodes().iter().find(|n| n.role == Role::Log);
        let unbound = log.is_some_and(|n| {
            let at = f::index(tree, n.id);
            tree.nodes()[at..at + usize::try_from(n.size).unwrap()].iter().all(|n| n.binding.is_none())
        });
        if entries != expected
            || end != row.end
            || ordered != row.ordered
            || failed != row.failed
            || !unbound
            || code != row.code
        {
            failures.push(format!("{}: entries {entries:?}, expected {expected:?}; end {end}/{}, ordered {ordered:?}/{:?}, failed {failed}/{}, unbound {unbound}, code {code}/{}", row.name,row.end,row.ordered,row.failed,row.code)).unwrap();
        }
    }
    assert!(failures.is_empty(), "{}", failures.as_slice().join("\n"));
}

#[expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "the request's frame rows are compared as text"
)]
fn request_rule() {
    use skein_ui_screen::{Focus, Frame, Size, render};
    let request = b"What can you tell me about Hello?";
    let mut task = task();
    task.chip.title = c::DocumentText::complete(c::Text::new(request.as_slice().into(), 64).unwrap());
    task.spec.specification.words = c::DocumentText::complete(c::Text::new(request.as_slice().into(), 64).unwrap());
    let root = root(&Input { task: Some(Box::new(task)), ..Input::empty() });
    for (boxed, expected) in [
        (true, "Hello / What can you tell… T2 on box:40691"),
        (false, "Hello / What can you tell… T2 on 127.0.0.1:40691"),
    ] {
        let mut tui = Tui::new(b"127.0.0.1:40691");
        tui.step(Told::Size { columns: 80 });
        if boxed {
            tui.step(Told::Boxed);
        }
        let mut view = f::view();
        f::build(&mut view, &tui, &root);
        for scheme in [Scheme::Dark, Scheme::Light] {
            let size = Size { columns: 80, rows: 24 };
            let mut frame = Frame::new(size);
            render(view.tree(), &crate::theme(scheme, None), size, &mut Focus::default(), &mut frame);
            let printed = String::from_utf8(frame.print().into_vec()).unwrap();
            assert_eq!(printed.lines().next(), Some("❯ What can you tell me about Hello?"));
            let rule = printed.lines().find(|line| line.starts_with('─')).unwrap();
            assert_eq!(rule.trim_matches('─').trim(), expected);
            assert!(rule.ends_with(" ──"), "{rule}");
        }
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the golden rows embed hand-written frame literals")]
#[expect(clippy::too_many_lines, reason = "one table checks the chat frames and printed views at their review sizes")]
fn the_chats_are_their_goldens() {
    use skein_ui_screen::{Focus, Size};
    let full = Size { columns: 80, rows: 24 };
    let narrow = Size { columns: 40, rows: 12 };
    let rows = [
        ("chat-80x24-dark", full, false, true, include_bytes!("golden/chat-80x24-dark.txt").as_slice()),
        (
            "chat-80x24-dark-no-band",
            full,
            false,
            false,
            include_bytes!("golden/chat-80x24-dark-no-band.txt").as_slice(),
        ),
        (
            "chat-question-80x24-dark",
            full,
            true,
            true,
            include_bytes!("golden/chat-question-80x24-dark.txt").as_slice(),
        ),
        ("chat-40x12-dark", narrow, false, true, include_bytes!("golden/chat-40x12-dark.txt").as_slice()),
    ];
    let mut views = List::uncapped();
    for (_, size, question, _, _) in &rows {
        let mut entries = List::uncapped();
        entries.push(Entry::Said(REQUEST.into())).unwrap();
        entries
            .push(Entry::Run(shared::RunLine {
                name: b"Listed changes, read 3 changes".as_slice().into(),
                description: Some(b"6 s".as_slice().into()),
                failed: 0,
                running: false,
            }))
            .unwrap();
        entries
            .push(Entry::Agent { words: if *question { COMMITTED.into() } else { ANSWER.into() }, growing: !question })
            .unwrap();
        let list = if *question {
            entries.push(Entry::Said(b"Answer it with a comma".as_slice().into())).unwrap();
            foot::lists::question_list(
                shared::Question {
                    object: c::ObjectKey::Question { task: DELEGATE, message: 1 },
                    head: b"Answer csv-export's question with \"Comma\"?".as_slice().into(),
                    choices: Box::new([
                        shared::QuestionChoice { words: b"Answer Comma".as_slice().into(), recommended: true },
                        shared::QuestionChoice { words: b"Answer Semicolon".as_slice().into(), recommended: false },
                    ]),
                    primary: Some(0),
                },
                None,
                &f::words(),
            )
        } else {
            None
        };
        views.push(draw(data(entries.into_boxed()), list, size.columns)).unwrap();
    }
    let mut checks = List::uncapped();
    for ((name, size, _, band, want), view) in rows.iter().zip(&views) {
        checks
            .push(goldens::Golden {
                background: band.then_some(crate::theme::palette(Scheme::Dark)[0].1),
                name,
                tree: view.tree(),
                scheme: Scheme::Dark,
                size: *size,
                focus: Focus::default(),
                want,
            })
            .unwrap();
    }
    goldens::check(checks.as_slice());
    let view_rows = [
        ("chat-views-80x24-dark", full, Scheme::Dark, include_bytes!("golden/chat-views-80x24-dark.txt").as_slice()),
        ("chat-views-80x24-light", full, Scheme::Light, include_bytes!("golden/chat-views-80x24-light.txt").as_slice()),
        (
            "chat-views-110x32-dark",
            Size { columns: 110, rows: 32 },
            Scheme::Dark,
            include_bytes!("golden/chat-views-110x32-dark.txt").as_slice(),
        ),
    ];
    let request = b"How does a change land?";
    let mut views = List::uncapped();
    for (_, size, _, _) in &view_rows {
        let part = data(Box::new([
            Entry::Said(request.as_slice().into()),
            Entry::Viewed(Box::new([
                Piece::Text(b"Under You approve:".as_slice().into()),
                Piece::View(printed::Printed::Drawn(Box::new(p::way(false)))),
                Piece::Text(b"Nothing lands without you.".as_slice().into()),
            ])),
        ]));
        views.push(draw_request(part, None, size.columns, request)).unwrap();
    }
    let mut checks = List::uncapped();
    for ((name, size, scheme, want), view) in view_rows.iter().zip(&views) {
        checks
            .push(goldens::Golden {
                name,
                size: *size,
                scheme: *scheme,
                want,
                tree: view.tree(),
                focus: Focus::default(),
                background: Some(crate::theme::palette(*scheme)[0].1),
            })
            .unwrap();
    }
    goldens::check(checks.as_slice());
}

#[test]
#[expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "the rendered rows are collected as text for comparison"
)]
fn agent_markdown_bullets_keep_their_own_mark_at_every_depth() {
    use skein_ui_screen::{Focus, Frame, Size, render};
    let bytes = b"Here is how:\n\n- **How temper works**: a box.\n  - nested\n- second\n\n1. one\n2. two";
    let expected =
        ["✦ Here is how:", "  • How temper works: a box.", "    • nested", "  • second", "  1. one", "  2. two"];
    let size = Size { columns: 80, rows: 24 };
    for growing in [false, true] {
        let view = draw(data(Box::new([Entry::Agent { words: bytes.as_slice().into(), growing }])), None, 80);
        let mut frame = Frame::new(size);
        render(view.tree(), &crate::theme(Scheme::Dark, None), size, &mut Focus::default(), &mut frame);
        let lines: Vec<String> = (0..size.rows)
            .map(|row| frame.row(row).unwrap().iter().map(|cell| cell.glyph).collect::<String>().trim_end().into())
            .filter(|row: &String| !row.is_empty())
            .take(expected.len())
            .collect();
        assert_eq!(lines, expected, "growing: {growing}");
    }
}
