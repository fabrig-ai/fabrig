use super as fixtures;
use super::chat::*;
use jig_client::{contract as c, thread::Thread};
use skein_lib::{List, Wall};
use skein_ui::Role;

pub fn deliver_chat(root: &mut temper_client_domain::Domain, watch: skein_lib::Token, delivery: c::Delivery) {
    fixtures::root_event(
        root,
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) },
            },
        },
    );
}

#[must_use]
pub fn named_position(nodes: &[skein_ui::Node], role: Role, name: Option<&[u8]>) -> Option<usize> {
    for (index, node) in nodes.iter().enumerate() {
        if node.role == role && (name.is_none() || node.name.as_deref() == name) {
            return Some(index);
        }
    }
    None
}

#[must_use]
pub fn structured_messages() -> List<c::Message> {
    let mut messages = List::with_capacity(4);
    for speaker in [c::Speaker::Person, c::Speaker::Assistant, c::Speaker::Tool, c::Speaker::System] {
        let mut blocks = List::with_capacity(2);
        blocks
            .push(c::Block::Call(c::Call {
                number: 1,
                tool: c::Text::new(Box::from(*b"Read"), TEXT).expect("tool"),
                input: c::Text::new(Box::from([]), TEXT).expect("input"),
                outcome: None,
                failed: false,
                objects: List::with_capacity(0),
                parts: List::with_capacity(0),
            }))
            .expect("call");
        blocks
            .push(c::Block::Result {
                call: 1,
                summary: c::Text::new(Box::from(*b"Done"), TEXT).expect("summary"),
                failed: false,
            })
            .expect("result");
        messages.push(c::Message { speaker, blocks }).expect("message");
    }
    messages
}

#[must_use]
pub fn structured_thread() -> Thread {
    let mut thread = Thread::new(TASK, &jig_client::thread::Limits { lines: 12, bytes: BYTES, text: 128, live: 128 });
    thread.reading = Some(jig_client::thread::Reading::Latest { known: 0 });
    let messages = structured_messages();
    let mut rows = List::with_capacity(5);
    for (position, event) in [
        (
            5,
            temper_fixtures::client_documents::line(
                1,
                Wall::EPOCH,
                jig_client::contract::Activity::RunEnded {
                    attempt: ATTEMPT,
                    end: jig_client::contract::RunEnd::Finished,
                    spent: 0,
                },
            ),
        ),
        (
            4,
            temper_fixtures::client_documents::turn(
                1,
                Wall::EPOCH,
                c::TurnName { attempt: ATTEMPT, number: 1 },
                c::TurnRead::Decoded(c::Turn {
                    messages,
                    used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
                    spent: 0,
                    read_through: None,
                    llm: None,
                    cut: false,
                }),
            ),
        ),
        (
            3,
            temper_fixtures::client_documents::line(
                1,
                Wall::EPOCH,
                jig_client::contract::Activity::Words {
                    message: 1,
                    by: jig_client::contract::Requester::Person(
                        (c::Party {
                            number: PARTY,
                            kind: c::PartyKind::Person,
                            name: c::Text::new(Box::from(*b"Pat"), TEXT).expect("person"),
                        })
                        .number,
                    ),
                    kind: jig_client::contract::MessageKind::Words,
                    words: c::Text::new(Box::from(*b"Hello"), TEXT).expect("words"),
                },
            ),
        ),
        (
            2,
            temper_fixtures::client_documents::line(
                1,
                Wall::EPOCH,
                jig_client::contract::Activity::RunBegan {
                    attempt: ATTEMPT,
                    why: jig_client::contract::RunWhy::Made,
                    resumed: false,
                },
            ),
        ),
    ] {
        rows.push({
            let mut line = event;
            line.source.position = position;
            line.source.at = Wall::EPOCH;
            line
        })
        .expect("line");
    }
    thread.page(c::ActivityPage {
        rows,
        after: None,
        before: Some(c::Cursor::new(Box::from(*b"older"), TEXT).expect("cursor")),
        cut: true,
    });
    thread
}

pub fn assert_run_line(tree: &skein_ui::Tree, name: &[u8], description: &[u8], lines: &[&[u8]]) {
    assert!(fixtures::person_find(tree, Role::Disclosure, name).is_some());
    let at = named_position(tree.nodes(), Role::Disclosure, Some(name)).expect("disclosure node");
    let fold = &tree.nodes()[at];
    assert_eq!(fold.description.as_deref().unwrap_or(b""), description);
    assert!(!fold.states.as_slice().contains(&skein_ui::State::Expanded));
    let slot = at.checked_add(1).unwrap();
    assert!(tree.nodes()[slot].styles.as_slice().contains(&crate::look::WORK_CALLS));
    let mut child = slot.checked_add(1).unwrap();
    for line in lines {
        let node = &tree.nodes()[child];
        assert_eq!(node.role, Role::Paragraph);
        assert_eq!(fixtures::node_text(tree, child).as_ref(), *line);
        child = child.checked_add(usize::try_from(node.size).unwrap()).unwrap();
    }
    assert_eq!(child, at.checked_add(usize::try_from(fold.size).unwrap()).unwrap());
}

#[must_use]
pub fn asking_task_root(task: c::Task, template: bool, charter: u32) -> temper_client_domain::Domain {
    asking_task_root_owned(task, template, charter, true)
}

#[must_use]
pub fn asking_task_root_owned(
    mut task: c::Task,
    template: bool,
    charter: u32,
    owned: bool,
) -> temper_client_domain::Domain {
    let (mut root, me_watch, task_watch) =
        fixtures::root_at(temper_client_domain::Address::Chat { task: TASK }, &fixtures::domain_limits());
    let source = messaging_root();
    let mut me = source.pages().me(source.client()).expect("me").clone();
    if template {
        me = fixtures::asking_me(me);
    }
    me.projects.get_mut(0).unwrap().models = super::choice::line_models(true);
    fixtures::deliver_me(&mut root, me_watch, me);
    if !owned {
        super::connect_forge::reread_deployment(&mut root, me_watch, owned);
    }
    task.chip.executor = c::ChipExecutor::Agent { charter };
    deliver_chat(&mut root, task_watch.expect("task watch"), c::Delivery::Task(Box::new(task)));
    root
}

#[must_use]
pub fn asking_task() -> c::Task {
    let source = messaging_root();
    let mut task = source.pages().chat().expect("chat").task(source.client()).expect("task").clone();
    task.chip.title = c::DocumentText::complete(fixtures::first_run::text(b"Hello"));
    task.acts = List::with_capacity(3);
    for act in [c::ActKind::Message, c::ActKind::Stop, c::ActKind::Cancel] {
        task.acts.push(c::Offered { act, admission: c::Admission::Allowed, effect: None }).unwrap();
    }
    task
}

#[must_use]
pub fn model_effort(model: &str, effort: Option<&str>) -> c::ModelEffort {
    c::ModelEffort {
        model: fixtures::first_run::text(model.as_bytes()),
        effort: match effort {
            Some(effort) => Some(fixtures::first_run::text(effort.as_bytes())),
            None => None,
        },
    }
}

#[must_use]
pub fn model_choice(model: &str, effort: Option<&str>) -> c::LlmChoice {
    c::LlmChoice { later: None, account: None, main: model_effort(model, effort), subagents: List::with_capacity(0) }
}

#[must_use]
pub fn model_line_page(latest: Option<c::ModelEffort>) -> c::ActivityPage {
    let mut row =
        temper_fixtures::client_documents::line(1, Wall::EPOCH, c::Activity::Decided { revision: 1, proposal: None });
    if let Some(llm) = latest {
        row.source.line = c::Activity::Turn { reply: 1, attempt: ATTEMPT, turn: TURN, read: None };
        row.decoded = Some(c::ArchivedTurn {
            task: TASK,
            name: c::TurnName { attempt: ATTEMPT, number: TURN },
            cumulative: 0,
            read: None,
            at: Wall::EPOCH,
            turn: c::TurnRead::Decoded(c::Turn {
                messages: List::with_capacity(0),
                used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
                spent: 0,
                read_through: None,
                cut: false,
                llm: Some(llm),
            }),
        });
    }
    c::ActivityPage { rows: List::from_boxed(Box::new([row])), before: None, after: None, cut: false }
}

#[must_use]
pub fn model_line_root(
    task: c::Task,
    latest: Option<c::ModelEffort>,
    preview: Option<u32>,
) -> temper_client_domain::Domain {
    let (mut root, me_watch, watch, _, _) = fixtures::root_at_with_activity(
        temper_client_domain::Address::Chat { task: TASK },
        &fixtures::domain_limits(),
        b"Shop",
        None,
        Some(model_line_page(latest)),
        None,
    );
    let (source, _) = super::choice::provider_byline_root(
        temper_client_domain::Address::NewChat { draft: false },
        [c::Readiness::Usable; 2],
        [true; 2],
        true,
    );
    let mut me = fixtures::asking_me(source.pages().me(source.client()).unwrap().clone());
    let project = me.projects.get_mut(0).unwrap();
    project.models = List::with_capacity(2);
    for (raw, shown, provider) in [("gpt-6-astra", "GPT-6 Astra", 17), ("claude-opus-5-5", "Opus 5.5", 18)] {
        let mut model = super::choice::line_models(true).get(0).unwrap().clone();
        model.name = fixtures::first_run::text(raw.as_bytes());
        model.shown = Some(fixtures::first_run::text(shown.as_bytes()));
        model.effort = Some(fixtures::first_run::text(b"high"));
        model.provider = Some(provider);
        project.models.push(model).unwrap();
    }
    project.defaults = List::with_capacity(2);
    if let Some(template) = preview {
        project.defaults.push(c::MeDefault { template, choice: model_choice("gpt-6-astra", None) }).unwrap();
        if template != 1 {
            project
                .defaults
                .push(c::MeDefault { template: 1, choice: model_choice("wrong-chat-preview", None) })
                .unwrap();
        }
    }
    fixtures::deliver_me(&mut root, me_watch, me);
    deliver_chat(&mut root, watch.unwrap(), c::Delivery::Task(Box::new(task)));
    root
}

#[must_use]
pub fn long_line_page(
    address: temper_client_domain::Address,
    line: c::Activity,
    tracked: bool,
) -> (temper_client_domain::Domain, temper_client_domain::Limits) {
    let mut domain = fixtures::domain_limits();
    domain.client.domain.text = 512;
    domain.client.domain.long_text = 131_072;
    domain.client.domain.turn_text = 131_072;
    domain.client.domain.document_bytes = 1_048_576;
    domain.client.domain.turn_bytes = 1_048_576;
    domain.client.domain.requests.bytes = 4_194_304;
    domain.client.domain.ask_bytes = 1_048_576;
    let mut rows = List::with_capacity(1);
    rows.push(temper_fixtures::client_documents::line(1, Wall::EPOCH, line)).unwrap();
    let (mut root, _, watch, _, _) = fixtures::root_at_with_activity(
        address,
        &domain,
        b"Shop",
        None,
        Some(c::ActivityPage { rows, before: None, after: None, cut: false }),
        None,
    );
    let mut task = fixtures::task::document();
    task.chip.task = TASK;
    task.chip.executor = c::ChipExecutor::Agent { charter: 1 };
    task.priority = tracked.then_some(1);
    task.activity = 1;
    temper_client_domain::step(
        &mut root,
        &skein_lib::Env { now: skein_lib::Time::ZERO, wall: Wall::EPOCH, limits: domain },
        temper_client_domain::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
        &mut skein_lib::Queue::uncapped(),
    );
    (root, domain)
}

pub fn pause_chip(chip: &mut c::Chip) {
    chip.phase = c::Phase::Held;
    chip.details.phase = c::ChipPhase::Held { reason: c::HoldCategory::Stopped, revision: 19 };
    chip.details.details.hold = Some(c::ChipHold {
        placed: Some(c::PlacedHold {
            number: 1,
            owner: c::HoldOwner::Party { person: PARTY },
            mode: c::HoldMode::Stop,
            scope: c::HoldScope::Subtree { task: chip.task },
            reason: Box::default(),
            placed_at: Wall::EPOCH,
            revision: c::Revision { value: 19 },
            lifted: false,
            may_lift: true,
        }),
        words: None,
        circle: None,
        account: None,
        source: None,
        reason: c::HoldCategory::Stopped,
        revision: 19,
        before: c::PriorPhase::Active(c::RunStage::Idle),
        escalation: None,
        tries: c::FailureTries { transient: 0, permanent: 0, run: 0, agent: 0, lost: 0, invalid: 0, infrastructure: 0 },
        hold_wait_since: None,
    });
}
