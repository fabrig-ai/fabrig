//! The foot's states and hand-written frames (client/README.md, 6.14 and 6.15).
use super::{fixtures as f, goldens};
use crate::{
    Event as TuiEvent, Scheme, Told, Tui, look,
    view::{self, foot},
};
use jig_client::contract as c;
use skein_lib::{List, Queue};
use skein_ui::{Act, Role, State};
use temper_client_domain::{
    self as d, Address, Domain, Event, Place,
    pages::terminal::{Command, Listing},
};
use temper_client_view::shared;

fn text(bytes: &[u8]) -> c::Text {
    c::Text::new(bytes.into(), 32).unwrap()
}
fn model() -> c::MeModel {
    c::MeModel {
        new_until: None,
        number: 1,
        source: c::ModelSource::Stock,
        estimated: false,
        name: text(b"model"),
        provider: None,
        window: 8192,
        output: 1024,
        efforts: List::from_boxed(Box::new([text(b"low"), text(b"high")])),
        effort: Some(text(b"high")),
        prices: c::ModelPrices { input: 0, cached: 0, cache_write: None, output: 0, unit: 1 },
        description: None,
        shown: None,
        accounts: List::from_boxed(Box::new([1])),
        picker: true,
    }
}
pub(super) fn template(number: u32) -> c::Template {
    c::Template {
        number,
        name: c::DocumentText::complete(text(b"Chat")),
        executor: c::TemplateExecutor::Charter(1),
        contract: c::TaskContractDocument::Report { words: 32 },
        shallower: 1,
        budget: c::TemplateBudget::AsAllowed { cap: None },
        deadline: c::TemplateDeadline::None,
        tracked: None,
        flags: c::TaskFlags {
            ended_by: c::EndedBy::Result,
            conversation: true,
            listed: false,
            retention: c::Retention::Kept,
        },
        label: 7,
        grace: None,
        llm: None,
        resources: List::with_capacity(0),
    }
}
#[expect(
    clippy::fn_params_excessive_bools,
    reason = "a row supplies the project, picker, chat, delegate and draft facts"
)]
#[expect(
    clippy::too_many_lines,
    reason = "one fixture supplies the chat owner and its draft through their public events"
)]
fn root_state(project: bool, picker: bool, chat: bool, delegate: bool, draft: bool, state: TaskState) -> Domain {
    let mut me = f::me(if project { &[b"Demo"] } else { &[] });
    if let Some(project) = me.projects.get_mut(0) {
        project.templates = List::from_boxed(Box::new([template(1), template(5)]));
        if picker {
            project.defaults = List::from_boxed(Box::new([c::MeDefault {
                template: d::pages::chats::ASKING_TEMPLATE,
                choice: c::LlmChoice {
                    account: Some(1),
                    main: c::ModelEffort { model: text(b"model"), effort: Some(text(b"high")) },
                    subagents: List::with_capacity(0),
                    later: None,
                },
            }]));
            project.models = List::from_boxed(Box::new([model()]));
        }
    }
    if project && !picker {
        // The browser offers connecting this provider; the TUI needs a model to choose.
        me.providers = List::from_boxed(Box::new([c::ProviderOffer {
            provider: 1,
            name: c::DocumentText::complete(text(b"Provider")),
            key_bytes: 256,
            sign_in: false,
            key: true,
            programs: List::with_capacity(0),
            programs_cut: false,
            none: false,
            signed_in_plan: false,
            manage_usage: None,
        }]));
    }
    let (mut root, _) = f::live(Some(me));
    f::event(
        &mut root,
        Event::LookKept {
            look: d::pages::terminal::Look {
                terminal: true,
                theme: d::pages::terminal::Theme::Dark,
                font: d::pages::terminal::Font::DejaVu,
                night: false,
            },
        },
    );
    let address = if chat {
        Address::Chat { task: 9 }
    } else if project {
        Address::Home { project: 1 }
    } else {
        Address::Inbox { filter: d::address::Filter::ALL }
    };
    let mut out = f::event(&mut root, Event::Went { place: Place::Address(address) });
    if chat {
        let mut task_watch = None;
        for _ in 0..out.len() {
            if let Some(d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down:
                            jig_client::Down::Open {
                                watch,
                                subject: jig_client::Watchable::Jig(c::Subject::Task { task: 9 }),
                            },
                    },
            }) = out.pop()
            {
                task_watch = Some(watch);
            }
        }
        let link = jig_client_protocol_link::Limits::new(f::limits().client.domain, 8).unwrap();
        let mut task =
            jig_client_protocol_link::translate::documents::task(jig_client_test_records::task_record(), &link)
                .unwrap();
        task.chip.task = 9;
        task.project = 1;
        task.label = 7;
        task.chip.executor = c::ChipExecutor::Agent { charter: 1 };
        task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Idle);
        task.chip.phase = c::Phase::Active;
        task.flags.conversation = true;
        task.result = None;
        task.run_wait = None;
        set_state(&mut task, state);
        task.requester = c::Requester::Person(73);
        task.delegates = List::with_capacity(1);
        if delegate {
            let mut chip = task.chip.clone();
            chip.task = 10;
            task.delegates.push(c::LiveDelegate { chip, takeover_budget: 0 }).unwrap();
        }
        task.acts = List::from_boxed(Box::new([
            c::Offered { act: c::ActKind::Amend, admission: c::Admission::Allowed, effect: None },
            c::Offered { act: c::ActKind::Stop, admission: c::Admission::Allowed, effect: None },
            c::Offered { act: c::ActKind::Cancel, admission: c::Admission::Allowed, effect: None },
        ]));
        task.llm = Some(c::LlmChoice {
            account: Some(1),
            main: c::ModelEffort { model: text(b"model"), effort: Some(text(b"high")) },
            subagents: List::with_capacity(0),
            later: None,
        });
        let mut out = f::client(
            &mut root,
            jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: task_watch.expect("chat watch"),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        );
        if draft {
            for _ in 0..out.len() {
                if let Some(d::Request::Unlinked {
                    request:
                        jig_client::UnlinkedRequest::Down {
                            down:
                                jig_client::Down::Read {
                                    read,
                                    what: jig_client::Readable::Jig(c::Read::Notes { scope, .. }),
                                },
                        },
                }) = out.pop()
                {
                    let rows = List::from_boxed(Box::new([c::Note {
                        number: 1,
                        revision: c::Revision { value: 1 },
                        scope,
                        description: c::DocumentText::complete(text(b"Draft")),
                        body: c::DocumentText::complete(text(b"Write this up")),
                        writer: c::NoteAuthor::Task { task: 9, attempt: 1 },
                        tasks: List::with_capacity(0),
                        changed: None,
                        acts: List::with_capacity(0),
                        cut: false,
                    }]));
                    f::client(
                        &mut root,
                        jig_client::Event::Up {
                            up: jig_client::Up::ReadDone {
                                read,
                                result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(
                                    c::NotePage { rows, before: None, after: None, cut: false },
                                )))),
                            },
                        },
                    );
                }
            }
        }
    }
    root
}

#[derive(Clone, Copy)]
enum TaskState {
    Idle,
    Ended,
    Result,
    Paused,
    Working,
}
#[expect(clippy::fn_params_excessive_bools, reason = "fixture flags select the project's and chat's facts")]
fn root(project: bool, picker: bool, chat: bool, delegate: bool, draft: bool) -> Domain {
    root_state(project, picker, chat, delegate, draft, TaskState::Idle)
}
fn set_state(task: &mut c::Task, state: TaskState) {
    match state {
        TaskState::Idle => {}
        TaskState::Ended => {
            task.chip.details.phase = c::ChipPhase::Ended;
        }
        TaskState::Result => {
            task.result = Some(c::TaskResult::Ending(Box::new(c::ChipEnding::Failed(c::DocumentText::complete(
                text(b"failed"),
            )))));
        }
        TaskState::Working => {
            task.chip.details.phase = c::ChipPhase::Active(c::RunStage::Due);
            task.waits_on = c::WaitsOn::Nothing;
            task.wake.words = c::WakeRule::Immediate;
        }
        TaskState::Paused => {
            task.chip.details.phase = c::ChipPhase::Held { reason: c::HoldCategory::Stopped, revision: 1 };
            task.chip.details.details.hold = Some(c::ChipHold {
                placed: Some(c::PlacedHold {
                    number: 1,
                    owner: c::HoldOwner::Party { person: 73 },
                    mode: c::HoldMode::Stop,
                    scope: c::HoldScope::Subtree { task: 9 },
                    reason: Box::from(*b"pause"),
                    placed_at: f::NOW,
                    revision: c::Revision { value: 1 },
                    lifted: false,
                    may_lift: true,
                }),
                words: None,
                circle: None,
                account: None,
                source: None,
                reason: c::HoldCategory::Stopped,
                revision: 1,
                before: c::PriorPhase::Active(c::RunStage::Idle),
                escalation: None,
                tries: c::FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0 },
                hold_wait_since: None,
            });
        }
    }
}
fn act(root: &mut Domain, action: d::pages::Act) {
    f::event(root, Event::Act { action: d::Action::Pages(action) });
}
fn draw(part: foot::Foot) -> crate::View {
    let (root, _) = f::live(Some(f::me(&[b"Hello", b"Demo"])));
    let body = if part.rule.is_some() {
        view::Body::Chat(view::chat::Chat {
            to_end: 0,
            entries: Box::default(),
            notes: foot::Notes { lines: Box::default() },
        })
    } else {
        view::Body::Launchpad(view::launchpad::read(&root, &Tui::new(f::ADDRESS), &f::words(), f::NOW))
    };
    let mut view = f::view();
    view::signed(&mut view, &view::Signed { body, foot: part }, &mut Queue::uncapped());
    view
}

#[test]
fn active_command_name_is_plain_weight() {
    use skein_ui_screen::{Focus, Frame, Size, render};
    let mut root = root(true, true, false, false, false);
    act(&mut root, d::pages::Act::ChatWords { words: Box::from(*b"/") });
    let mut view = f::view();
    f::build(&mut view, &Tui::new(f::ADDRESS), &root);
    let size = Size { columns: 80, rows: 24 };
    for scheme in [Scheme::Dark, Scheme::Light] {
        let theme = crate::theme(scheme, None);
        let mut focus = Focus::default();
        let mut frame = Frame::new(size);
        for active in [b"/model".as_slice(), b"/new"] {
            if active == b"/new" {
                focus.active = Some(f::find(view.tree(), Role::MenuItem, active).id);
            }
            render(view.tree(), &theme, size, &mut focus, &mut frame);
            assert_eq!(focus.active, Some(f::find(view.tree(), Role::MenuItem, active).id));
            command_paints(&frame, active, &theme);
        }
    }
}

fn command_paints(frame: &skein_ui_screen::Frame, active: &[u8], theme: &skein_ui_screen::Theme) {
    let mut commands = 0_usize;
    for row in 0..frame.size().rows {
        let cells = frame.row(row).unwrap();
        if cells[2].glyph != '/' || cells[3].glyph == ' ' {
            continue;
        }
        commands = commands.checked_add(1).unwrap();
        let selected = cells[0].glyph == theme.glyphs.focus;
        let node_name = if selected { active } else { b"" };
        for (column, cell) in cells.iter().enumerate().skip(2) {
            if cell.glyph == ' ' {
                break;
            }
            assert_eq!(cell.paint.bold, !selected, "name at {row}:{column}");
            if selected {
                assert_eq!(cell.glyph, char::from(node_name[column.checked_sub(2).unwrap()]));
                assert_eq!(cell.paint.fore, theme.focus.fore);
            }
        }
        for cell in cells.iter().skip(11) {
            if cell.glyph != ' ' {
                assert!(!cell.paint.bold, "description on row {row}");
            }
        }
        if selected {
            for cell in cells {
                assert_eq!(cell.paint.back, theme.active.back);
            }
        }
    }
    assert_eq!(commands, 7);
}
#[expect(clippy::disallowed_methods, reason = "test assertions search the rendered tree")]
fn has(tree: &skein_ui::Tree, style: skein_ui::Style, words: &[u8]) -> bool {
    tree.nodes().iter().any(|node| node.styles.as_slice().contains(&style) && f::text(tree, node).as_ref() == words)
}

#[test]
#[expect(clippy::too_many_lines, reason = "one table checks the foot's read states and their bindings")]
#[expect(clippy::disallowed_methods, reason = "test assertions find and collect the rendered fields and options")]
#[expect(clippy::disallowed_types, reason = "test assertions collect the rendered options")]
fn the_foot_says_each_state() {
    let mut failures = List::with_capacity(32);
    for (name, project, chat, typed, listing, placeholder, commands) in [
        ("launchpad", true, false, "", None, Some("Ask temper"), &[][..]),
        ("no project", false, false, "", None, Some("/ for commands"), &[][..]),
        ("chat", true, true, "", None, Some("Reply"), &[][..]),
        (
            "commands",
            true,
            false,
            "/",
            None,
            None,
            &[
                Command::Model,
                Command::Effort,
                Command::New,
                Command::Chats,
                Command::Theme,
                Command::Help,
                Command::Exit,
            ][..],
        ),
        ("launchpad prefix", true, false, "/c", None, None, &[Command::Chats][..]),
        ("chat prefix", true, true, "/c", None, None, &[Command::Close, Command::Chats][..]),
        ("unknown", true, false, "/zz", None, Some("Ask temper"), &[][..]),
        ("whitespace", true, false, "/new ", None, Some("Ask temper"), &[][..]),
        ("unicode whitespace", true, false, "/new\u{a0}", None, Some("Ask temper"), &[][..]),
        ("models", true, false, "", Some(Listing::Model), None, &[][..]),
        ("launchpad efforts", true, false, "", Some(Listing::Effort), None, &[][..]),
        ("efforts", true, true, "", Some(Listing::Effort), None, &[][..]),
        ("chats", true, false, "", Some(Listing::Chats), None, &[][..]),
    ] {
        let mut root = root(project, true, chat, false, false);
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Went);
        if !typed.is_empty() {
            let act = if chat {
                d::pages::Act::MessageWords { words: typed.as_bytes().into() }
            } else {
                d::pages::Act::ChatWords { words: typed.as_bytes().into() }
            };
            f::event(&mut root, Event::Act { action: d::Action::Pages(act) });
        }
        if let Some(listing) = listing {
            let command = match listing {
                Listing::Model => Command::Model,
                Listing::Effort => Command::Effort,
                Listing::Chats => Command::Chats,
            };
            f::event(
                &mut root,
                Event::Act {
                    action: d::Action::Pages(d::pages::Act::Terminal(d::pages::terminal::Act::Run { command })),
                },
            );
        }
        let mut view = f::view();
        f::build(&mut view, &tui, &root);
        let tree = view.tree();
        let field = tree.nodes().iter().find(|node| node.role == Role::TextArea);
        let mut good = false;
        if let Some(field) = field {
            good = field.name.as_deref() == Some(if chat { b"Message" } else { b"Ask temper" })
                && field.placeholder.as_deref() == placeholder.map(str::as_bytes)
                && field.value.as_ref().unwrap().text.as_ref() == typed.as_bytes()
                && field.value.as_ref().unwrap().written == tui.box_written()
                && field.focus == Some(tui.focus())
                && field.styles.as_slice() == [look::FIELD]
                && field.states.as_slice().contains(&State::SubmitsOnEnter)
                && crate::event(&view, field.id, Act::Submit) == Some(TuiEvent::Send)
                && crate::event(&view, field.id, Act::Input { value: b"q".as_slice().into() })
                    == Some(TuiEvent::Typed { value: b"q".as_slice().into() });
        }
        let menu = tree.nodes().iter().find(|node| node.role == Role::Menu);
        let items: Vec<_> = tree.nodes().iter().filter(|node| node.role == Role::MenuItem).collect();
        if !commands.is_empty() {
            good &= menu.is_some_and(|node| {
                node.name.as_deref() == Some(b"Commands") && node.states.as_slice().contains(&State::Options)
            });
            good &= items.len() == commands.len();
            for (item, command) in items.iter().zip(commands) {
                good &= item.name.as_deref() == Some(command.name())
                    && item.styles.as_slice() == [look::COMMAND]
                    && item.description.as_deref()
                        == Some(if *command == Command::Exit {
                            b"Close temper"
                        } else {
                            shared::description(*command)
                        })
                    && crate::event(&view, item.id, Act::Press) == Some(TuiEvent::Command { command: *command });
            }
        } else if let Some(listing) = listing {
            good &= menu.is_some_and(|node| node.states.as_slice().contains(&State::Options));
            good &= tree.nodes().iter().any(|node| {
                node.role == Role::Text
                    && node.styles.as_slice() == [look::LIST_HEAD]
                    && f::text(tree, node).as_ref() == shared::list_head(listing)
            });
            let choices = match listing {
                Listing::Model => shared::models(&root, None, view.words().view()),
                Listing::Effort => shared::efforts(&root, None, view.words().view()),
                Listing::Chats => shared::chats(&root, view.words().view()),
            };
            good &= items.len() == choices.len();
            for (item, choice) in items.iter().zip(&choices) {
                good &= item.name.as_deref() == Some(&choice.name)
                    && item.description.as_deref() == Some(&choice.words)
                    && item.states.as_slice().contains(&State::Selected) == choice.selected
                    && crate::event(&view, item.id, Act::Press) == Some(TuiEvent::Act { act: choice.act.clone() });
            }
        } else {
            good &= menu.is_none();
        }
        good &= crate::escape(&view)
            == if listing.is_some() || !commands.is_empty() { Some(TuiEvent::Dismiss) } else { None };
        good &= !tree.nodes().iter().any(|node| node.shortcut.is_some());
        if !good {
            failures.push(name).unwrap();
        }
    }
    assert!(failures.is_empty(), "foot read rows: {failures:?}");
    read_states();
    quiet_states();
    rule_states();
    question_states();
    note_states();
}

#[expect(clippy::disallowed_methods, reason = "test assertions search and collect the rendered states and options")]
#[expect(clippy::disallowed_types, reason = "test assertions collect the confirmation options")]
fn read_states() {
    let words = f::words();
    let tui = Tui::new(f::ADDRESS);
    for (name, state, placeholder, paused, running) in [
        ("idle", TaskState::Idle, "Reply", false, false),
        ("ended", TaskState::Ended, "/new starts another chat", false, false),
        ("result", TaskState::Result, "/new starts another chat", false, false),
        ("paused", TaskState::Paused, "Reply", true, false),
        ("running", TaskState::Working, "Reply", false, true),
    ] {
        let root = root_state(true, true, true, false, false, state);
        let part = foot::read(&root, &tui, &words, f::NOW);
        assert_eq!(part.composer.as_ref().unwrap().placeholder.as_ref(), placeholder.as_bytes(), "{name}");
        assert_eq!(
            part.hint.as_ref(),
            if paused { "paused · a message resumes it".as_bytes() } else { b"/ for commands" },
            "{name}"
        );
        assert_eq!(part.running.is_some(), running, "{name}");
        let expected = part.running.clone();
        let view = draw(part);
        assert_eq!(view.tree().nodes().iter().any(|node| node.region == Some(look::WORKING)), running, "{name}");
        if let Some(expected) = expected {
            assert!(has(view.tree(), look::RUNNING, &expected), "{name}");
        }
    }
    for (link, wanted, style) in [
        (jig_client::Link::Offline, "● Offline", look::DOWN),
        (jig_client::Link::SignedOut, "● Signed out", look::DOWN),
    ] {
        let (mut root, watch) = f::live(Some(f::me(&[b"Hello"])));
        if link == jig_client::Link::Offline {
            f::client(
                &mut root,
                jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch, end: c::WatchEnd::Dropped } },
            );
        } else {
            root = f::signed_out();
        }
        let mut view = f::view();
        f::build(&mut view, &tui, &root);
        assert!(has(view.tree(), style, wanted.as_bytes()));
    }
    for (command, head, names) in [
        (Command::Close, "Close this chat? It stays in Chats.", ["Close", "Keep it open"]),
        (
            Command::StartOver,
            "Start over? A new Ask temper chat starts. This one stays in Chats.",
            ["Start over", "Keep this one"],
        ),
    ] {
        let mut root = root(true, true, true, false, true);
        act(&mut root, d::pages::Act::Terminal(d::pages::terminal::Act::Run { command }));
        let part = foot::read(&root, &tui, &words, f::NOW);
        assert_eq!(part.list.as_ref().unwrap().head.as_deref(), Some(head.as_bytes()));
        let view = draw(part);
        let items: Vec<_> = view.tree().nodes().iter().filter(|node| node.role == Role::MenuItem).collect();
        assert_eq!(items.len(), 2);
        for (item, (name, event)) in
            items.iter().zip(names.into_iter().zip([d::pages::Act::Confirm, d::pages::Act::Cancel]))
        {
            assert_eq!(item.name.as_deref(), Some(name.as_bytes()));
            assert_eq!(crate::event(&view, item.id, Act::Press), Some(TuiEvent::Act { act: event }));
        }
        assert_eq!(crate::escape(&view), Some(TuiEvent::Act { act: d::pages::Act::Cancel }));
    }
    // The first connection's composer must not consume the sign-in field's focus request.
    for (name, initial, link, local, next) in [
        (
            "sign-in after starting",
            Box::new(f::starting().0),
            jig_client::Link::Starting,
            crate::Local::Idle,
            crate::Local::Idle,
        ),
        (
            "sign-in after engine-link fallback",
            Box::new(f::signed_out()),
            jig_client::Link::SignedOut,
            crate::Local::Signing,
            crate::Local::Ended,
        ),
    ] {
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Link { link });
        tui.step(Told::Local { local });
        let mut view = f::view();
        let size = skein_ui_screen::Size { columns: 80, rows: 24 };
        let mut focus = skein_ui_screen::Focus::default();
        let mut frame = skein_ui_screen::Frame::new(size);
        f::build(&mut view, &tui, &initial);
        skein_ui_screen::render(view.tree(), &crate::theme(Scheme::Dark, None), size, &mut focus, &mut frame);
        tui.step(Told::Link { link: jig_client::Link::SignedOut });
        tui.step(Told::Local { local: next });
        f::build(&mut view, &tui, &f::signed_out());
        skein_ui_screen::render(view.tree(), &crate::theme(Scheme::Dark, None), size, &mut focus, &mut frame);
        assert_eq!(focus.node, Some(f::find(view.tree(), Role::TextField, b"Link").id), "{name}");
    }
}
#[expect(clippy::disallowed_methods, reason = "test assertions search and collect the quiet line's nodes")]
#[expect(clippy::disallowed_types, reason = "test assertions collect the signal nodes")]
fn quiet_states() {
    let root = root(true, true, false, false, false);
    let words = f::words();
    let tui = Tui::new(f::ADDRESS);
    for (name, count, cut, columns, line, closed, expected) in [
        ("zero", 0, false, 0, "Claude Opus 5.5 · high", true, ""),
        ("one", 1, false, 0, "Claude Opus 5.5 · high", true, "1 needs you"),
        ("three", 3, false, 0, "Claude Opus 5.5 · high", true, "3 need you"),
        ("cut", 3, true, 0, "Claude Opus 5.5 · high", true, "3+ need you"),
        ("narrow", 1, false, 40, "Claude Opus 5.5 · high", false, "1 needs you"),
        ("fits exactly", 0, false, 49, "Claude Opus 5.5 · high", true, ""),
        ("one column short", 0, false, 48, "Claude Opus 5.5 · high", false, ""),
        ("wide glyph fits", 0, false, 29, "界", true, ""),
        ("wide glyph short", 0, false, 28, "界", false, ""),
    ] {
        let mut part = foot::read(&root, &tui, &words, f::NOW);
        part.needs = d::pages::needs::Count { count, cut };
        part.columns = columns;
        part.closed = Some(line.as_bytes().into());
        let view = draw(part);
        let signals: Vec<_> =
            view.tree().nodes().iter().filter(|node| node.styles.as_slice() == [look::SIGNAL]).collect();
        assert_eq!(signals.len(), usize::from(!expected.is_empty()), "{name}");
        if let Some(signal) = signals.first() {
            assert_eq!(f::text(view.tree(), signal).as_ref(), expected.as_bytes(), "{name}");
        }
        assert_eq!(has(view.tree(), look::HINT, line.as_bytes()), closed, "{name}");
        let bottom = view.tree().nodes().iter().find(|node| node.region == Some(look::BOTTOM)).unwrap();
        let start = f::index(view.tree(), bottom.id);
        let mut next = start.checked_add(1).unwrap();
        let mut children = 0_usize;
        for index in start.checked_add(1).unwrap()..start.checked_add(usize::try_from(bottom.size).unwrap()).unwrap() {
            if index == next {
                children = children.checked_add(1).unwrap();
                next = index.checked_add(usize::try_from(view.tree().nodes()[index].size).unwrap()).unwrap();
            }
        }
        assert_eq!(children, 2, "{name}");
    }
    let no_picker = root_state(true, false, false, false, false, TaskState::Idle);
    assert_eq!(
        foot::read(&no_picker, &tui, &words, f::NOW).closed.is_some(),
        no_picker.pages().choice().context.is_some()
    );
    let mut part = foot::read(&root, &tui, &words, f::NOW);
    part.composer.as_mut().unwrap().refusal = Some(b"This message was refused.".as_slice().into());
    assert!(has(draw(part).tree(), look::REFUSED, b"This message was refused."));
}
#[expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "rule rows compare semantic parts and rendered lines"
)]
fn rule_states() {
    use skein_ui_screen::{Focus, Frame, Size, render};
    let words = f::words();
    let title = "What can you tell…";
    let all = "Hello / What can you tell… T2 on box:40691";
    for (columns, expected) in [
        (80, all),
        (48, all),
        (47, "Hello / What can you tell… T2"),
        (35, "Hello / What can you tell… T2"),
        (34, "What can you tell… T2"),
        (27, "What can you tell… T2"),
        (26, ""),
        (0, all),
    ] {
        for list in [false, true] {
            let mut root = root(true, true, true, false, false);
            if list {
                act(&mut root, d::pages::Act::MessageWords { words: b"/".as_slice().into() });
            }
            let mut tui = Tui::new(b"127.0.0.1:40691");
            tui.step(Told::Size { columns });
            tui.step(Told::Boxed);
            let mut part = foot::read(&root, &tui, &words, f::NOW);
            let read = part.rule.as_ref().unwrap();
            assert_eq!(read.project.as_ref(), b"Demo");
            assert_eq!(read.place.as_ref(), b"box:40691");
            assert_eq!(read.task, 9);
            assert_eq!(part.list.is_some(), list);
            part.rule = Some(foot::Rule {
                project: b"Hello".as_slice().into(),
                title: title.as_bytes().into(),
                task: 2,
                place: b"box:40691".as_slice().into(),
            });
            let view = draw(part);
            let tree = view.tree();
            let region = tree.nodes().iter().find(|node| node.region == Some(look::BOX)).unwrap();
            let at = f::index(tree, region.id);
            let heading_at = at.checked_add(1).unwrap();
            let heading = &tree.nodes()[heading_at];
            assert_eq!(heading.role, Role::Heading { level: 1 });
            let parts = &tree.nodes()[heading_at.checked_add(1).unwrap()
                ..heading_at.checked_add(usize::try_from(heading.size).unwrap()).unwrap()];
            assert!(
                parts
                    .iter()
                    .filter(|node| !node.styles.is_empty())
                    .all(|node| node.styles.as_slice().contains(&look::RULE_TITLE))
            );
            let paints: Vec<_> =
                parts.iter().filter(|node| !node.styles.is_empty()).map(|node| node.styles.as_slice()[0]).collect();
            let semantic: &[skein_ui::Style] = if columns == 0 || columns >= 48 {
                &[look::NAME, look::FAINT, look::NAME, look::HINT, look::HINT, look::ADDRESS]
            } else if columns >= 35 {
                &[look::NAME, look::FAINT, look::NAME, look::HINT]
            } else {
                &[look::NAME, look::HINT]
            };
            assert_eq!(paints, semantic, "columns {columns}, list {list}");
            for scheme in [Scheme::Dark, Scheme::Light] {
                let size = Size { columns: if columns == 0 { 80 } else { columns }, rows: 24 };
                let mut frame = Frame::new(size);
                render(tree, &crate::theme(scheme, None), size, &mut Focus::default(), &mut frame);
                let lines = String::from_utf8(frame.print().into_vec()).unwrap();
                let rule = lines.lines().find(|line| line.starts_with('─')).unwrap();
                let actual = rule.trim_matches('─').trim();
                assert_eq!(actual, expected, "columns {columns}, list {list}, {scheme:?}");
                if !expected.is_empty() {
                    assert!(rule.ends_with(" ──"), "{rule}");
                }
            }
        }
    }
    let root = root(true, true, true, false, false);
    let part = foot::read(&root, &Tui::new(b"127.0.0.1:40691"), &words, f::NOW);
    assert_eq!(part.rule.unwrap().place.as_ref(), b"127.0.0.1:40691");
}
fn question(primary: Option<usize>, empty: bool) -> shared::Question {
    shared::Question {
        object: c::ObjectKey::Task(9),
        head: b"Answer csv-export's question with \"Comma\"?".as_slice().into(),
        choices: if empty {
            Box::default()
        } else {
            Box::new([
                shared::QuestionChoice { words: b"Answer Semicolon".as_slice().into(), recommended: false },
                shared::QuestionChoice { words: b"Answer Comma".as_slice().into(), recommended: true },
            ])
        },
        primary,
    }
}
#[expect(clippy::disallowed_methods, reason = "test assertions collect the question's rendered options")]
#[expect(clippy::disallowed_types, reason = "test assertions collect the question's rendered options")]
fn question_states() {
    let root = root(true, true, true, false, false);
    let words = f::words();
    let tui = Tui::new(f::ADDRESS);
    for (primary, empty, names, selected) in [
        (Some(1), false, &["1. Answer Comma", "2. Answer Semicolon", "3. Tell it something else"][..], Some(0)),
        (None, false, &["1. Answer Semicolon", "2. Answer Comma", "3. Tell it something else"][..], None),
        (None, true, &["1. Tell it something else"][..], None),
    ] {
        let mut part = foot::read(&root, &tui, &words, f::NOW);
        let title = part.rule.as_ref().unwrap().title.clone();
        part.list = foot::lists::question_list(question(primary, empty), None, &words);
        let view = draw(part);
        let field = f::find(view.tree(), Role::TextArea, b"Message");
        assert_eq!(field.placeholder, None);
        assert!(view.tree().nodes().iter().any(|node| node.styles.as_slice().contains(&look::RULE_TITLE)
            && f::text(view.tree(), node).as_ref() == title.as_ref()));
        let items: Vec<_> = view.tree().nodes().iter().filter(|node| node.role == Role::MenuItem).collect();
        assert_eq!(items.len(), names.len());
        for (index, (item, name)) in items.iter().zip(names).enumerate() {
            assert_eq!(item.name.as_deref(), Some(name.as_bytes()));
            assert_eq!(item.states.as_slice().contains(&State::Selected), Some(index) == selected);
            let event = if index.checked_add(1).unwrap() == names.len() {
                TuiEvent::Aside { object: c::ObjectKey::Task(9) }
            } else {
                TuiEvent::Act {
                    act: d::pages::Act::Needs(d::pages::needs::Act::Choose {
                        object: c::ObjectKey::Task(9),
                        words: name.as_bytes()[3..].into(),
                    }),
                }
            };
            assert_eq!(crate::event(&view, item.id, Act::Press), Some(event));
        }
        assert_eq!(crate::escape(&view), Some(TuiEvent::Aside { object: c::ObjectKey::Task(9) }));
    }
    assert!(foot::lists::question_list(question(Some(1), false), Some(c::ObjectKey::Task(9)), &words).is_none());
    assert!(foot::lists::question_list(question(Some(1), false), Some(c::ObjectKey::Task(10)), &words).is_some());
}
#[expect(clippy::disallowed_methods, reason = "test assertions filter the rendered note paragraphs")]
fn note_states() {
    let mut root = root(true, true, false, false, false);
    let words = f::words();
    let mut tui = Tui::new(f::ADDRESS);
    act(&mut root, d::pages::Act::Terminal(d::pages::terminal::Act::Run { command: Command::Help }));
    act(&mut root, d::pages::Act::Terminal(d::pages::terminal::Act::Font));
    tui.step(Told::Unknown { words: Some(b"/font".as_slice().into()) });
    let notes: foot::Notes = foot::read_notes(&root, &tui, &words);
    let want = [
        "/model   Change the model, from the next turn",
        "/effort  Change the effort",
        "/new     Start a new chat",
        "/chats   Open another chat",
        "/theme   Dark or light",
        "/help    Commands and keys",
        "/exit    Close temper",
        "Enter sends",
        "Tab completes a command",
        "PgUp and PgDn read back",
        "Esc closes a list",
        "Ctrl+C closes temper",
        "Font: IBM Plex Mono.",
        "Unknown command /font. /help lists them.",
    ];
    assert_eq!(notes.lines.len(), want.len());
    let bounds = crate::limits(&f::limits(), &skein_ui::Limits::EMPTY, 512).unwrap();
    let mut render = skein_ui::View::new(&bounds);
    let mut b = render.builder();
    b.open(Role::Page);
    foot::draw_notes(&mut b, &notes, &words);
    b.close();
    render.finish(b, 1, &mut Queue::uncapped());
    for (line, want) in notes.lines.iter().zip(want) {
        assert_eq!(line.as_ref(), want.as_bytes());
    }
    for (node, want) in render.tree().nodes().iter().filter(|node| node.role == Role::Paragraph).zip(want) {
        assert_eq!(node.styles.as_slice(), &[look::NOTE]);
        assert_eq!(f::text(render.tree(), node).as_ref(), want.as_bytes());
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the golden table embeds its hand-written frame oracles")]
fn the_foots_are_their_goldens() {
    use skein_ui_screen::{Focus, Size};
    let full = Size { columns: 80, rows: 24 };
    let rows = [
        (
            "foot-commands-80x24-dark",
            Scheme::Dark,
            full,
            1_u8,
            include_bytes!("golden/foot-commands-80x24-dark.txt").as_slice(),
        ),
        (
            "foot-question-80x24-dark",
            Scheme::Dark,
            full,
            2,
            include_bytes!("golden/foot-question-80x24-dark.txt").as_slice(),
        ),
    ];
    let mut views = List::with_capacity(6);
    let words = f::words();
    for (_, _, size, kind, _) in &rows {
        let mut root = root(true, true, *kind == 2, false, false);
        if *kind == 1 {
            act(&mut root, d::pages::Act::ChatWords { words: Box::from(*b"/") });
        }
        let mut part = foot::read(&root, &Tui::new(f::ADDRESS), &words, f::NOW);
        part.closed = Some("Claude Opus 5.5 · high".as_bytes().into());
        part.needs = d::pages::needs::Count { count: 1, cut: false };
        part.columns = size.columns;
        if *kind == 2 {
            part.rule = Some(foot::Rule {
                project: b"Hello".as_slice().into(),
                title: "What landed…".as_bytes().into(),
                task: 2,
                place: b"box".as_slice().into(),
            });
            part.list = foot::lists::question_list(question(Some(1), false), None, &words);
        }
        views.push(draw(part)).unwrap();
    }
    let mut checks = List::with_capacity(6);
    for ((name, scheme, size, _, want), view) in rows.iter().zip(&views) {
        checks
            .push(goldens::Golden {
                background: Some(crate::theme::palette(*scheme)[0].1),
                name,
                tree: view.tree(),
                scheme: *scheme,
                size: *size,
                focus: Focus::default(),
                want,
            })
            .unwrap();
    }
    goldens::check(checks.as_slice());
}

#[test]
fn the_launchpad_effort_list_changes_homes_choice() {
    let mut root = root(true, true, false, false, false);
    let mut view = f::view();
    let tui = Tui::new(f::ADDRESS);
    let me = root.pages().me(root.client()).unwrap();
    assert_eq!(root.pages().choice().chosen(me).unwrap().main.effort.as_ref().unwrap().bytes(), b"high");
    act(&mut root, d::pages::Act::ChatWords { words: b"/effort".as_slice().into() });
    f::build(&mut view, &tui, &root);
    let item = f::find(view.tree(), Role::MenuItem, b"/effort");
    assert_eq!(crate::event(&view, item.id, Act::Press), Some(TuiEvent::Command { command: Command::Effort }));
    act(&mut root, d::pages::Act::Terminal(d::pages::terminal::Act::Run { command: Command::Effort }));
    f::build(&mut view, &tui, &root);
    let head = b"Effort. A change applies from the next turn.";
    let heading = f::find(view.tree(), Role::Text, head);
    assert_eq!(heading.styles.as_slice(), [look::LIST_HEAD]);
    let field = f::find(view.tree(), Role::TextArea, b"Ask temper");
    assert!(f::index(view.tree(), field.id) < f::index(view.tree(), heading.id));
    for (name, index) in [(b"low".as_slice(), 0), (b"high".as_slice(), 1)] {
        let item = f::find(view.tree(), Role::MenuItem, name);
        assert_eq!(
            crate::event(&view, item.id, Act::Press),
            Some(TuiEvent::Act { act: d::pages::Act::Choice(d::pages::choice::Act::Effort { index }) })
        );
    }
    let item = f::find(view.tree(), Role::MenuItem, b"low");
    let Some(TuiEvent::Act { act: action }) = crate::event(&view, item.id, Act::Press) else {
        unreachable!("effort option carries the root's act");
    };
    act(&mut root, action);
    let me = root.pages().me(root.client()).unwrap();
    assert_eq!(root.pages().choice().chosen(me).unwrap().main.effort.as_ref().unwrap().bytes(), b"low");
    assert!(root.pages().chat().is_none());
}
