//! Starting a goal in a chat uses the shared sheet (client/pages/README.md).
#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect semantic trees and prints")]
use super::{connect_forge, documents, fixtures as f};

use crate::look;

use jig_client::{Down, Readable, Up, Watchable, contract as c};

use skein_lib::{Env, List, Queue, Time, Wall, Writer};

use skein_ui::{Role, Tree};

use temper_client_domain::{Action, Address, Domain, Event, Request, pages::Act};

use temper_client_protocol_wire::{self as wire, v1 as w};


const CHAT: u64 = f::chat::TASK;

const GOAL: u64 = 42;

const BODY: &[u8] = b"# Away mode\n\n## Setting\n\nAsk first.\n";

const TITLE: &[u8] = b"Away mode";

const START: &[u8] = "Start goal…".as_bytes();

const LINK: &[u8] = "Goal Keep working →".as_bytes();

const BEFORE: u64 = 10;

const MIDDLE: u64 = 20;

const AFTER: u64 = 30;

const TEXT: u32 = 512;


#[expect(clippy::struct_excessive_bools, reason = "each prerequisite is varied independently")]
struct Case {
    body: Option<&'static [u8]>,
    offered: bool,
    repository: bool,
    asking: bool,
    linked: Option<u64>,
    chip: bool,
}

impl Case {
    const READY: Self =
        Self { body: Some(BODY), offered: true, repository: true, asking: false, linked: None, chip: true };
    #[expect(clippy::too_many_lines, reason = "the fixture answers the chat's watches and reads through public events")]
    fn root(&self) -> Domain {
        let mut limits = f::domain_limits();
        limits.client.domain.text = TEXT;
        limits.client.domain.long_text = TEXT;
        limits.client.domain.turn_text = TEXT;
        limits.client.domain.rows = 4;
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let source = f::playground_root(Address::Chat { task: CHAT }, f::PROJECT, true);
        let mut me = source.pages().me(source.client()).unwrap().clone();
        let member = me.projects.get_mut(0).unwrap();
        let mut templates = List::with_capacity(3);
        for template in &member.templates {
            templates.push(template.clone()).unwrap();
        }
        if self.offered {
            let mut goal = super::goals::template(2, b"goal", true);
            goal.flags.conversation = false;
            templates.push(goal).unwrap();
        }
        member.templates = templates;
        if self.asking {
            let mut template = super::goals::template(5, b"Ask temper", false);
            template.executor = c::TemplateExecutor::Charter(1);
            member.templates.push(template).unwrap();
        }
        let mut task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
        if self.asking {
            task.chip.executor = c::ChipExecutor::Agent { charter: 1 };
        }
        task.activity = 3;
        task.spec.specification.words = c::DocumentText::complete(c::Text::new(Box::default(), TEXT).unwrap());
        let mut root = temper_client_domain::unlinked(&limits, f::SEED);
        let mut out = Queue::uncapped();
        for event in [
            Event::Client { event: jig_client::Event::StartUnlinked { kept: List::uncapped() } },
            Event::Start {
                address: Address::Chat { task: CHAT },
                kept: List::uncapped(),
                with: Some(jig_client::With::Transport),
            },
            Event::Client { event: jig_client::Event::Up { up: Up::Served { served: true, versions: None } } },
        ] {
            temper_client_domain::step(&mut root, &env, event, &mut out);
        }
        for _ in 0_u32..128 {
            let Some(request) = out.pop() else { break };
            let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else { continue };
            let up = match down {
                Down::Open { watch, subject: Watchable::Jig(c::Subject::Me) } => {
                    Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me.clone()))) }
                }
                Down::Open { watch, subject: Watchable::Jig(c::Subject::Settings) } => Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Settings(Box::new(c::SettingsDocument {
                        newest: 0,
                        settings: List::uncapped(),
                        held: List::uncapped(),
                        pending: List::uncapped(),
                    }))),
                },
                Down::Read { read, what: Readable::Family { record, .. } } => {
                    let reading = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap();
                    let w::Read::Setting(setting) = reading.read() else { continue };
                    let value = if setting.kind() == 7 {
                        let value = w::ProjectWays::new(
                            &w::CEILINGS,
                            w::ProjectWaysParts { drafts: self.offered, goals: true, changes: false, way: 2 },
                        )
                        .unwrap();
                        let mut writer = Writer::new(usize::try_from(value.measure()).unwrap());
                        value.encode(&mut writer).unwrap();
                        Some(writer.finish())
                    } else {
                        None
                    };
                    let answer = w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value }).unwrap();
                    Up::ReadDone {
                        read,
                        result: jig_client::Readout::Family {
                            family: wire::FORGE,
                            record: Some(wire::setting_answer(&answer)),
                        },
                    }
                }
                Down::Open { watch, subject: Watchable::Jig(c::Subject::Task { .. }) } => {
                    Up::Watched { watch, event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task.clone()))) }
                }
                Down::Open { watch, subject: Watchable::Jig(c::Subject::Chips { tasks }) } if self.chip => {
                    assert_eq!(tasks.as_slice(), [GOAL]);
                    let mut chip = task.chip.clone();
                    chip.task = GOAL;
                    chip.title =
                        c::DocumentText::complete(c::Text::new(b"Keep working".as_slice().into(), TEXT).unwrap());
                    chip.phase = c::Phase::Running;
                    chip.details.phase = c::ChipPhase::Active(c::RunStage::Running { attempt: 1 });
                    Up::Watched {
                        watch,
                        event: c::WatchEvent::Delivered(c::Delivery::Chips(List::from_boxed(Box::new([chip])))),
                    }
                }
                Down::Open { watch, subject: Watchable::Family { record, .. } } => {
                    let bytes = match wire::decode_watching(&record.bytes, &w::CEILINGS).unwrap().watch() {
                        w::Watch::Forges => wire::forges(
                            &w::Forges::new(
                                &w::CEILINGS,
                                w::ForgesParts { forges: forges(self.repository), linked: false },
                            )
                            .unwrap(),
                        ),
                        w::Watch::Dismissed => {
                            let mut dismissed = List::with_capacity(wire::DISMISSED);
                            if let Some(at) = self.linked {
                                dismissed
                                    .push(w::PersonFact::GoalFrom(
                                        w::GoalFrom::new(
                                            &w::CEILINGS,
                                            w::GoalFromParts { project: f::PROJECT, chat: CHAT, goal: GOAL, at },
                                        )
                                        .unwrap(),
                                    ))
                                    .unwrap();
                            }
                            wire::dismissals(
                                &w::Dismissals::new(&w::CEILINGS, w::DismissalsParts { dismissed }).unwrap(),
                            )
                        }
                        w::Watch::Change(_)
                        | w::Watch::Landing(_)
                        | w::Watch::Landings(_)
                        | w::Watch::Drafts(_)
                        | w::Watch::Follows(_)
                        | w::Watch::Ways(_)
                        | w::Watch::Trials
                        | w::Watch::Approvals
                        | w::Watch::Findings(_)
                        | w::Watch::Changes(_)
                        | w::Watch::Status(_)
                        | w::Watch::Away
                        | w::Watch::Devices
                        | w::Watch::Decided(_) => continue,
                    };
                    Up::FamilyDelivered {
                        watch,
                        record: jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes },
                    }
                }
                Down::Read { read, what: Readable::Jig(c::Read::Activity { .. }) } => Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(
                        Box::new(activity()),
                    ))),
                },
                Down::Read { read, what: Readable::Jig(c::Read::Notes { scope, .. }) } => {
                    let mut rows = List::uncapped();
                    if let Some(body) = self.body {
                        rows.push(c::Note {
                            number: 1,
                            revision: c::Revision { value: 1 },
                            scope,
                            cut: false,
                            description: c::DocumentText::complete(c::Text::new(TITLE.into(), TEXT).unwrap()),
                            body: c::DocumentText::complete(c::Text::new(body.into(), TEXT).unwrap()),
                            writer: c::NoteAuthor::Task { task: CHAT, attempt: 1 },
                            tasks: List::uncapped(),
                            changed: Some(Wall::EPOCH),
                            acts: List::uncapped(),
                        })
                        .unwrap();
                    }
                    Up::ReadDone {
                        read,
                        result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(
                            c::NotePage { rows, before: None, after: None, cut: false },
                        )))),
                    }
                }
                Down::Save { .. }
                | Down::Erase { .. }
                | Down::Send { .. }
                | Down::Open { .. }
                | Down::Continue { .. }
                | Down::Close { .. }
                | Down::Read { .. } => continue,
            };
            temper_client_domain::step(
                &mut root,
                &env,
                Event::Client { event: jig_client::Event::Up { up } },
                &mut out,
            );
        }
        assert!(out.is_empty());
        assert_eq!(root.pages().chat().unwrap().draft.is_some(), self.body.is_some());
        root
    }
}

fn forges(repository: bool) -> List<w::ForgeRow> {
    let mut rows = connect_forge::first_rows(connect_forge::connected(), connect_forge::made());
    if repository {
        let mut row = rows.get(0).unwrap().clone().into_parts();
        row.repositories = List::with_capacity(1);
        row.repositories
            .push(
                w::Adoption::new(
                    &w::CEILINGS,
                    w::AdoptionParts {
                        owner: b"work".as_slice().into(),
                        name: b"shop".as_slice().into(),
                        project: f::PROJECT,
                    },
                )
                .unwrap(),
            )
            .unwrap();
        *rows.get_mut(0).unwrap() = w::ForgeRow::new(&w::CEILINGS, row).unwrap();
    }
    rows
}

fn activity() -> c::ActivityPage {
    let mut rows = List::uncapped();
    for (number, at, text) in [(1, BEFORE, b"Before".as_slice()), (2, MIDDLE, b"Same time"), (3, AFTER, b"After")] {
        rows.push(documents::turn(
            number,
            Wall::from_nanos(at),
            c::TurnName { attempt: 1, number: u32::try_from(number).unwrap() },
            c::TurnRead::Decoded(c::Turn {
                messages: List::from_boxed(Box::new([c::Message {
                    speaker: c::Speaker::Assistant,
                    blocks: List::from_boxed(Box::new([c::Block::Text(c::Text::new(text.into(), TEXT).unwrap())])),
                }])),
                used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
                spent: 0,
                read_through: None,
                llm: None,
                cut: false,
            }),
        ))
        .unwrap();
    }
    c::ActivityPage { rows, before: None, after: None, cut: false }
}

fn find(tree: &Tree, role: Role, name: &[u8]) -> Option<skein_ui::NodeId> {
    f::find(tree, role, name)
}


#[test]
fn start_goal_shows_when_it_may() {
    for case in [
        Case::READY,
        Case { body: None, ..Case::READY },
        Case { body: Some(b""), ..Case::READY },
        Case { offered: false, ..Case::READY },
        Case { repository: false, ..Case::READY },
    ] {
        let mut root = case.root();
        for words in [f::words(), f::browser_words()] {
            let mut view = f::view_of(words);
            f::build(&mut view, &root);
            let start = find(view.tree(), Role::Button, START);
            let offered = case.body.is_some_and(|body| !body.is_empty()) && case.offered && case.repository;
            assert_eq!(start.is_some(), offered);
            if let Some(start) = start {
                assert_eq!(view.tree().find(start).unwrap().styles.as_slice(), [look::CHAT_GOAL, look::GOAL_PRIMARY]);
                let action = crate::act(&mut view, start, skein_ui::Act::Press).unwrap();
                assert_eq!(
                    action,
                    Action::Pages(Act::OpenStart {
                        draft: Some(b"docs/design/draft/away-mode.md".as_slice().into()),
                        chat: Some(CHAT)
                    })
                );
                f::root_event(&mut root, Event::Act { action });
                assert_eq!(root.pages().start().unwrap().body.as_deref(), Some(BODY));
            }
        }
    }
}

#[test]
fn none_on_an_asking_chat() {
    for linked in [None, Some(MIDDLE)] {
        let root = Case { asking: true, linked, ..Case::READY }.root();
        assert!(crate::pages::chat::asking(&root));
        let view = f::browser_page_view(&root);
        assert!(find(view.tree(), Role::Button, START).is_none());
        assert!(find(view.tree(), Role::Link, LINK).is_none());
        assert!(find(view.tree(), Role::Paragraph, b"Started goal").is_none());
    }
}

#[test]
fn a_linked_goal_shows_its_link() {
    for (body, offered, repository) in [(Some(BODY), true, true), (None, false, false)] {
        for chip in [false, true] {
            let root = Case { linked: Some(MIDDLE), chip, body, offered, repository, ..Case::READY }.root();
            for words in [f::words(), f::browser_words()] {
                let mut view = f::view_of(words);
                f::build(&mut view, &root);
                assert!(find(view.tree(), Role::Button, START).is_none());
                let link = find(view.tree(), Role::Link, LINK);
                assert_eq!(link.is_some(), chip);
                assert_eq!(find(view.tree(), Role::Paragraph, b"Started goal").is_some(), chip);
                if let Some(link) = link {
                    assert_eq!(crate::href(&mut view, link).as_deref(), Some(b"/tasks/42".as_slice()));
                    assert_eq!(
                        crate::act(&mut view, link, skein_ui::Act::Press),
                        Some(Action::Pages(Act::Go { address: Address::Task { task: GOAL } }))
                    );
                }
            }
        }
    }
}

#[test]
fn the_line_stands_by_its_time() {
    for (at, before, after) in [
        (0, None, Some(b"Before".as_slice())),
        (MIDDLE, Some(b"Same time".as_slice()), Some(b"After".as_slice())),
        (AFTER, Some(b"After".as_slice()), None),
    ] {
        let root = Case { linked: Some(at), ..Case::READY }.root();
        for words in [f::words(), f::browser_words()] {
            let mut view = f::view_of(words);
            let tree = f::build(&mut view, &root);
            let text = f::text(tree);
            assert_eq!(skein_lib::bytes::count(&text, b"Started goal", 2), 1);
            let pos = skein_lib::bytes::find(&text, b"Started goal").unwrap();
            if let Some(before) = before {
                assert!(skein_lib::bytes::find(&text, before).unwrap() < pos);
            }
            if let Some(after) = after {
                assert!(pos < skein_lib::bytes::find(&text, after).unwrap());
            }
            let index = tree.nodes().iter().position(|n| n.styles.as_slice().contains(&look::TURN_TOOL)).unwrap();
            assert_eq!(f::node_text(tree, index).as_ref(), b"Started goalKeep workingRunningOpen");
            let end = index + usize::try_from(tree.nodes()[index].size).unwrap();
            assert!(tree.nodes()[index..end].iter().any(|node| node.styles.as_slice().contains(&look::TOOL_FILE)));
            let link = find(tree, Role::Link, b"Open").unwrap();
            assert_eq!(crate::href(&mut view, link).as_deref(), Some(b"/tasks/42".as_slice()));
        }
    }
}
