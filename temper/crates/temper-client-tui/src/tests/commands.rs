//! The TUI's offered commands (temper/docs/design/client/README.md, 6.14, The commands).
use super::fixtures as f;
use jig_client::contract as c;
use skein_lib::List;
use temper_client_domain::{self as d, Address, Domain, Event, Place, pages::terminal::Command};

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
fn root(project: bool, picker: bool, chat: bool, delegate: bool, draft: bool) -> Domain {
    let mut me = f::me(if project { &[b"Demo"] } else { &[] });
    if let Some(project) = me.projects.get_mut(0) {
        project.templates = List::from_boxed(Box::new([template(1), template(5)]));
        if picker {
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
    use Command::{Chats, Close, Effort, Exit, Help, Model, New, Pause, StartOver, Theme};
    let words = f::words();
    let mut failures = List::with_capacity(7);
    for (name, project, picker, chat, delegate, draft, want) in [
        ("launchpad picker", true, true, false, false, false, &[Model, New, Chats, Theme, Help, Exit][..]),
        ("launchpad no picker", true, false, false, false, false, &[New, Chats, Theme, Help, Exit][..]),
        ("no project", false, false, false, false, false, &[Theme, Help, Exit][..]),
        ("chat", true, true, true, false, false, &[Model, Effort, Pause, Close, New, Chats, Theme, Help, Exit][..]),
        ("chat delegate", true, true, true, true, false, &[Model, Effort, Pause, New, Chats, Theme, Help, Exit][..]),
        (
            "chat draft",
            true,
            true,
            true,
            false,
            true,
            &[Model, Effort, Pause, Close, StartOver, New, Chats, Theme, Help, Exit][..],
        ),
        (
            "chat draft delegate",
            true,
            true,
            true,
            true,
            true,
            &[Model, Effort, Pause, New, Chats, Theme, Help, Exit][..],
        ),
    ] {
        let root = root(project, picker, chat, delegate, draft);
        let task = if let Some(chat) = root.pages().chat() { chat.task(root.client()) } else { None };
        if chat {
            assert!(temper_client_view::shared::offered(&root, task, Close), "{name}");
        }
        if draft {
            assert!(root.pages().chat().unwrap().draft.is_some());
            assert!(temper_client_view::shared::offered(&root, task, Command::Draft));
            assert!(temper_client_view::shared::offered(&root, task, Command::Font));
        }
        let found = crate::offered(&root, &words);
        if found.as_ref() != want {
            failures.push((name, found)).unwrap();
        }
    }
    assert!(failures.is_empty(), "command rows: {failures:?}");
}
