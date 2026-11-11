//! The TUI's offered commands (temper/docs/design/client/tui/pages.md, The commands).
use super::fixtures as f;
use jig_client::contract as c;
use skein_lib::List;
use temper_client_domain::{self as d, Address, Domain, Event, Place, pages::terminal::Command};

fn text(bytes: &[u8]) -> c::Text {
    c::Text::new(bytes.into(), 32).unwrap()
}
pub(super) fn model() -> c::MeModel {
    c::MeModel {
        new_until: None,
        number: 1,
        source: c::ModelSource::Stock,
        estimated: false,
        name: text(b"model"),
        provider: None,
        window: 8192,
        output: 1024,
        efforts: List::from_boxed(Box::new([text(b"high")])),
        effort: Some(text(b"high")),
        prices: c::ModelPrices { input: 0, cached: 0, cache_write: None, output: 0, unit: 1 },
        description: None,
        shown: None,
        accounts: List::from_boxed(Box::new([1])),
        picker: true,
    }
}
fn template(number: u32) -> c::Template {
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
fn root(project: bool, picker: bool, chat: bool, delegate: bool, draft: bool, efforts: bool) -> Domain {
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
            let mut model = model();
            if !efforts {
                model.efforts.clear();
                model.effort = None;
            }
            project.models = List::from_boxed(Box::new([model]));
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

#[test]
fn the_tui_offers_its_commands() {
    use Command::{Chats, Close, Draft, Effort, Exit, Help, Model, New, Pause, StartOver};
    let words = f::words();
    let mut failures = List::with_capacity(9);
    for (name, project, picker, chat, delegate, draft, efforts, want) in [
        ("launchpad picker", true, true, false, false, false, true, &[Model, Effort, New, Chats, Help, Exit][..]),
        ("launchpad no efforts", true, true, false, false, false, false, &[Model, New, Chats, Help, Exit][..]),
        ("launchpad no picker", true, false, false, false, false, true, &[New, Chats, Help, Exit][..]),
        ("no project", false, false, false, false, false, true, &[Help, Exit][..]),
        (
            "chat writing",
            true,
            true,
            true,
            false,
            false,
            true,
            &[Model, Effort, Draft, Pause, Close, New, Chats, Help, Exit][..],
        ),
        ("chat", true, true, true, false, false, true, &[Model, Effort, Pause, Close, New, Chats, Help, Exit][..]),
        ("chat delegate", true, true, true, true, false, true, &[Model, Effort, Pause, New, Chats, Help, Exit][..]),
        (
            "chat draft",
            true,
            true,
            true,
            false,
            true,
            true,
            &[Model, Effort, Draft, Pause, Close, StartOver, New, Chats, Help, Exit][..],
        ),
        (
            "chat draft delegate",
            true,
            true,
            true,
            true,
            true,
            true,
            &[Model, Effort, Draft, Pause, New, Chats, Help, Exit][..],
        ),
    ] {
        let mut root = root(project, picker, chat, delegate, draft, efforts);
        if name == "chat writing" {
            f::event(
                &mut root,
                Event::Act {
                    action: d::Action::Pages(d::pages::Act::Terminal(d::pages::terminal::Act::Run { command: Draft })),
                },
            );
            assert!(root.pages().chat().unwrap().writing);
        }
        let task = if let Some(chat) = root.pages().chat() { chat.task(root.client()) } else { None };
        if chat {
            assert!(temper_client_view::shared::offered(&root, task, Close), "{name}");
        }
        if draft {
            assert!(root.pages().chat().unwrap().draft.is_some());
            assert!(temper_client_view::shared::offered(&root, task, Draft));
            assert!(temper_client_view::shared::offered(&root, task, Command::Font));
        }
        let found = crate::offered(&root, &words);
        if found.as_ref() != want {
            failures.push((name, found)).unwrap();
        }
    }
    goal_commands(&words);
    own_commands(&words);
    front_commands(&words);
    assert!(failures.is_empty(), "command rows: {failures:?}");
}

fn goal_commands(words: &crate::words::Words) {
    for (name, case, offered) in [
        ("headed document", f::GoalCase::READY, true),
        ("no heading", f::GoalCase { body: Some(b"Words\n"), ..f::GoalCase::READY }, false),
        ("fenced heading", f::GoalCase { body: Some(b"```\n# Plan\n```\n"), ..f::GoalCase::READY }, false),
        ("blank", f::GoalCase { body: Some(b" \n"), ..f::GoalCase::READY }, false),
        ("no document", f::GoalCase { body: None, ..f::GoalCase::READY }, false),
        ("linked goal", f::GoalCase { linked: true, ..f::GoalCase::READY }, false),
        ("no repository", f::GoalCase { repository: false, ..f::GoalCase::READY }, false),
        ("no drafts", f::GoalCase { drafts: false, ..f::GoalCase::READY }, false),
        ("Ask temper", f::GoalCase { asking: true, ..f::GoalCase::READY }, false),
    ] {
        let root = case.root();
        let commands = crate::offered(&root, words);
        assert_eq!(commands.contains(&Command::Goal), offered, "{name}");
        assert_eq!(crate::goal_offered(&root), offered, "{name}: one callable gate");
        if offered {
            assert_eq!(
                commands.as_ref(),
                &[
                    Command::Draft,
                    Command::Close,
                    Command::Goal,
                    Command::New,
                    Command::Chats,
                    Command::Help,
                    Command::Exit
                ]
            );
        }
    }
}

fn front_commands(words: &crate::words::Words) {
    for (name, front, prefix, expected) in [
        (
            "in front",
            true,
            "/",
            &["/change", "/ask", "/new", "/chats", "/open", "/theme", "/help", "/log", "/quit"][..],
        ),
        ("log prefix", true, "/l", &["/log"][..]),
        ("q prefix", false, "/q", &["/quit"][..]),
        ("qu prefix", false, "/qu", &["/quit"][..]),
        ("exit prefix", false, "/ex", &[][..]),
        ("exi prefix", false, "/exi", &[][..]),
        ("exit alias", false, "/exit", &[][..]),
        ("out of front", false, "/", &["/change", "/ask", "/new", "/chats", "/open", "/theme", "/help", "/quit"][..]),
        ("log out of front", false, "/l", &[][..]),
    ] {
        let mut root = root(true, false, false, false, false, false);
        f::event(
            &mut root,
            Event::Act { action: d::Action::Pages(d::pages::Act::ChatWords { words: prefix.as_bytes().into() }) },
        );
        let mut tui = crate::Tui::new(f::ADDRESS);
        if front {
            tui.step(crate::Told::Front);
        }
        let part = crate::view::foot::read(&root, &tui, words, f::NOW);
        let items = match &part.list {
            Some(list) => list.items.as_ref(),
            None => &[][..],
        };
        assert_eq!(items.len(), expected.len(), "{name}");
        for (item, expected) in items.iter().zip(expected) {
            assert_eq!(item.name.as_ref(), expected.as_bytes(), "{name}");
            if *expected == "/log" {
                assert_eq!(item.words.as_ref(), b"The box's recent log");
                assert_eq!(item.event, crate::Event::Log);
            }
            if *expected == "/quit" {
                assert_eq!(
                    item.words.as_ref(),
                    if front { b"Close temper and stop the box".as_slice() } else { b"Close temper".as_slice() }
                );
            }
        }
    }
}

#[expect(clippy::disallowed_methods, reason = "command rows inspect help names, padding and key order")]
#[expect(clippy::too_many_lines, reason = "one table checks commands, their availability, bindings and help")]
fn own_commands(words: &crate::words::Words) {
    use crate::{Own, Told, Tui};
    for (name, brief, needs, prefix, chat, project, front, expected) in [
        (
            "brief and need",
            true,
            true,
            "/",
            false,
            true,
            false,
            &[
                "/draft", "/change", "/ask", "/model", "/effort", "/new", "/chats", "/brief", "/needs", "/open",
                "/theme", "/help", "/quit",
            ][..],
        ),
        (
            "neither",
            false,
            false,
            "/",
            false,
            true,
            false,
            &["/draft", "/change", "/ask", "/new", "/chats", "/open", "/theme", "/help", "/quit"][..],
        ),
        ("c prefix", true, true, "/c", false, true, false, &["/change", "/chats"][..]),
        ("b prefix", true, true, "/b", false, true, false, &["/brief"][..]),
        ("no brief", false, true, "/b", false, true, false, &[][..]),
        ("no needs", true, false, "/n", false, true, false, &["/new"][..]),
        (
            "chat",
            false,
            true,
            "/",
            true,
            true,
            false,
            &["/pause", "/close", "/new", "/chats", "/open", "/theme", "/help", "/quit"][..],
        ),
        ("no project", false, false, "/", false, false, false, &["/theme", "/help", "/quit"][..]),
        (
            "front",
            true,
            true,
            "/",
            false,
            true,
            true,
            &[
                "/draft", "/change", "/ask", "/new", "/chats", "/brief", "/needs", "/open", "/theme", "/help", "/log",
                "/quit",
            ][..],
        ),
    ] {
        let mut root = if name == "brief and need" {
            super::foot::walk_root(true)
        } else if !project {
            root(false, false, false, false, false, false)
        } else if chat {
            root(true, false, true, false, false, false)
        } else {
            super::launchpad::home_read(Some(brief), &[], needs, None)
        };
        let action = if chat {
            d::pages::Act::MessageWords { words: prefix.as_bytes().into() }
        } else {
            d::pages::Act::ChatWords { words: prefix.as_bytes().into() }
        };
        f::event(&mut root, Event::Act { action: d::Action::Pages(action) });
        let mut tui = Tui::new(f::ADDRESS);
        if front {
            tui.step(Told::Front);
        }
        if !project {
            tui.step(Told::CommandWords { value: prefix.as_bytes().into() });
        }
        let part = crate::view::foot::read(&root, &tui, words, f::NOW);
        let items = match &part.list {
            Some(list) => list.items.as_ref(),
            None => &[],
        };
        assert_eq!(items.len(), expected.len(), "{name}: {items:?}");
        for (item, expected) in items.iter().zip(expected) {
            assert_eq!(item.name.as_ref(), expected.as_bytes(), "{name}");
            for (own, description) in [
                (Own::Draft, "A design session that ends in a draft and a goal"),
                (Own::Change, "One change, run as the project's way says"),
                (Own::Ask, "Ask temper"),
                (Own::Brief, "The brief, whole"),
                (Own::Needs, "Walk what needs you"),
                (Own::Open, "Open this on the web"),
                (Own::Theme, "Auto, light or dark"),
            ] {
                if own.name() == expected.as_bytes() {
                    assert_eq!(item.event, crate::Event::Own { command: own }, "{name}");
                    assert_eq!(item.words.as_ref(), description.as_bytes(), "{name}");
                }
            }
        }
        f::event(
            &mut root,
            Event::Act {
                action: d::Action::Pages(d::pages::Act::Terminal(d::pages::terminal::Act::Run {
                    command: Command::Help,
                })),
            },
        );
        let notes = crate::view::foot::read_notes(&root, &tui, words);
        let mut names = List::with_capacity(16);
        for line in &notes.lines {
            if line.starts_with(b"/") {
                names.push(line.split(|b| *b == b' ').next().unwrap()).unwrap();
            }
        }
        // Help is unfiltered even when the box contains a command prefix.
        if prefix == "/" {
            assert_eq!(usize::try_from(names.len()).unwrap(), expected.len(), "help {name}");
            for (found, expected) in names.iter().zip(expected) {
                assert_eq!(*found, expected.as_bytes(), "help {name}");
            }
        }
        assert_eq!(
            notes.lines.iter().any(|line| line.as_ref() == b"n walks what needs you"),
            needs && !chat,
            "help {name}"
        );
        if name == "brief and need" {
            for line in [
                b"/draft   A design session that ends in a draft and a goal".as_slice(),
                b"/change  One change, run as the project's way says",
                b"/ask     Ask temper",
                b"/brief   The brief, whole",
                b"/needs   Walk what needs you",
                b"/open    Open this on the web",
            ] {
                assert!(notes.lines.iter().any(|found| found.as_ref() == line), "help padding and words: {line:?}");
            }
            let esc = notes.lines.iter().position(|line| line.as_ref() == b"Esc closes a list").unwrap();
            assert_eq!(notes.lines[esc.checked_add(1).unwrap()].as_ref(), b"n walks what needs you");
        }
    }
    let mut root = root(true, true, false, false, false, true);
    f::event(
        &mut root,
        Event::Act { action: d::Action::Pages(d::pages::Act::ChatWords { words: b"/".as_slice().into() }) },
    );
    let part = crate::view::foot::read(&root, &Tui::new(f::ADDRESS), words, f::NOW);
    let list = part.list.unwrap();
    let expected = ["/change", "/ask", "/model", "/effort", "/new", "/chats", "/open", "/theme", "/help", "/quit"];
    for (item, expected) in list.items.iter().zip(expected) {
        assert_eq!(item.name.as_ref(), expected.as_bytes());
    }
    assert_eq!(list.items.len(), expected.len());
}

#[test]
fn theme_is_own_on_every_screen() {
    for (name, project, chat, signed_out, front) in [
        ("project", true, false, false, false),
        ("chat", true, true, false, false),
        ("no project", false, false, false, false),
        ("signed out", false, false, true, false),
        ("welcome signing in", false, false, true, true),
        ("welcome signed in", false, false, false, true),
    ] {
        let mut root = if signed_out { f::signed_out() } else { root(project, false, chat, false, false, false) };
        let words = f::words();
        let mut tui = crate::Tui::new(f::ADDRESS);
        if front {
            tui.step(crate::Told::Front);
            tui.step(crate::Told::Local { local: crate::Local::Signing });
        }
        tui.step(crate::Told::CommandWords { value: b"/".as_slice().into() });
        assert!(!crate::offered(&root, &words).contains(&Command::Theme), "{name}");
        assert!(crate::offered_own(&root, &tui).contains(&crate::Own::Theme), "{name}");
        if signed_out {
            assert!(crate::view::foot::read(&root, &tui, &words, f::NOW).list.is_none(), "{name}");
            continue;
        }
        let action = if root.pages().chat().is_some() {
            d::pages::Act::MessageWords { words: b"/".as_slice().into() }
        } else {
            d::pages::Act::ChatWords { words: b"/".as_slice().into() }
        };
        f::event(&mut root, Event::Act { action: d::Action::Pages(action) });
        let Some(list) = crate::view::foot::read(&root, &tui, &words, f::NOW).list else {
            panic!("{name}: command list missing");
        };
        let items = list.items;
        let mut theme = None;
        for (index, item) in items.iter().enumerate() {
            if item.name.as_ref() == b"/theme" {
                theme = Some(index);
            }
        }
        let index = theme.expect("theme offered");
        assert_eq!(items[index].words.as_ref(), b"Auto, light or dark", "{name}");
        assert_eq!(items[index].event, crate::Event::Own { command: crate::Own::Theme }, "{name}");
        assert_eq!(items[index + 1].name.as_ref(), b"/help", "{name}");
        if index > 0 {
            assert_eq!(items[index - 1].name.as_ref(), b"/open", "{name}");
        }
    }
}
