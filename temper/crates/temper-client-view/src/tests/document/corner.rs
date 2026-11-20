//! The pane's live path (client/pages/chat/document.md, 1).
use super::*;
use jig_client::{Event, Up, Watchable};
use skein_lib::{List, Token};
use temper_fixtures::client_documents as documents;

const ASKED: Wall = Wall::from_nanos(10_250_000_000);

fn text(bytes: &[u8]) -> c::Text {
    c::Text::new(bytes.into(), 512).unwrap()
}
fn step(root: &mut d::Domain, wall: Wall, event: d::Event) -> Queue<d::Request> {
    let mut out = Queue::uncapped();
    d::step(root, &Env { now: Time::ZERO, wall, limits: limits() }, event, &mut out);
    out
}
fn deliver(root: &mut d::Domain, watch: Token, value: c::Delivery) -> Queue<d::Request> {
    step(
        root,
        ASKED,
        d::Event::Client { event: Event::Up { up: Up::Watched { watch, event: c::WatchEvent::Delivered(value) } } },
    )
}
fn at(root: &d::Domain, shell: Shell, wall: Wall) -> crate::View {
    let words = crate::words::Words::english(f::FORGE, 512, shell).unwrap();
    let limits = limits();
    let bounds = crate::Limits::of(&limits, &words).unwrap();
    let mut view = crate::View::new(&bounds, &limits, words, f::FORGE);
    crate::build(&mut view, root, wall, &mut Queue::uncapped());
    view
}
fn state() -> c::RunState {
    c::RunState { task: f::chat::TASK, attempt: Some(f::chat::ATTEMPT), committed_turn: 0, writing: None, call: None }
}
fn run_text(tool: &[u8]) -> c::RunText {
    c::RunText {
        attempt: f::chat::ATTEMPT,
        began: Some(ASKED),
        turn: c::ProvisionalTurnName { attempt: f::chat::ATTEMPT, number: 2, retry: 0 },
        from: 0,
        through: 0,
        text: text(b""),
        cut: false,
        call: None,
        writing: Some(c::WritingCall {
            id: text(b"w1"),
            tool: text(tool),
            from: 0,
            through: 0,
            input: text(b""),
            cut: false,
        }),
    }
}
fn call(tool: &[u8], input: &[u8]) -> c::CurrentCall {
    c::CurrentCall {
        identity: c::CallIdentity { owner: Token::new(1), sequence: 1, retry: 0, id: text(b"c1") },
        tool: text(tool),
        input: text(input),
        cut: false,
        began: None,
    }
}
#[expect(clippy::too_many_lines, reason = "one chat world delivers its activity, task, document and run watch")]
fn live(body: Option<&[u8]>, mode: Mode) -> (d::Domain, Token, Token) {
    let mut page = activity(&[]);
    page.rows = List::with_capacity(3);
    page.rows
        .push(documents::turn(
            3,
            Wall::from_nanos(12_000_000_000),
            c::TurnName { attempt: f::chat::ATTEMPT, number: 1 },
            c::TurnRead::Decoded(c::Turn {
                messages: List::with_capacity(0),
                used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
                spent: 0,
                read_through: None,
                llm: None,
                cut: false,
            }),
        ))
        .unwrap();
    page.rows
        .push(documents::line(
            2,
            Wall::from_nanos(10_750_000_000),
            c::Activity::RunBegan { attempt: f::chat::ATTEMPT, why: c::RunWhy::Made, resumed: false, fresh: None },
        ))
        .unwrap();
    page.rows
        .push(documents::line(
            1,
            ASKED,
            c::Activity::Words {
                message: 1,
                by: c::Requester::Person(f::chat::PARTY),
                kind: c::MessageKind::Words,
                words: text(d::pages::chat::document::WRITE_THIS_UP),
            },
        ))
        .unwrap();
    let (mut root, me, task_watch, _, _) =
        f::root_at_with_activity(d::Address::Chat { task: f::chat::TASK }, &limits(), b"Shop", None, Some(page), None);
    let source = offered(None, false);
    f::deliver_me(&mut root, me, source.pages().me(source.client()).unwrap().clone());
    let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
    task.activity = 3;
    f::task::phase(&mut task, c::ChipPhase::Active(c::RunStage::Running { attempt: f::chat::ATTEMPT }));
    let task_watch = task_watch.unwrap();
    let mut out = deliver(&mut root, task_watch, c::Delivery::Task(Box::new(task)));
    let mut run = None;
    while let Some(request) = out.pop() {
        match request {
            d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Open { watch, subject: Watchable::Jig(c::Subject::Run { .. }) },
                    },
            } => run = Some(watch),
            d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down:
                            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Notes { scope, .. }) },
                    },
            } => {
                let mut rows = List::with_capacity(1);
                if let Some(body) = body {
                    rows.push(c::Note {
                        number: 1,
                        revision: c::Revision { value: 1 },
                        scope,
                        description: c::DocumentText::complete(text(TITLE)),
                        body: c::DocumentText::complete(c::Text::new(body.into(), LONG).unwrap()),
                        writer: c::NoteAuthor::Task { task: f::chat::TASK, attempt: 1 },
                        tasks: List::with_capacity(0),
                        changed: Some(Wall::EPOCH),
                        acts: List::with_capacity(0),
                        cut: false,
                    })
                    .unwrap();
                }
                step(
                    &mut root,
                    ASKED,
                    d::Event::Client {
                        event: Event::Up {
                            up: Up::ReadDone {
                                read,
                                result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(
                                    c::NotePage { rows, before: None, after: None, cut: false },
                                )))),
                            },
                        },
                    },
                );
            }
            d::Request::Push(_)
            | d::Request::Screen { .. }
            | d::Request::KeepLook { .. }
            | d::Request::KeepSignIn { .. }
            | d::Request::Tab { .. }
            | d::Request::Copy { .. }
            | d::Request::Client { .. }
            | d::Request::Unlinked { .. }
            | d::Request::Show { .. }
            | d::Request::Leave { .. }
            | d::Request::SignOut
            | d::Request::Dropped { .. }
            | d::Request::Fact { .. } => {}
        }
    }
    step(&mut root, ASKED, d::Event::Act { action: d::Action::Pages(Act::Mode { mode }) });
    assert_eq!(root.pages().chat().unwrap().draft.is_some(), body.is_some());
    (root, task_watch, run.expect("a live chat watches its run"))
}

#[test]
fn the_path_counts_from_the_press_and_only_the_browser_ticks() {
    for mode in [Mode::Chat, Mode::Split, Mode::Draft] {
        let mut root = offered(None, false);
        step(&mut root, ASKED, d::Event::Act { action: d::Action::Pages(Act::AddDraft) });
        step(&mut root, ASKED, d::Event::Act { action: d::Action::Pages(Act::Mode { mode }) });
        for shell in [Shell::Browser, Shell::Terminal] {
            for (wall, expected, next) in [
                (ASKED, "thinking · 0 s", 11_000_000_000),
                (Wall::from_nanos(12_250_000_000), "thinking · 2 s", 13_000_000_000),
                (Wall::from_nanos(76_250_000_000), "thinking · 66 s", 77_000_000_000),
            ] {
                let mut view = at(&root, shell, wall);
                if mode == Mode::Chat {
                    assert_eq!(styled(view.tree(), look::DOC_PATH).len(), 0);
                    assert_eq!(view.next_change(), None);
                    continue;
                }
                let separator = if shell == Shell::Terminal { " · " } else { "" };
                assert_eq!(
                    section_words(view.tree(), look::DOC_PATH),
                    format!("untitled.md{separator}{expected}").as_bytes()
                );
                assert_eq!(
                    view.next_change(),
                    if shell == Shell::Browser { Some(Wall::from_nanos(next)) } else { None }
                );
                if shell == Shell::Terminal {
                    let mut print = skein_ui_text::Print::new(&view.limits().tree).unwrap();
                    skein_ui_text::render(view.tree(), b"temper", None, &mut print);
                    assert!(
                        String::from_utf8_lossy(print.text.as_slice()).contains(&format!("untitled.md · {expected}"))
                    );
                } else {
                    crate::build(&mut view, &root, Wall::from_nanos(next + 1_000_000_000), &mut Queue::uncapped());
                    assert_ne!(
                        section_words(view.tree(), look::DOC_PATH),
                        format!("untitled.md{expected}").as_bytes(),
                        "the timer rebuilds without a domain change"
                    );
                }
            }
        }
    }
}

#[test]
fn the_path_names_calls_and_the_draft_call_being_composed() {
    for (tool, input, composing, streamed, elapsed, expected) in [
        (
            b"see".as_slice(),
            br#"{"address":"proposal t3/4"}"#.as_slice(),
            false,
            false,
            4,
            "reading proposal t3/4 · 4 s",
        ),
        (b"unphrased", b"{}", false, false, 4, "unphrased · 4 s"),
        (b"document", b"{}", false, false, 11, "writing · 11 s"),
        (b"note", b"{}", false, false, 11, "writing · 11 s"),
        (b"document", b"", true, false, 11, "writing · 11 s"),
        (b"note", b"", true, true, 11, "writing · 11 s"),
        (b"other", b"", true, true, 11, "thinking · 11 s"),
    ] {
        for mode in [Mode::Draft, Mode::Split] {
            let (mut root, _, run) = live(None, mode);
            let mut state = state();
            if !composing {
                state.call = Some(call(tool, input));
            }
            if composing && !streamed {
                state.writing = Some(run_text(tool));
            }
            deliver(&mut root, run, c::Delivery::RunState(Box::new(state)));
            if streamed {
                deliver(&mut root, run, c::Delivery::Run(Box::new(run_text(tool))));
            }
            let view = at(&root, Shell::Browser, Wall::from_nanos(ASKED.as_nanos() + elapsed * 1_000_000_000));
            assert_eq!(section_words(view.tree(), look::DOC_PATH), format!("untitled.md{expected}").as_bytes());
        }
    }
    // The domain's hold timer must invalidate even a terminal view, which has no clock tick.
    let (mut root, _, run) = live(None, Mode::Draft);
    let mut reading = state();
    reading.call = Some(call(b"see", br#"{"address":"proposal t3/4"}"#));
    deliver(&mut root, run, c::Delivery::RunState(Box::new(reading)));
    deliver(&mut root, run, c::Delivery::RunState(Box::new(state())));
    let mut view = at(&root, Shell::Terminal, ASKED);
    assert_eq!(section_words(view.tree(), look::DOC_PATH), "untitled.md · reading proposal t3/4 · 0 s".as_bytes());
    d::fire(
        &mut root,
        &Env {
            now: Time::ZERO.saturating_add(skein_lib::Duration::from_secs(1)),
            wall: Wall::from_nanos(11_250_000_000),
            limits: limits(),
        },
        &mut Queue::uncapped(),
    );
    crate::build(&mut view, &root, Wall::from_nanos(11_250_000_000), &mut Queue::uncapped());
    assert_eq!(section_words(view.tree(), look::DOC_PATH), "untitled.md · thinking · 1 s".as_bytes());
}

#[test]
fn a_reopened_pane_times_its_run_from_activity_and_stops_with_it() {
    for (body, expected) in [(None, "thinking · 4 s"), (Some(BODY), "thinking · 3 s")] {
        let (mut root, task_watch, run) = live(body, Mode::Draft);
        deliver(&mut root, run, c::Delivery::RunState(Box::new(state())));
        let wall = Wall::from_nanos(14_250_000_000);
        let view = at(&root, Shell::Browser, wall);
        let path = if body.is_some() { "docs/design/draft/how-shop-works.md" } else { "untitled.md" };
        assert_eq!(section_words(view.tree(), look::DOC_PATH), format!("{path}{expected}").as_bytes());
        assert_eq!(view.next_change(), Some(Wall::from_nanos(15_000_000_000)));
        let mut ended = state();
        ended.attempt = None;
        deliver(&mut root, run, c::Delivery::RunState(Box::new(ended)));
        let after_run = at(&root, Shell::Browser, wall);
        let right = if body.is_some() { "edited just now" } else { "" };
        assert_eq!(section_words(after_run.tree(), look::DOC_PATH), format!("{path}{right}").as_bytes());
        let mut task = root.pages().chat().unwrap().task(root.client()).unwrap().clone();
        f::task::phase(&mut task, c::ChipPhase::Active(c::RunStage::Idle));
        deliver(&mut root, task_watch, c::Delivery::Task(Box::new(task)));
        let view = at(&root, Shell::Browser, wall);
        let right = if body.is_some() { "edited just now" } else { "" };
        assert_eq!(section_words(view.tree(), look::DOC_PATH), format!("{path}{right}").as_bytes());
        assert!(
            view.next_change().is_none_or(|next| next > Wall::from_nanos(15_000_000_000)),
            "no second tick remains"
        );
    }
}
