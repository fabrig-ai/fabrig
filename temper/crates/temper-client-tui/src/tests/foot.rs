//! The foot's states and hand-written frames (client/tui/pages.md and client/tui/theme.md).
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
pub(super) fn root_state(
    project: bool,
    picker: bool,
    chat: bool,
    delegate: bool,
    draft: bool,
    state: TaskState,
) -> Domain {
    let mut me = f::me(if project { &[b"Demo"] } else { &[] });
    if let Some(project) = me.projects.get_mut(0) {
        project.templates = List::from_boxed(Box::new([template(1), template(5), {
            let mut draft = template(6);
            draft.executor = c::TemplateExecutor::Charter(12);
            draft
        }]));
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
    let (mut root, me_watch) = f::starting();
    f::client(&mut root, jig_client::Event::Up { up: jig_client::Up::Served { served: true, versions: None } });
    let mut out = f::client(
        &mut root,
        jig_client::Event::Up {
            up: jig_client::Up::Watched {
                watch: me_watch,
                event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
            },
        },
    );
    let mut looked = f::event(
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
    while let Some(request) = looked.pop() {
        out.push(request);
    }
    let address = if chat {
        Address::Chat { task: 9 }
    } else if project {
        Address::Home { project: 1 }
    } else {
        Address::Inbox { filter: d::address::Filter::ALL }
    };
    let mut went = f::event(&mut root, Event::Went { place: Place::Address(address) });
    while let Some(request) = went.pop() {
        out.push(request);
    }
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
    if !chat {
        while let Some(request) = out.pop() {
            if let d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } },
                    },
            } = request
                && f::ways_read(&record.bytes)
            {
                f::client(
                    &mut root,
                    jig_client::Event::Up {
                        up: jig_client::Up::ReadDone {
                            read,
                            result: jig_client::Readout::Family {
                                family: temper_client_protocol_wire::FORGE,
                                record: Some(f::draft_setting(true)),
                            },
                        },
                    },
                );
            }
        }
    }
    if state == TaskState::SessionOffline {
        f::client(
            &mut root,
            jig_client::Event::Up { up: jig_client::Up::WatchEnded { watch: me_watch, end: c::WatchEnd::Dropped } },
        );
    }
    root
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TaskState {
    Idle,
    Session,
    SessionOffline,
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
        TaskState::Session | TaskState::SessionOffline => {
            task.chip.executor = c::ChipExecutor::Agent { charter: 12 };
        }
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
            pane: None,
            to_end: 0,
            entries: Box::default(),
            notes: foot::read_notes(&root, &Tui::new(f::ADDRESS), &f::words()),
        })
    } else {
        let mut launchpad = view::launchpad::read(&root, &Tui::new(f::ADDRESS), &f::words(), f::NOW);
        launchpad.drafts = true;
        view::Body::Launchpad(launchpad)
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
        for active in [b"/draft".as_slice(), b"/new"] {
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
    assert_eq!(commands, 11);
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
        ("launchpad", true, false, "", None, Some("Ask for a change, or ask temper.  / for commands"), &[][..]),
        ("no project", false, false, "", None, Some("/ for commands"), &[][..]),
        ("chat", true, true, "", None, Some("Reply"), &[][..]),
        ("chat prefix", true, true, "/c", None, None, &[Command::Close, Command::Chats][..]),
        ("unknown", true, false, "/zz", None, Some("Ask for a change, or ask temper.  / for commands"), &[][..]),
        ("whitespace", true, false, "/new ", None, Some("Ask for a change, or ask temper.  / for commands"), &[][..]),
        (
            "unicode whitespace",
            true,
            false,
            "/new\u{a0}",
            None,
            Some("Ask for a change, or ask temper.  / for commands"),
            &[][..],
        ),
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
                good &= item.name.as_deref() == Some(crate::command_name(*command))
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
    goal_states();
    notice_states();
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
        assert_eq!(part.paused, paused, "{name}");
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
fn goal_states() {
    let words = f::words();
    for (name, ways, chosen, funds, reserved, send, expected, refusal) in [
        ("preferred first", true, None, 100, 0, false, &[4, 2, 8][..], None),
        ("chosen first", true, Some(8), 100, 0, false, &[8, 2, 4][..], None),
        ("nothing to fund it", true, None, 0, 0, false, &[4, 2, 8][..],
            Some(b"Nothing is left in your pool to fund a goal.".as_slice())),
        ("held by chats", true, None, 0, 100, false, &[4, 2, 8][..],
            Some("Nothing is left in your pool to fund a goal: your open chats hold it. Close one you’ve finished with, and you can start this.".as_bytes())),
        ("refused", true, None, 100, 0, true, &[4, 2, 8][..], Some(b"Limit reached".as_slice())),
        ("no ways", false, None, 100, 0, false, &[][..], None),
    ] {
        let mut root = f::GoalCase { ways, funds, reserved, ..f::GoalCase::READY }.root();
        open_goal(&mut root);
        if let Some(template) = chosen { act(&mut root, d::pages::Act::StartWay { template }); }
        if send { act(&mut root, d::pages::Act::StartGoal); }
        let part = foot::read(&root, &Tui::new(f::ADDRESS), &words, f::NOW);
        let list = part.list.as_ref().unwrap();
        let head = b"Start a goal from plan.md. How should it run?";
        assert_eq!(list.head.as_deref(), Some(head.as_slice()), "{name}");
        assert_eq!(list.items.len(), expected.len().checked_add(1).unwrap(), "{name}");
        for (index, (item, template)) in list.items.iter().zip(expected).enumerate() {
            let (way, description) = match template {
                2 => ("Task by task", "The same plan; each task lands on its own once checked."),
                4 => ("Plan, tasks, land whole", "A plan from the draft, its tasks on a goal branch, landed as one change once all are checked."),
                8 => ("Custom", ""),
                _ => unreachable!(),
            };
            let number = skein_lib::Decimal::of(u64::try_from(index).unwrap().checked_add(1).unwrap());
            assert_eq!(item.name, skein_lib::bytes::concat(&[number.as_bytes(), b". ", way.as_bytes()]), "{name}");
            assert_eq!(item.words.as_ref(), description.as_bytes(), "{name}");
            assert_eq!(item.selected, index == 0, "{name}");
            assert_eq!(item.event, TuiEvent::Way { template: *template }, "{name}");
        }
        let cancel = list.items.last().unwrap();
        assert_eq!(cancel.name.as_ref(), b"Cancel");
        assert_eq!(cancel.event, TuiEvent::Act { act: d::pages::Act::StartCancel });
        assert_eq!(list.escape, cancel.event);
        assert_eq!(part.composer.as_ref().unwrap().refusal.as_deref(), refusal, "{name}");
        assert!(root.pages().start().is_some(), "{name}: the sheet stays");
        let view = draw(part);
        assert!(has(view.tree(), look::LIST_HEAD, head));
        assert_eq!(crate::escape(&view), Some(cancel_goal()));
        for item in view.tree().nodes() {
            if item.role == Role::MenuItem {
                assert_eq!(item.styles.as_slice(), &[look::OPTION]);
            }
        }
        if let Some(refusal) = refusal { assert!(has(view.tree(), look::REFUSED, refusal), "{name}"); }
    }
}
fn cancel_goal() -> TuiEvent {
    TuiEvent::Act { act: d::pages::Act::StartCancel }
}
pub(super) fn open_goal(root: &mut Domain) {
    act(root, d::pages::Act::OpenStart { draft: Some(b"docs/design/draft/plan.md".as_slice().into()), chat: Some(9) });
    assert!(root.pages().start().is_some());
}

#[expect(clippy::disallowed_methods, reason = "notice rows inspect the quiet line's two children")]
fn notice_states() {
    let words = f::words();
    let notice = crate::Notice::Unknown { words: b"/qiut".as_slice().into() };
    let phrase = "Unknown command /qiut · /help lists them";
    for (name, chat, state, box_state, playground, count, columns, left, right) in [
        ("box", false, TaskState::Idle, true, false, 0, 110, "Demo · model · high · ● Live", phrase),
        ("playground", false, TaskState::Idle, false, true, 0, 110, "Demo · model · high · ● Live", phrase),
        ("paused", true, TaskState::Paused, false, false, 0, 110, "Demo · model · high · ● Live", phrase),
        ("needs and walk", false, TaskState::Idle, false, false, 1, 110, "Demo · model · high · ● Live", phrase),
        ("fits exactly", false, TaskState::Idle, false, false, 0, 72, "Demo · model · high · ● Live", phrase),
        (
            "one short",
            false,
            TaskState::Idle,
            false,
            false,
            0,
            71,
            "Unknown command /qiut · /help lists them · Demo · model · high · ● Live",
            "",
        ),
        (
            "narrow",
            false,
            TaskState::Idle,
            true,
            true,
            1,
            40,
            "Unknown command /qiut · /help lists them · Demo · model · high · ● Live",
            "",
        ),
    ] {
        let root = root_state(true, true, chat, false, false, state);
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Notice { notice: Some(notice.clone()) });
        if box_state {
            tui.step(Told::Front);
            tui.step(Told::BoxLine { line: crate::BoxLine::Restarting { step: b"forgejo-3".as_slice().into() } });
        }
        let mut part = foot::read(&root, &tui, &words, f::NOW);
        part.playground = playground;
        part.needs = d::pages::needs::Count { count, cut: false };
        part.columns = columns;
        assert_eq!(part.notice.as_deref(), Some(phrase.as_bytes()), "{name}");
        assert!(foot::read_notes(&root, &tui, &words).lines.is_empty(), "{name}: notice stays out of notes");
        let view = draw(part);
        let tree = view.tree();
        let bottom = tree.nodes().iter().find(|node| node.region == Some(look::BOTTOM)).unwrap();
        let first = &tree.nodes()[f::index(tree, bottom.id).checked_add(1).unwrap()];
        let second = &tree.nodes()[f::index(tree, first.id).checked_add(usize::try_from(first.size).unwrap()).unwrap()];
        assert_eq!(first.role, Role::Status);
        assert_eq!(f::text(tree, first).as_ref(), left.as_bytes(), "{name}");

        assert_eq!(f::text(tree, second).as_ref(), right.as_bytes(), "{name}");
        assert!(has(tree, look::SIGNAL, phrase.as_bytes()), "{name}");
        assert!(!has(tree, look::SIGNAL, b"forgejo-3 restarting"), "{name}");
        assert!(!has(tree, look::HINT, b"playground"), "{name}");
        assert!(!has(tree, look::HINT, b"paused \xc2\xb7 a message resumes it"), "{name}");
        assert!(!has(tree, look::HINT, b"n to walk"), "{name}");
        assert!(!has(tree, look::SIGNAL, b"1 needs you"), "{name}");
    }
    let mut tui = Tui::new(f::ADDRESS);
    tui.step(Told::Notice { notice: Some(crate::Notice::SigningIn) });
    let root = root_state(true, true, false, false, false, TaskState::Idle);
    let part = foot::read(&root, &tui, &words, f::NOW);
    assert_eq!(part.notice.as_deref(), Some(b"Still signing in".as_slice()));
    assert!(has(draw(part).tree(), look::SIGNAL, b"Still signing in"));
    let part = foot::read(&f::signed_out(), &tui, &words, f::NOW);
    assert!(!has(draw(part).tree(), look::SIGNAL, b"Still signing in"));
    tui.step(Told::Notice { notice: Some(crate::Notice::Proposed) });
    let part = foot::read(&root, &tui, &words, f::NOW);
    let proposed = b"Proposed: the goal starts once it is accepted.";
    assert_eq!(part.notice.as_deref(), Some(proposed.as_slice()));
    assert!(foot::read_notes(&root, &tui, &words).lines.is_empty());
    assert!(has(draw(part).tree(), look::SIGNAL, proposed));
}
#[expect(clippy::disallowed_methods, clippy::disallowed_types, reason = "quiet rows inspect their rendered parts")]
#[expect(clippy::too_many_lines, reason = "one table checks quiet-line parts and width boundaries")]
fn quiet_states() {
    let root = root(true, true, false, false, false);
    let words = f::words();
    let tui = Tui::new(f::ADDRESS);
    for (name, count, cut, columns, line, closed, expected) in [
        ("zero", 0, false, 0, "Claude Opus 5.5 · high", true, ""),
        ("one", 1, false, 0, "Claude Opus 5.5 · high", true, "1 needs you"),
        ("three", 3, false, 0, "Claude Opus 5.5 · high", true, "3 need you"),
        ("cut", 3, true, 0, "Claude Opus 5.5 · high", true, "3+ need you"),
        ("medium", 1, false, 60, "Claude Opus 5.5 · high", true, "1 needs you"),
        ("narrow", 1, false, 40, "Claude Opus 5.5 · high", false, "1 needs you"),
        ("fits exactly", 0, false, 42, "Claude Opus 5.5 · high", true, ""),
        ("one column short", 0, false, 41, "Claude Opus 5.5 · high", false, ""),
        ("wide glyph fits", 0, false, 22, "界", true, ""),
        ("wide glyph short", 0, false, 21, "界", false, ""),
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
        assert_eq!(has(view.tree(), look::HINT, b"n to walk"), count > 0 && columns == 0, "{name}");
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
    let part = foot::read(&root, &tui, &words, f::NOW);
    assert_eq!(
        part.composer.as_ref().unwrap().placeholder.as_ref(),
        b"Ask for a change, or ask temper.  / for commands"
    );
    let view = draw(part);
    let bottom = view.tree().nodes().iter().find(|node| node.region == Some(look::BOTTOM)).unwrap();
    assert_eq!(f::text(view.tree(), bottom).as_ref(), "Demo · model · high · ● Live".as_bytes());
    for (name, project, picker, chat, task, columns, left, right) in [
        (
            "launchpad",
            true,
            true,
            false,
            TaskState::Idle,
            80,
            "Demo · Claude Opus 5.5 · high · ● Live",
            "1 needs you · n to walk",
        ),
        ("medium", true, true, false, TaskState::Idle, 60, "Demo · Claude Opus 5.5 · high · ● Live", "1 needs you"),
        ("narrow", true, true, false, TaskState::Idle, 40, "Demo · ● Live", "1 needs you"),
        ("session", true, true, true, TaskState::Session, 80, "Demo · drafter · Claude Opus 5.5 · high", "1 needs you"),
        ("session narrow", true, true, true, TaskState::Session, 30, "Demo · drafter", "1 needs you"),
        (
            "session offline",
            true,
            true,
            true,
            TaskState::SessionOffline,
            110,
            "Demo · drafter · Claude Opus 5.5 · high · ● Offline",
            "1 needs you",
        ),
        ("chat", true, true, true, TaskState::Idle, 80, "Demo · Claude Opus 5.5 · high · ● Live", "1 needs you"),
        (
            "paused",
            true,
            true,
            true,
            TaskState::Paused,
            110,
            "Demo · Claude Opus 5.5 · high · ● Live",
            "paused · a message resumes it · 1 needs you",
        ),
        ("no picker", true, false, false, TaskState::Idle, 80, "Demo · ● Live", "1 needs you · n to walk"),
        ("no project", false, false, false, TaskState::Idle, 80, "● Live", ""),
    ] {
        let root = root_state(project, picker, chat, false, false, task);
        let mut part = foot::read(&root, &tui, &words, f::NOW);
        part.columns = columns;
        part.closed = if picker { Some("Claude Opus 5.5 · high".as_bytes().into()) } else { None };
        part.needs = d::pages::needs::Count { count: u32::from(project), cut: false };
        let view = draw(part);
        let tree = view.tree();
        let bottom = tree.nodes().iter().find(|node| node.region == Some(look::BOTTOM)).unwrap();
        let first = &tree.nodes()[f::index(tree, bottom.id).checked_add(1).unwrap()];
        let second = &tree.nodes()[f::index(tree, first.id).checked_add(usize::try_from(first.size).unwrap()).unwrap()];
        assert_eq!(f::text(tree, first).as_ref(), left.as_bytes(), "{name}");
        assert_eq!(
            has(tree, look::HINT, b"drafter"),
            task == TaskState::Session || task == TaskState::SessionOffline,
            "{name}"
        );
        assert_eq!(f::text(tree, second).as_ref(), right.as_bytes(), "{name}");
    }
    for (name, mode, columns, paused, left, right) in [
        (
            "folded",
            d::pages::chat::document::Mode::Chat,
            110,
            false,
            "Demo · model · high · ● Live",
            "pane: folded · Ctrl-P opens · 1 needs you",
        ),
        (
            "half",
            d::pages::chat::document::Mode::Split,
            110,
            false,
            "Demo · model · high · ● Live",
            "pane: half · Ctrl-P widens · 1 needs you",
        ),
        (
            "all",
            d::pages::chat::document::Mode::Draft,
            110,
            false,
            "Demo · model · high · ● Live",
            "pane: all · Ctrl-P folds · 1 needs you",
        ),
        (
            "paused pane",
            d::pages::chat::document::Mode::Split,
            150,
            true,
            "Demo · model · high · ● Live",
            "paused · a message resumes it · pane: half · Ctrl-P widens · 1 needs you",
        ),
        (
            "drop key before model",
            d::pages::chat::document::Mode::Split,
            65,
            false,
            "Demo · model · high · ● Live",
            "pane: half · 1 needs you",
        ),
        (
            "drop model next",
            d::pages::chat::document::Mode::Split,
            40,
            false,
            "Demo · ● Live",
            "pane: half · 1 needs you",
        ),
    ] {
        let mut part = foot::read(&root, &tui, &words, f::NOW);
        part.pane = Some(mode);
        part.launchpad = false;
        part.columns = columns;
        part.paused = paused;
        part.needs = d::pages::needs::Count { count: 1, cut: false };
        let view = draw(part);
        let tree = view.tree();
        let bottom = tree.nodes().iter().find(|node| node.region == Some(look::BOTTOM)).unwrap();
        let first = &tree.nodes()[f::index(tree, bottom.id).checked_add(1).unwrap()];
        let second = &tree.nodes()[f::index(tree, first.id).checked_add(usize::try_from(first.size).unwrap()).unwrap()];
        assert_eq!(f::text(tree, first).as_ref(), left.as_bytes(), "{name}");
        assert_eq!(f::text(tree, second).as_ref(), right.as_bytes(), "{name}");
        assert_eq!(
            tree.nodes()
                .iter()
                .filter(
                    |node| node.styles.as_slice() == [look::HINT] && f::text(tree, node).as_ref() == " · ".as_bytes()
                )
                .count(),
            usize::from(columns > 65),
            "{name}: pane separator is a hint"
        );
    }
    for (name, case, columns, count, right, goal) in [
        (
            "goal offered",
            f::GoalCase::READY,
            110,
            1,
            "pane: half · Ctrl-P widens · /goal starts a goal from the draft · 1 needs you",
            Some("/goal starts a goal from the draft"),
        ),
        (
            "linked goal",
            f::GoalCase { linked: true, ..f::GoalCase::READY },
            110,
            0,
            "pane: half · Ctrl-P widens · goal Farewell line for… · /open opens it",
            Some("goal Farewell line for… · /open opens it"),
        ),
        (
            "goal fits exactly",
            f::GoalCase::READY,
            96,
            0,
            "pane: half · Ctrl-P widens · /goal starts a goal from the draft",
            Some("/goal starts a goal from the draft"),
        ),
        ("goal one column short", f::GoalCase::READY, 95, 0, "pane: half · Ctrl-P widens", None),
        (
            "linked goal fits exactly",
            f::GoalCase { linked: true, ..f::GoalCase::READY },
            102,
            0,
            "pane: half · Ctrl-P widens · goal Farewell line for… · /open opens it",
            Some("goal Farewell line for… · /open opens it"),
        ),
        (
            "linked goal one column short",
            f::GoalCase { linked: true, ..f::GoalCase::READY },
            101,
            0,
            "pane: half · Ctrl-P widens",
            None,
        ),
        ("goal at eighty", f::GoalCase::READY, 80, 0, "pane: half · Ctrl-P widens", None),
        (
            "linked goal at eighty",
            f::GoalCase { linked: true, ..f::GoalCase::READY },
            80,
            0,
            "pane: half · Ctrl-P widens",
            None,
        ),
        (
            "neither offered nor linked",
            f::GoalCase { body: Some(b"Words\n"), ..f::GoalCase::READY },
            110,
            0,
            "pane: half · Ctrl-P widens",
            None,
        ),
        ("no pane", f::GoalCase { body: None, linked: true, ..f::GoalCase::READY }, 110, 0, "", None),
    ] {
        let root = case.root();
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Size { columns });
        let mut part = foot::read(&root, &tui, &words, f::NOW);
        part.closed = Some("model · high".as_bytes().into());
        part.needs = d::pages::needs::Count { count, cut: false };
        let view = draw(part);
        let tree = view.tree();
        let bottom = tree.nodes().iter().find(|node| node.region == Some(look::BOTTOM)).unwrap();
        let first = &tree.nodes()[f::index(tree, bottom.id).checked_add(1).unwrap()];
        let second = &tree.nodes()[f::index(tree, first.id).checked_add(usize::try_from(first.size).unwrap()).unwrap()];
        assert_eq!(f::text(tree, second).as_ref(), right.as_bytes(), "{name}");
        if let Some(goal) = goal {
            assert!(has(tree, look::HINT, goal.as_bytes()), "{name}: goal is a hint");
        }
        assert_eq!(
            tree.nodes()
                .iter()
                .filter(
                    |node| node.styles.as_slice() == [look::HINT] && f::text(tree, node).as_ref() == " · ".as_bytes()
                )
                .count(),
            usize::from(!right.is_empty()).checked_add(usize::from(goal.is_some())).unwrap(),
            "{name}: pane separators are hints",
        );
    }
    front_quiet(&root, &words);
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
    reply_states();
    walk_states();
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
    for (count, prefix, head) in [
        (0, "", "Stop the box?"),
        (1, "/", "Stop the box? csv-export's running task stops with it."),
        (2, "", "Stop the box? 2 running tasks stop with it."),
    ] {
        let mut rows = List::with_capacity(2);
        for task in 0..count {
            rows.push(super::front::working(9_u64.checked_add(task).unwrap(), b"csv-export", task)).unwrap();
        }
        let mut root = super::launchpad::home(None, rows.as_slice());
        act(&mut root, d::pages::Act::ChatWords { words: prefix.as_bytes().into() });
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Front);
        tui.step(Told::Quitting);
        let part = foot::read(&root, &tui, &words, f::NOW);
        let list = part.list.as_ref().expect("quit question stands before every other list");
        assert_eq!(list.head.as_deref(), Some(head.as_bytes()));
        assert_eq!(list.escape, TuiEvent::Stay);
        let view = draw(part);
        for (name, event) in [(b"1. Stop the box".as_slice(), TuiEvent::Stop), (b"2. Don't quit", TuiEvent::Stay)] {
            let item = f::find(view.tree(), Role::MenuItem, name);
            assert_eq!(item.styles.as_slice(), &[look::OPTION]);
            assert_eq!(crate::event(&view, item.id, Act::Press), Some(event));
        }
        assert_eq!(crate::escape(&view), Some(TuiEvent::Stay));
        let size = skein_ui_screen::Size { columns: 80, rows: 24 };
        let mut focus = skein_ui_screen::Focus::default();
        let mut frame = skein_ui_screen::Frame::new(size);
        skein_ui_screen::render(view.tree(), &crate::theme(Scheme::Dark, None), size, &mut focus, &mut frame);
        assert_eq!(focus.active, Some(f::find(view.tree(), Role::MenuItem, b"1. Stop the box").id));
    }
    quit_priority();
    assert!(foot::lists::question_list(question(Some(1), false), Some(c::ObjectKey::Task(9)), &words).is_none());
    assert!(foot::lists::question_list(question(Some(1), false), Some(c::ObjectKey::Task(10)), &words).is_some());
}
#[expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::too_many_lines,
    reason = "reply rows inspect the rendered options and their bindings"
)]
fn reply_states() {
    use super::chat::{REPLY_TURN, reply_root};
    let aside = TuiEvent::ReplyAside { turn: REPLY_TURN };
    for (case, typed, drawn, numbered) in [
        ("choices", b"".as_slice(), true, true),
        ("plain chat", b"", false, false),
        ("writing draft", b"", true, true),
        ("document", b"", true, true),
        ("no choices", b"", false, false),
        ("walk", b"", false, false),
        ("held question", b"", false, false),
        ("set aside", b"", false, false),
        ("newer", b"", true, true),
        ("newer without choices", b"", false, false),
        ("newer without text", b"", false, false),
        ("person after", b"", false, false),
        ("provisional after", b"", false, false),
        ("working", b"", false, false),
        ("typed number", b"1", false, true),
        ("typed words", b"my answer", false, true),
        ("commands", b"/", false, false),
        ("models", b"", false, false),
    ] {
        let mut root = reply_root(case);
        let mut tui = Tui::new(f::ADDRESS);
        if case == "set aside" || case == "newer" {
            tui.step(Told::ReplyAside { turn: REPLY_TURN });
        }
        if case == "writing draft" {
            act(&mut root, d::pages::Act::AddDraft);
            assert!(root.pages().chat().unwrap().writing);
        }
        if case == "document" {
            assert!(root.pages().chat().unwrap().draft.is_some());
        }
        if case == "walk" {
            tui.step(Told::Walk { by_key: false });
        }
        act(&mut root, d::pages::Act::MessageWords { words: typed.into() });
        if case == "models" {
            act(&mut root, d::pages::Act::Terminal(d::pages::terminal::Act::Run { command: Command::Model }));
        }
        let mut view = f::view();
        f::build(&mut view, &tui, &root);
        let turn = if case == "newer" { c::TurnName { number: 2, ..REPLY_TURN } } else { REPLY_TURN };
        let expected = TuiEvent::ReplyAside { turn };
        assert_eq!(crate::escape(&view) == Some(expected.clone()), drawn || numbered, "{case}: escape");
        let tree = view.tree();
        let items: Vec<_> = tree.nodes().iter().filter(|node| node.role == Role::MenuItem).collect();
        if drawn {
            assert_eq!(items.len(), 3, "{case}");
            assert!(!tree.nodes().iter().any(|node| node.styles.as_slice().contains(&look::LIST_HEAD)));
            assert!(!tree.nodes().iter().any(|node| node.region == Some(look::BOTTOM)));
            let field = f::find(tree, Role::TextArea, b"Message");
            assert!(field.placeholder.is_none());
            for (index, (name, description, event)) in [
                (
                    "1. Move it at once",
                    "no mixed state",
                    TuiEvent::Answer { words: b"Move it at once".as_slice().into() },
                ),
                ("2. Side by side", "", TuiEvent::Answer { words: b"Side by side".as_slice().into() }),
                ("3. Tell it something else", "", expected.clone()),
            ]
            .into_iter()
            .enumerate()
            {
                let item = items[index];
                assert_eq!(item.name.as_deref(), Some(name.as_bytes()), "{case}");
                assert_eq!(item.description.as_deref(), Some(description.as_bytes()), "{case}");
                assert_eq!(item.styles.as_slice(), &[look::OPTION], "{case}");
                assert_eq!(item.states.as_slice().contains(&State::Selected), index == 0, "{case}");
                assert_eq!(crate::event(&view, item.id, Act::Press), Some(event), "{case}");
                assert!(f::index(tree, field.id) < f::index(tree, item.id));
            }
        } else if case == "walk" {
            assert_eq!(crate::escape(&view), Some(TuiEvent::EndWalk));
            assert!(items.iter().any(|item| item.name.as_deref() == Some(b"4. Next")));
        } else if case == "held question" {
            assert!(shared::question(&root, view.words().view()).is_some());
            assert!(items.iter().any(|item| item.name.as_deref() == Some(b"1. Comma")));
            assert_ne!(crate::escape(&view), Some(aside.clone()));
        } else if case != "commands" && case != "models" {
            assert!(items.is_empty(), "{case}");
        }
        for (words, event) in [
            (b"1".as_slice(), Some(TuiEvent::Answer { words: b"Move it at once".as_slice().into() })),
            (b"2", Some(TuiEvent::Answer { words: b"Side by side".as_slice().into() })),
            (b"3", Some(expected.clone())),
            (b"0", None),
            (b"4", None),
            (b"01", None),
            (b"1 ", None),
            (b" 1", None),
            (b"1\n", None),
            (b"-1", None),
            (b"words", None),
            (b"", None),
        ] {
            assert_eq!(crate::reply_number(&view, words), if numbered { event } else { None }, "{case}: {words:?}");
        }
    }
}

#[expect(clippy::disallowed_methods, reason = "test assertions filter the rendered note paragraphs")]
fn note_states() {
    walk_notes();
    front_notes();
    let mut root = root(true, true, false, false, false);
    let words = f::words();
    let mut tui = Tui::new(f::ADDRESS);
    act(&mut root, d::pages::Act::Terminal(d::pages::terminal::Act::Run { command: Command::Help }));
    act(&mut root, d::pages::Act::Terminal(d::pages::terminal::Act::Font));
    let notes: foot::Notes = foot::read_notes(&root, &tui, &words);
    let want = [
        "/draft   A design session that ends in a draft and a goal",
        "/change  One change, run as the project's way says",
        "/ask     Ask temper",
        "/model   Change the model, from the next turn",
        "/effort  Change the effort",
        "/new     Start a new chat",
        "/chats   Open another chat",
        "/open    Open this on the web",
        "/theme   Auto, light or dark",
        "/help    Commands and keys",
        "/quit    Close temper",
        "Enter sends",
        "Tab completes a command",
        "PgUp and PgDn read back",
        "Esc closes a list",
        "Ctrl+C closes temper",
        "Font: IBM Plex Mono.",
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
    tui.step(Told::Front);
    let notes = foot::read_notes(&root, &tui, &words);
    let mut log = None;
    let mut exit = None;
    for (index, line) in notes.lines.iter().enumerate() {
        if line.as_ref() == b"/log     The box's recent log" {
            log = Some(index);
        }
        if line.as_ref() == b"/quit    Close temper and stop the box" {
            exit = Some(index);
        }
    }
    assert_eq!(log, Some(10));
    assert_eq!(exit, Some(11));
    assert!(notes.lines.iter().any(|line| line.as_ref() == b"Ctrl+C closes temper and stops the box"));
}

#[test]
#[expect(clippy::disallowed_macros, reason = "the golden table embeds its hand-written frame oracles")]
#[expect(clippy::too_many_lines, reason = "one table holds the foot screens and their hand-written frames")]
fn the_foots_are_their_goldens() {
    use skein_ui_screen::{Focus, Size};
    let full = Size { columns: 80, rows: 24 };
    let rows = [
        (
            "foot-notice-80x24-dark",
            Scheme::Dark,
            full,
            6,
            include_bytes!("golden/foot-notice-80x24-dark.txt").as_slice(),
        ),
        (
            "foot-notice-40x12-dark",
            Scheme::Dark,
            Size { columns: 40, rows: 12 },
            6,
            include_bytes!("golden/foot-notice-40x12-dark.txt").as_slice(),
        ),
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
        (
            "foot-walk-80x24-dark",
            Scheme::Dark,
            Size { columns: 80, rows: 24 },
            3,
            include_bytes!("golden/foot-walk-80x24-dark.txt").as_slice(),
        ),
        (
            "foot-walk-80x24-light",
            Scheme::Light,
            Size { columns: 80, rows: 24 },
            3,
            include_bytes!("golden/foot-walk-80x24-light.txt").as_slice(),
        ),
        (
            "foot-walk-110x32-dark",
            Scheme::Dark,
            Size { columns: 110, rows: 32 },
            3,
            include_bytes!("golden/foot-walk-110x32-dark.txt").as_slice(),
        ),
        ("foot-goal-80x24-dark", Scheme::Dark, full, 7, include_bytes!("golden/foot-goal-80x24-dark.txt").as_slice()),
        (
            "foot-goal-80x24-light",
            Scheme::Light,
            full,
            7,
            include_bytes!("golden/foot-goal-80x24-light.txt").as_slice(),
        ),
        ("foot-theme-80x24-dark", Scheme::Dark, full, 4, include_bytes!("golden/foot-theme-80x24-dark.txt").as_slice()),
        (
            "foot-theme-answered-80x24-dark",
            Scheme::Dark,
            full,
            5,
            include_bytes!("golden/foot-theme-answered-80x24-dark.txt").as_slice(),
        ),
    ];
    let mut views = List::with_capacity(12);
    let words = f::words();
    for (_, _, size, kind, _) in &rows {
        if *kind == 3 {
            views.push(walk_golden(*size)).unwrap();
            continue;
        }
        let mut root = if *kind == 7 {
            let mut root = f::GoalCase::READY.root();
            open_goal(&mut root);
            root
        } else {
            root(true, true, *kind == 2, false, false)
        };
        if *kind == 1 {
            act(&mut root, d::pages::Act::ChatWords { words: Box::from(*b"/") });
        }
        let mut tui = Tui::new(f::ADDRESS);
        if *kind == 4 || *kind == 5 {
            tui.step(Told::Theme {
                choice: if *kind == 4 { crate::theme::ThemeChoice::Auto } else { crate::theme::ThemeChoice::Light },
                terminal: if *kind == 4 { None } else { Some(Scheme::Dark) },
            });
            tui.step(Told::ThemeList { open: true });
        }
        if *kind == 6 {
            tui.step(Told::Front);
            tui.step(Told::BoxLine { line: crate::BoxLine::Restarting { step: b"forgejo-3".as_slice().into() } });
            tui.step(Told::Notice { notice: Some(crate::Notice::Unknown { words: b"/qiut".as_slice().into() }) });
        }
        let mut part = foot::read(&root, &tui, &words, f::NOW);
        if *kind != 6 {
            part.closed = Some("Claude Opus 5.5 · high".as_bytes().into());
        }
        part.needs = d::pages::needs::Count { count: 1, cut: false };
        part.columns = size.columns;
        if *kind == 7 {
            part.rule = Some(foot::Rule {
                project: b"Demo".as_slice().into(),
                title: b"Plan".as_slice().into(),
                task: 9,
                place: b"box".as_slice().into(),
            });
        }
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
    let mut checks = List::with_capacity(12);
    for ((name, scheme, size, kind, want), view) in rows.iter().zip(&views) {
        checks
            .push(goldens::Golden {
                background: if *kind == 4 { None } else { Some(crate::theme::palette(*scheme)[0].1) },
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

#[expect(clippy::disallowed_methods, reason = "note rows inspect their semantic log and paragraphs")]
#[expect(clippy::disallowed_types, reason = "note assertions collect the log paragraphs")]
fn front_notes() {
    // Section 6.17's log-line boundary and an overlong incoming line.
    const LOG_BYTES: usize = 512;
    const LONG_LINE: usize = 600;
    type Lines = Box<[Box<[u8]>]>;
    type NoteCase = (&'static str, bool, bool, bool, Lines, Lines);
    let root = root(true, true, false, false, false);
    let words = f::words();
    let rows: [NoteCase; 7] = [
        (
            "three lines",
            true,
            true,
            false,
            Box::new([b"one".as_slice().into(), b"two".as_slice().into(), b"three".as_slice().into()]),
            Box::new([b"one".as_slice().into(), b"two".as_slice().into(), b"three".as_slice().into()]),
        ),
        ("empty", true, true, false, Box::default(), Box::new([b"Nothing yet.".as_slice().into()])),
        ("cut", true, true, false, Box::new([Box::new([b'x'; LONG_LINE])]), Box::new([Box::new([b'x'; LOG_BYTES])])),
        (
            "invalid UTF-8",
            true,
            true,
            false,
            Box::new([Box::from(*b"bad \xff line")]),
            Box::new(["bad � line".as_bytes().into()]),
        ),
        (
            "UTF-8 boundary",
            true,
            true,
            false,
            Box::new([skein_lib::bytes::concat(&[&[b'x'; 511], "界".as_bytes()])]),
            Box::new([Box::new([b'x'; 511])]),
        ),
        ("given up", true, false, true, Box::default(), Box::default()),
        ("out of front", false, true, false, Box::new([b"hidden".as_slice().into()]), Box::default()),
    ];
    for (name, front, log, gave_up, lines, expected) in rows {
        let mut tui = Tui::new(f::ADDRESS);
        if front {
            tui.step(Told::Front);
        }
        for line in lines {
            tui.step(Told::BoxLine { line: crate::BoxLine::Log { line } });
        }
        if log {
            tui.step(Told::Log);
        }
        if gave_up {
            tui.step(Told::BoxLine { line: crate::BoxLine::GaveUp { step: b"forgejo-3".as_slice().into() } });
        }
        let notes = foot::read_notes(&root, &tui, &words);
        let bounds = crate::limits(&f::limits(), &skein_ui::Limits::EMPTY, 512).unwrap();
        let mut render = skein_ui::View::new(&bounds);
        let mut b = render.builder();
        b.open(Role::Page);
        foot::draw_notes(&mut b, &notes, &words);
        b.close();
        render.finish(b, 1, &mut Queue::uncapped());
        let tree = render.tree();
        let log = tree.nodes().iter().find(|node| node.role == Role::Log);
        assert_eq!(log.is_some(), front && !expected.is_empty(), "{name}");
        if let Some(log) = log {
            assert_eq!(log.end_request, Some(tui.revision()));
            assert!(has(tree, look::HEAD, b"/log"));
            let start = f::index(tree, log.id);
            let end = start.checked_add(usize::try_from(log.size).unwrap()).unwrap();
            let paragraphs: Vec<_> =
                tree.nodes()[start..end].iter().filter(|node| node.role == Role::Paragraph).collect();
            assert_eq!(paragraphs.len(), expected.len(), "{name}");
            for (node, expected) in paragraphs.iter().zip(expected) {
                assert_eq!(f::text(tree, node).len(), expected.len(), "{name}: bounded line");
                assert_eq!(f::text(tree, node), expected, "{name}");
                assert_eq!(node.styles.as_slice(), &[look::NOTE]);
            }
        }
        assert_eq!(
            has(tree, look::REFUSED, b"The box gave up: forgejo-3. /log shows why; Ctrl+C closes."),
            gave_up,
            "{name}"
        );
    }
}

fn quit_priority() {
    #[derive(Debug)]
    enum Standing {
        Commands,
        Models,
        Confirmation,
        Question,
    }
    for standing in [Standing::Commands, Standing::Models, Standing::Confirmation, Standing::Question] {
        let mut root = match standing {
            Standing::Commands | Standing::Models => root(true, true, false, false, false),
            Standing::Confirmation => root(true, true, true, false, false),
            Standing::Question => super::launchpad::question_root(),
        };
        match standing {
            Standing::Commands => act(&mut root, d::pages::Act::ChatWords { words: b"/".as_slice().into() }),
            Standing::Models => {
                act(&mut root, d::pages::Act::Terminal(d::pages::terminal::Act::Run { command: Command::Model }));
            }
            Standing::Confirmation => {
                act(&mut root, d::pages::Act::Terminal(d::pages::terminal::Act::Run { command: Command::Close }));
            }
            Standing::Question => {}
        }
        let words = f::words();
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Front);
        let before = foot::read(&root, &tui, &words, f::NOW);
        assert!(before.list.is_some(), "the other list stands before quitting: {standing:?}");
        tui.step(Told::Quitting);
        let part = foot::read(&root, &tui, &words, f::NOW);
        let list = part.list.as_ref().unwrap();
        assert_eq!(list.escape, TuiEvent::Stay);
        assert_eq!(list.items[0].event, TuiEvent::Stop);
        assert_eq!(list.items[1].event, TuiEvent::Stay);
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "quiet-line rows find the bottom paragraph and make UTF-8 boundary inputs"
)]
fn front_quiet(root: &Domain, words: &crate::words::Words) {
    // The design's step boundary and the first scalar that crosses it.
    const STEP_BYTES: usize = 64;
    const PAST_STEP: usize = 65;
    type StepCase = (Box<[u8]>, Box<[u8]>);
    for (name, step, playground, count, columns, expected, closed) in [
        ("restarting", Some(false), false, 0, 0, "forgejo-3 restarting", true),
        ("gave up", Some(true), false, 0, 0, "forgejo-3 gave up", true),
        ("playground", None, true, 0, 0, "playground", true),
        ("restarting with needs", Some(false), false, 1, 0, "forgejo-3 restarting · 1 needs you · n to walk", true),
        ("gave up with needs", Some(true), false, 1, 0, "forgejo-3 gave up · 1 needs you · n to walk", true),
        ("playground with needs", None, true, 1, 0, "playground · 1 needs you · n to walk", true),
        ("all in order", Some(false), true, 1, 0, "forgejo-3 restarting · playground · 1 needs you · n to walk", true),
        ("narrow with state", Some(false), false, 0, 60, "forgejo-3 restarting", false),
    ] {
        let root = super::launchpad::home_read(
            None,
            &[],
            false,
            if playground { Some((Some(1), temper_client_protocol_wire::v1::PlaygroundState::Running)) } else { None },
        );
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Front);
        if let Some(gave_up) = step {
            tui.step(Told::BoxLine {
                line: if gave_up {
                    crate::BoxLine::GaveUp { step: b"forgejo-3".as_slice().into() }
                } else {
                    crate::BoxLine::Restarting { step: b"forgejo-3".as_slice().into() }
                },
            });
        }
        let mut part = foot::read(&root, &tui, words, f::NOW);
        part.needs = d::pages::needs::Count { count, cut: false };
        part.columns = columns;
        part.closed = Some("Claude Opus 5.5 · high".as_bytes().into());
        let view = draw(part);
        let bottom = view.tree().nodes().iter().find(|node| node.region == Some(look::BOTTOM)).unwrap();
        let paragraph = &view.tree().nodes()[f::index(view.tree(), bottom.id).checked_add(1).unwrap()];
        let paragraph = &view.tree().nodes()
            [f::index(view.tree(), paragraph.id).checked_add(usize::try_from(paragraph.size).unwrap()).unwrap()];
        assert_eq!(f::text(view.tree(), paragraph).as_ref(), expected.as_bytes(), "{name}");
        assert_eq!(has(view.tree(), look::HINT, "Claude Opus 5.5 · high".as_bytes()), closed, "{name}");
        if let Some(gave_up) = step {
            assert!(
                has(
                    view.tree(),
                    if gave_up { look::DOWN } else { look::SIGNAL },
                    if gave_up { b"forgejo-3 gave up" } else { b"forgejo-3 restarting" }
                ),
                "{name}"
            );
        }
    }
    let steps: [StepCase; 4] = [
        (Box::new([b'x'; STEP_BYTES]), Box::new([b'x'; STEP_BYTES])),
        (Box::new([b'x'; PAST_STEP]), Box::new([b'x'; STEP_BYTES])),
        ("界".repeat(22).as_bytes().into(), "界".repeat(21).as_bytes().into()),
        (Box::from(*b"bad \xff"), "bad �".as_bytes().into()),
    ];
    for (raw, expected) in steps {
        for gave_up in [false, true] {
            let mut tui = Tui::new(f::ADDRESS);
            tui.step(Told::Front);
            tui.step(Told::BoxLine {
                line: if gave_up {
                    crate::BoxLine::GaveUp { step: raw.clone() }
                } else {
                    crate::BoxLine::Restarting { step: raw.clone() }
                },
            });
            let part = foot::read(root, &tui, words, f::NOW);
            let (state, style) = part.box_state.unwrap();
            assert_eq!(
                state,
                skein_lib::bytes::concat(&[&expected, if gave_up { b" gave up" } else { b" restarting" }])
            );
            assert_eq!(style, if gave_up { look::DOWN } else { look::SIGNAL });
            if gave_up {
                assert_eq!(
                    foot::read_notes(root, &tui, words).gave_up.unwrap(),
                    skein_lib::bytes::concat(&[b"The box gave up: ", &expected, b". /log shows why; Ctrl+C closes."])
                );
            }
        }
    }
}

const WALK_QUESTION: c::ObjectKey = c::ObjectKey::Question { task: 9, message: 1 };
const WALK_HELD: c::ObjectKey = c::ObjectKey::Escalation(10);

// Feed the page's public reads, including a primary choice after a non-primary one.
#[expect(clippy::too_many_lines, reason = "one fixture delivers Home's reads, question, held task and status")]
pub(super) fn walk_root(choices: bool) -> Domain {
    use jig_client::Up;
    let mut root = f::signed_out();
    let mut out = f::event(
        &mut root,
        Event::Start {
            address: Address::Home { project: 1 },
            kept: List::with_capacity(0),
            with: Some(jig_client::With::Transport),
        },
    );
    let mut more = f::client(&mut root, jig_client::Event::Up { up: Up::Served { served: true, versions: None } });
    while let Some(request) = more.pop() {
        out.push(request);
    }
    while let Some(request) = out.pop() {
        let d::Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else { continue };
        let up = match down {
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Me) } => {
                let mut me = f::me(&[b"Hello"]);
                let project = me.projects.get_mut(0).unwrap();
                project.templates = List::from_boxed(Box::new([template(1), template(5), {
                    let mut draft = template(6);
                    draft.executor = c::TemplateExecutor::Charter(12);
                    draft
                }]));
                project.models = List::from_boxed(Box::new([super::commands::model()]));
                project.defaults = List::from_boxed(Box::new([c::MeDefault {
                    template: d::pages::chats::ASKING_TEMPLATE,
                    choice: c::LlmChoice {
                        account: Some(1),
                        main: c::ModelEffort { model: text(b"model"), effort: Some(text(b"high")) },
                        subagents: List::with_capacity(0),
                        later: None,
                    },
                }]));
                Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) }
            }
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Inbox { .. }) } => {
                Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Inbox(Box::new(c::Inbox {
                        party: 73,
                        needs: List::with_capacity(0),
                        updates: List::with_capacity(0),
                        next: None,
                        cached_needs: 0,
                        cached_updates: c::Count { value: 0, limited: false },
                        results_seen: 0,
                        updates_time_complete: true,
                        selection: None,
                    }))),
                }
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Inbox { group, .. }) } => {
                let mut rows = List::with_capacity(2);
                if group == c::InboxReadGroup::Needs {
                    for (number, object) in
                        [(9, c::InboxObject::Question { message: 1 }), (10, c::InboxObject::Escalation { revision: 1 })]
                    {
                        rows.push(c::InboxEntry {
                            reply: None,
                            task: c::TaskLink::Stored { number, status: None },
                            project: Some(1),
                            object,
                            activity: Some(1),
                            at: Some(f::NOW),
                            unread: None,
                            cursor: c::InboxCursor::Inbox { task: number, object, at: Some(f::NOW) },
                            projects: List::with_capacity(0),
                        })
                        .unwrap();
                    }
                }
                Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Inbox(Box::new(
                        c::InboxPage { rows, cut: false, before: None, after: None, selection: None },
                    )))),
                }
            }
            jig_client::Down::Open { watch, subject }
                if Some(&subject) == d::forge::status::watch(1, &f::limits().forge).as_ref() =>
            {
                use temper_client_protocol_wire::{self as wire, v1 as w};
                let brief = w::StatusBrief::new(
                    &w::CEILINGS,
                    w::StatusBriefParts {
                        sentence: b"One change landed.".as_slice().into(),
                        lede: Box::default(),
                        at: 0,
                        landed: 1,
                    },
                )
                .unwrap();
                let status = w::ProjectStatus::new(
                    &w::CEILINGS,
                    w::ProjectStatusParts {
                        project: 1,
                        landed: 1,
                        seen: 0,
                        last_landed: Some(0),
                        previous: Some(0),
                        queued: 0,
                        red: false,
                        branch: b"main".as_slice().into(),
                        brief: Some(brief),
                    },
                )
                .unwrap();
                Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::project_status(&status),
                    },
                }
            }
            jig_client::Down::Send { attempt, request, .. }
                if Some(&request) == d::forge::status::ask(1, &f::limits().forge).as_ref() =>
            {
                use temper_client_protocol_wire::{self as wire, v1 as w};
                Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Family(jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::answer(
                            &w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome: w::Outcome::Looked }).unwrap(),
                        ),
                    })),
                }
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Family { record, .. } }
                if f::ways_read(&record.bytes) =>
            {
                Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family {
                        family: temper_client_protocol_wire::FORGE,
                        record: Some(f::draft_setting(true)),
                    },
                }
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Question { task, message }) } => {
                let mut chip = walk_task().chip;
                chip.task = task;
                chip.title = c::DocumentText::complete(text(b"csv-export"));
                let words: &[u8] = if choices {
                    b"csv-export: comma or semicolon?\nChoice: Semicolon\nChoice: Comma (recommended)"
                } else {
                    b"csv-export: comma or semicolon?"
                };
                Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Question(Box::new(
                        c::Question {
                            task: chip,
                            from: None,
                            message,
                            words: c::DocumentText::complete(c::Text::new(words.into(), 256).unwrap()),
                            since: f::NOW,
                            acts: List::from_boxed(Box::new([c::Offered {
                                act: c::ActKind::Answer,
                                admission: c::Admission::Allowed,
                                effect: None,
                            }])),
                            ending: None,
                        },
                    )))),
                }
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Escalation { task }) } => {
                let mut document = walk_task();
                document.chip.task = task;
                document.chip.title = c::DocumentText::complete(text(b"Held task"));
                document.hold = None;
                Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Escalation(Box::new(
                        c::Escalation { task: Box::new(document), acts: List::with_capacity(0) },
                    )))),
                }
            }
            jig_client::Down::Open { watch, subject: jig_client::Watchable::Jig(c::Subject::Tasks { .. }) } => {
                Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                        rows: List::with_capacity(0),
                        more: false,
                    }))),
                }
            }
            jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Tasks { .. }) } => {
                Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Tasks(Box::new(
                        c::TaskPage { live: List::with_capacity(0), live_after: None, ended: c::EndedGroup::None },
                    )))),
                }
            }
            jig_client::Down::Open { watch, subject }
                if Some(&subject) == d::forge::decided::watch(1, &f::limits().forge).as_ref() =>
            {
                use temper_client_protocol_wire::{self as wire, v1 as w};
                use temper_protocol_reaches::{Class, Kind, Reach, Reason};
                let reach = Reach {
                    kind: Kind::Implementation,
                    class: Class::Ask,
                    reason: Reason::Kind,
                    under: 0,
                    sets: 0,
                    matched: 0,
                    away: false,
                };
                let asked = w::Asked::new(
                    &w::CEILINGS,
                    w::AskedParts { number: 1, task: 9, at: 0, reach: temper_protocol_reaches::reach_record(&reach) },
                )
                .unwrap();
                let document = w::Decisions::new(
                    &w::CEILINGS,
                    w::DecisionsParts {
                        project: 1,
                        rows: List::with_capacity(0),
                        asked: List::from_boxed(Box::new([asked])),
                        cut: false,
                    },
                )
                .unwrap();
                Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::decisions(&document),
                    },
                }
            }
            jig_client::Down::Open { subject, .. }
                if Some(&subject) == d::forge::approvals::watch(&f::limits().forge).as_ref() =>
            {
                continue;
            }
            jig_client::Down::Open { subject: jig_client::Watchable::Jig(c::Subject::Accounts), .. }
            | jig_client::Down::Save { .. }
            | jig_client::Down::Erase { .. }
            | jig_client::Down::Send { .. }
            | jig_client::Down::Continue { .. }
            | jig_client::Down::Close { .. }
            | jig_client::Down::Read { .. } => continue,
            jig_client::Down::Open { watch, .. } => Up::WatchEnded { watch, end: c::WatchEnd::Unknown },
        };
        let mut more = f::client(&mut root, jig_client::Event::Up { up });
        while let Some(request) = more.pop() {
            out.push(request);
        }
    }
    root
}
fn walk_task() -> c::Task {
    let bounds =
        jig_client_protocol_link::Limits::new(f::limits().client.domain, f::limits().client.exchanges).unwrap();
    jig_client_protocol_link::translate::documents::task(jig_client_test_records::task_record(), &bounds).unwrap()
}
fn walk_review(root: &mut Domain) {
    f::event(
        root,
        Event::Forge(d::forge::Event::Approvals {
            event: d::forge::approvals::Event::Rows {
                more: 0,
                rows: Box::new([d::forge::approvals::Row {
                    task: 81,
                    project: 1,
                    goal: None,
                    forge: 3,
                    kind: d::forge::forges::Kind::Forgejo,
                    owner: b"work".as_slice().into(),
                    name: b"hello".as_slice().into(),
                    pull: 24,
                    address: b"https://forge.example/work/hello/pulls/24/files".as_slice().into(),
                    title: b"A change".as_slice().into(),
                    since: f::NOW,
                    scope: 1,
                    rule: 1,
                    role: 1,
                    required: 1,
                    counted: 0,
                    behind: 0,
                    whole: true,
                    yours: d::forge::approvals::Yours::Counts,
                }]),
            },
        }),
    );
}
#[expect(clippy::disallowed_methods, reason = "walk rows inspect the rendered menu items")]
fn walk_states() {
    use crate::WalkStep;
    let words = f::words();
    for (name, choices, shown, prefix, expected) in [
        ("question", true, &[][..], "", &["1. Comma", "2. Semicolon", "3. Tell it something else", "4. Next"][..]),
        ("unparsed", false, &[][..], "", &["1. Tell it something else", "2. Next"][..]),
        (
            "before commands",
            true,
            &[][..],
            "/",
            &["1. Comma", "2. Semicolon", "3. Tell it something else", "4. Next"][..],
        ),
        ("second need", true, &[WALK_QUESTION][..], "", &["1. Open it on the web", "2. Next"][..]),
        (
            "shown object gone",
            true,
            &[c::ObjectKey::Task(999)][..],
            "",
            &["1. Comma", "2. Semicolon", "3. Tell it something else", "4. Next"][..],
        ),
        ("done", true, &[WALK_QUESTION, WALK_HELD][..], "", &[][..]),
        ("review", true, &[WALK_QUESTION, WALK_HELD][..], "review", &["1. Review on Forgejo ↗", "2. Next"][..]),
    ] {
        let mut root = walk_root(choices);
        if prefix == "review" {
            walk_review(&mut root);
        }
        if prefix == "/" {
            act(&mut root, d::pages::Act::ChatWords { words: b"/".as_slice().into() });
        }
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Walk { by_key: true });
        for object in shown {
            tui.step(Told::Walked { object: *object });
        }
        let part = foot::read(&root, &tui, &words, f::NOW);
        if expected.is_empty() {
            assert!(part.list.is_none(), "{name}");
            continue;
        }
        let list = part.list.as_ref().expect(name);
        assert_eq!(list.items.len(), expected.len(), "{name}");
        assert_eq!(list.escape, TuiEvent::EndWalk, "{name}");
        let object = if prefix == "review" {
            c::ObjectKey::Task(81)
        } else if name == "second need" {
            WALK_HELD
        } else {
            WALK_QUESTION
        };
        let head = if prefix == "review" {
            "A change: Approve A change on Forgejo"
        } else if name == "second need" {
            "Held task: Held task"
        } else {
            "csv-export: comma or semicolon?"
        };
        assert_eq!(list.head.as_deref(), Some(head.as_bytes()), "{name}");
        for (index, (item, expected)) in list.items.iter().zip(expected).enumerate() {
            assert_eq!(item.name.as_ref(), expected.as_bytes(), "{name}");
            let step = if expected.ends_with("Next") {
                WalkStep::Next
            } else if expected.contains("Review") {
                WalkStep::Open { address: b"https://forge.example/work/hello/pulls/24/files".as_slice().into() }
            } else if expected.contains("web") {
                WalkStep::Open { address: b"http://127.0.0.1:8367/?project=1".as_slice().into() }
            } else if expected.contains("something else") {
                assert_eq!(item.event, TuiEvent::Aside { object }, "{name}");
                continue;
            } else {
                WalkStep::Choose {
                    words: if index == 0 { b"Comma".as_slice().into() } else { b"Semicolon".as_slice().into() },
                }
            };
            assert_eq!(item.event, TuiEvent::Walk { object, step }, "{name}");
            assert_eq!(item.selected, choices && object == WALK_QUESTION && index == 0, "{name}");
        }
        let view = draw(part);
        assert_eq!(crate::escape(&view), Some(TuiEvent::EndWalk));
        for item in view.tree().nodes().iter().filter(|node| node.role == Role::MenuItem) {
            assert!(crate::event(&view, item.id, Act::Press).is_some(), "{name}");
        }
    }
    // The walk owns the prompt even where the chat has its own question.
    let mut root = super::launchpad::question_root();
    let mut tui = Tui::new(f::ADDRESS);
    tui.step(Told::Walk { by_key: false });
    let list = foot::read(&root, &tui, &words, f::NOW).list.unwrap();
    assert_eq!(list.items[0].name.as_ref(), b"1. Tell it something else");
    assert_eq!(list.escape, TuiEvent::EndWalk);
    act(&mut root, d::pages::Act::Terminal(d::pages::terminal::Act::Run { command: Command::Close }));
    assert_eq!(foot::read(&root, &tui, &words, f::NOW).list.unwrap().escape, TuiEvent::EndWalk);
    act(&mut root, d::pages::Act::Cancel);
    act(&mut root, d::pages::Act::Terminal(d::pages::terminal::Act::Run { command: Command::Chats }));
    assert_eq!(foot::read(&root, &tui, &words, f::NOW).list.unwrap().escape, TuiEvent::EndWalk);
    tui.step(Told::Front);
    tui.step(Told::Quitting);
    assert_eq!(foot::read(&root, &tui, &words, f::NOW).list.unwrap().escape, TuiEvent::Stay);
}
#[expect(clippy::disallowed_methods, reason = "note rows find each rendered note by its words")]
fn walk_notes() {
    let mut root = walk_root(true);
    act(&mut root, d::pages::Act::Terminal(d::pages::terminal::Act::Font));
    let words = f::words();
    for (name, done, opened, clear) in [
        ("walking", false, false, false),
        ("done", true, false, false),
        ("opener", false, true, false),
        ("both", true, true, false),
        ("cleared", true, true, true),
    ] {
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Walk { by_key: false });
        if done {
            for object in [WALK_QUESTION, WALK_HELD] {
                tui.step(Told::Walked { object });
            }
        }
        if opened {
            tui.step(Told::NotOpened { address: Some(b"http://127.0.0.1:8367/projects/1".as_slice().into()) });
        }
        if clear {
            tui.step(Told::Cleared);
        }
        let notes = foot::read_notes(&root, &tui, &words);
        assert_eq!(notes.lines[0].as_ref(), b"Font: IBM Plex Mono.", "root's notes first: {name}");
        assert_eq!(
            notes.lines.len(),
            1_usize
                .checked_add(usize::from(done && !clear))
                .unwrap()
                .checked_add(usize::from(opened && !clear))
                .unwrap(),
            "{name}"
        );
        let mut view = f::view();
        f::build(&mut view, &tui, &root);
        for line in &notes.lines {
            assert!(has(view.tree(), look::NOTE, line), "{name}: {line:?}");
        }
        assert_eq!(
            notes.lines.iter().any(|line| line.as_ref() == b"Nothing else needs you."),
            done && !clear,
            "{name}"
        );
        assert_eq!(
            notes.lines.iter().any(|line| line.as_ref() == b"Open http://127.0.0.1:8367/projects/1 in your browser."),
            opened && !clear,
            "{name}"
        );
    }
}

fn walk_golden(size: skein_ui_screen::Size) -> crate::View {
    use shared::NeedMark;
    use view::launchpad::{self, LeftKind, LeftOff, NeedRow};
    const MINUTE: u64 = 60_000_000_000;
    let now = skein_lib::Wall::from_nanos(582 * MINUTE);
    let root = super::launchpad::home(Some(true), &[super::launchpad::line(9, b"Away mode", 492 * MINUTE, false)]);
    let words = f::words();
    let tui = Tui::new(f::ADDRESS);
    let mut body = launchpad::read(&root, &tui, &words, now);
    body.columns = size.columns;
    body.left_off
        .push(LeftOff {
            task: 7,
            project: b"Hello".as_slice().into(),
            title: b"csv-export".as_slice().into(),
            kind: LeftKind::Goal,
            at: b"today 07:42".as_slice().into(),
            state: b"Working".as_slice().into(),
        })
        .unwrap();
    body.needs = if size.columns == 110 {
        Box::new([
            NeedRow {
                since: Some(skein_lib::Wall::from_nanos(562 * MINUTE)),
                mark: NeedMark::Question,
                project: b"Hello".as_slice().into(),
                words: b"csv-export: comma or semicolon?".as_slice().into(),
            },
            NeedRow {
                since: Some(skein_lib::Wall::from_nanos(562 * MINUTE)),
                mark: NeedMark::Money,
                project: b"Demo".as_slice().into(),
                words: b"Porter: raise its weekly budget?".as_slice().into(),
            },
            NeedRow {
                since: Some(skein_lib::Wall::from_nanos(562 * MINUTE)),
                mark: NeedMark::SignOff,
                project: b"Hello".as_slice().into(),
                words: b"Monthly totals: ready for review".as_slice().into(),
            },
            NeedRow {
                since: Some(skein_lib::Wall::from_nanos(562 * MINUTE)),
                mark: NeedMark::Other,
                project: b"Demo".as_slice().into(),
                words: b"Proposal: review the new way".as_slice().into(),
            },
        ])
    } else {
        Box::new([NeedRow {
            since: Some(skein_lib::Wall::from_nanos(562 * MINUTE)),
            mark: NeedMark::Question,
            project: b"Hello".as_slice().into(),
            words: b"csv-export: comma or semicolon?".as_slice().into(),
        }])
    };
    let mut facts = d::pages::all_home::facts(&root, now);
    facts.needs = d::pages::home::Needs::Count { count: u32::try_from(body.needs.len()).unwrap(), cut: false };
    body.lines.get_mut(0).unwrap().words = shared::all_home_headline(words.view(), &facts, now).0;
    let root = walk_root(true);
    let mut tui = Tui::new(f::ADDRESS);
    tui.step(Told::Walk { by_key: true });
    let mut foot = foot::read(&root, &tui, &words, now);
    foot.columns = size.columns;
    let mut view = f::view();
    view::signed(&mut view, &view::Signed { body: view::Body::Launchpad(body), foot }, &mut Queue::uncapped());
    view
}

#[test]
fn the_theme_list_reads_the_terminal_and_selected_choice() {
    use crate::theme::ThemeChoice;
    let words = f::words();
    let mut root = root(true, true, false, false, false);
    act(&mut root, d::pages::Act::Terminal(d::pages::terminal::Act::Run { command: Command::Model }));
    for (terminal, description) in [
        (Some(Scheme::Light), b"light, as your terminal says".as_slice()),
        (Some(Scheme::Dark), b"dark, as your terminal says".as_slice()),
        (None, b"dark: your terminal doesn't say".as_slice()),
    ] {
        for choice in ThemeChoice::ALL {
            let mut tui = Tui::new(f::ADDRESS);
            tui.step(Told::Theme { choice, terminal });
            tui.step(Told::ThemeList { open: true });
            let part = foot::read(&root, &tui, &words, f::NOW);
            let list = part.list.as_ref().unwrap();
            assert_eq!(list.head.as_deref(), Some(b"Theme. Auto follows your terminal.".as_slice()));
            assert_eq!(list.escape, TuiEvent::Theme { choice: None });
            assert_eq!(list.items.len(), 3);
            for (item, (name, expected)) in list.items.iter().zip([
                (b"Auto".as_slice(), ThemeChoice::Auto),
                (b"Light", ThemeChoice::Light),
                (b"Dark", ThemeChoice::Dark),
            ]) {
                assert_eq!(item.name.as_ref(), name);
                assert_eq!(item.words.as_ref(), if expected == ThemeChoice::Auto { description } else { b"" });
                assert_eq!(item.selected, choice == expected);
                assert_eq!(item.event, TuiEvent::Theme { choice: Some(expected) });
            }
            let view = draw(part);
            let head = f::find(view.tree(), Role::Text, b"Theme. Auto follows your terminal.");
            assert_eq!(head.styles.as_slice(), &[look::LIST_HEAD]);
            assert_eq!(crate::escape(&view), Some(TuiEvent::Theme { choice: None }));
            for name in [b"Auto".as_slice(), b"Light", b"Dark"] {
                assert_eq!(f::find(view.tree(), Role::MenuItem, name).styles.as_slice(), &[look::OPTION]);
            }
            tui.step(Told::Front);
            tui.step(Told::Quitting);
            assert_eq!(foot::read(&root, &tui, &words, f::NOW).list.unwrap().escape, TuiEvent::Stay);
        }
    }
}

#[test]
fn a_theme_notice_says_the_choice_and_terminals_word() {
    use crate::{Notice, theme::ThemeChoice};
    let root = root(true, true, false, false, false);
    let words = f::words();
    for (choice, terminal, expected) in [
        (ThemeChoice::Auto, Some(Scheme::Light), b"Theme: auto, light as your terminal says.".as_slice()),
        (ThemeChoice::Auto, Some(Scheme::Dark), b"Theme: auto, dark as your terminal says.".as_slice()),
        (ThemeChoice::Auto, None, b"Theme: auto, dark: your terminal doesn't say.".as_slice()),
        (ThemeChoice::Light, Some(Scheme::Dark), b"Theme: light.".as_slice()),
        (ThemeChoice::Dark, Some(Scheme::Light), b"Theme: dark.".as_slice()),
    ] {
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Notice { notice: Some(Notice::Theme { choice, terminal }) });
        let part = foot::read(&root, &tui, &words, f::NOW);
        assert_eq!(part.notice.as_deref(), Some(expected));
        assert!(foot::read_notes(&root, &tui, &words).lines.is_empty());
        assert!(has(draw(part).tree(), look::SIGNAL, expected));
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "the test locates the reason among semantic nodes and theme paints")]
fn a_walks_reason_stands_between_the_head_and_choices_but_not_on_a_sign_off() {
    let words = f::words();
    let mut root = walk_root(true);
    walk_review(&mut root);
    for review in [false, true] {
        let mut tui = Tui::new(f::ADDRESS);
        tui.step(Told::Walk { by_key: true });
        if review {
            tui.step(Told::Walked { object: WALK_QUESTION });
            tui.step(Told::Walked { object: WALK_HELD });
        }
        let part = foot::read(&root, &tui, &words, f::NOW);
        let list = part.list.as_ref().unwrap();
        assert_eq!(
            list.why.as_deref(),
            if review {
                None
            } else {
                Some(
                    "Reached you because it is an implementation choice, which asks you unless you’re away.".as_bytes(),
                )
            }
        );
        let view = draw(part);
        let nodes = view.tree().nodes();
        let reason = nodes.iter().position(|node| node.styles.as_slice() == [look::WHY]);
        assert_eq!(reason.is_none(), review);
        if let Some(reason) = reason {
            let head = nodes.iter().position(|node| node.styles.as_slice() == [look::LIST_HEAD]).unwrap();
            let choices = nodes.iter().position(|node| node.role == Role::Menu).unwrap();
            assert!(head < reason && reason < choices);
            assert_eq!(nodes[reason].role, Role::Text);
            for scheme in [Scheme::Dark, Scheme::Light] {
                let theme = crate::theme(scheme, None);
                let why = theme.styles.iter().find(|style| style.style == look::WHY).unwrap();
                let signal = theme.styles.iter().find(|style| style.style == look::SIGNAL).unwrap();
                assert_eq!(why.paint, signal.paint);
            }
        }
    }
}
