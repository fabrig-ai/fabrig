//! Reply and document views share the engine's drawing in both shells (temper/docs/design/client/README.md, 5.16).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary tests build wire fixtures and inspect semantic trees and prints"
)]
use super::{documents, fixtures as f};
use crate::{look, paths, words::Shell};
use jig_client::contract as c;
use skein_lib::{Env, List, Queue, Time, Wall};
use skein_ui::{Role, Run, Tree};
use temper_client_domain::{
    self as d,
    pages::{
        Act,
        chat::{Shown, document::Mode},
    },
};
use temper_client_protocol_wire::{self as wire, v1 as w};

const ADDRESS: &[u8] = b"way Free";
const BLOCK: &[u8] = b"```view\nway Free\n```";
const MIXED: &[u8] = b"Before the picture.\n\n```view\nway Free\n```\n\nAfter the picture.";
const INDENTED: &[u8] = b"# Heading\n\n ```view\nway Free\n```";
const HEADED: &[u8] = b"# Heading\n\nBefore the picture.\n\n```view\nway Free\n```\n\nAfter the picture.";
const FENCED: &[u8] = b"````text\n```view\nway Free\n```\n````";

fn card() -> w::Card {
    w::Card::new(
        &w::CEILINGS,
        w::CardParts {
            name: b"The coding agent".as_slice().into(),
            who: "an agent · makes the change".as_bytes().into(),
            facts: List::from_boxed(
                vec![
                    w::Fact::new(
                        &w::CEILINGS,
                        w::FactParts { label: b"Model".as_slice().into(), value: "A model · high".as_bytes().into() },
                    )
                    .unwrap(),
                    w::Fact::new(
                        &w::CEILINGS,
                        w::FactParts {
                            label: b"Can".as_slice().into(),
                            value: b"change and push its branch; never merge".as_slice().into(),
                        },
                    )
                    .unwrap(),
                ]
                .into_boxed_slice(),
            ),
        },
    )
    .unwrap()
}
fn path(repository: bool) -> w::BranchPath {
    let stops = [
        (w::Actor::Person, w::PathStep::Ask, b"you ask".as_slice()),
        (w::Actor::Agent, w::PathStep::Produce, b"produce"),
        (w::Actor::Code, w::PathStep::Merge, b"merge"),
        (w::Actor::Bar, w::PathStep::Branch, b"main"),
    ];
    w::BranchPath::new(
        &w::CEILINGS,
        w::BranchPathParts {
            repository: repository.then(|| b"acme/shop".as_slice().into()),
            branch: b"main".as_slice().into(),
            stops: List::from_boxed(
                stops
                    .into_iter()
                    .map(|(actor, step, words)| {
                        w::Stop::new(
                            &w::CEILINGS,
                            w::StopParts {
                                needed: false,
                                over: Box::default(),
                                actor,
                                step,
                                words: words.into(),
                                opens: w::Opens::Nothing,
                                card: card(),
                            },
                        )
                        .unwrap()
                    })
                    .collect(),
            ),
        },
    )
    .unwrap()
}
fn way(current: bool) -> Shown {
    Shown::Drawn(Box::new(w::View::Way(
        w::WayView::new(
            &w::CEILINGS,
            w::WayViewParts {
                name: b"Free".as_slice().into(),
                line: b"Agents make the change.".as_slice().into(),
                path: path(true),
                current,
            },
        )
        .unwrap(),
    )))
}
fn activity(text: &[u8]) -> c::ActivityPage {
    let turn = c::Turn {
        messages: List::from_boxed(
            vec![c::Message {
                speaker: c::Speaker::Assistant,
                blocks: List::from_boxed(
                    vec![c::Block::Text(c::Text::new(text.into(), 2048).unwrap())].into_boxed_slice(),
                ),
            }]
            .into_boxed_slice(),
        ),
        used: c::Usage { input: 0, output: 0, cached: 0, cache_written: 0 },
        spent: 0,
        read_through: None,
        llm: None,
        cut: false,
    };
    c::ActivityPage {
        rows: List::from_boxed(
            vec![documents::turn(1, Wall::EPOCH, c::TurnName { attempt: 1, number: 1 }, c::TurnRead::Decoded(turn))]
                .into_boxed_slice(),
        ),
        before: None,
        after: None,
        cut: false,
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "the fixture answers activity, document and view reads through their public events"
)]
fn root(text: &[u8], shown: &Shown, document: bool) -> d::Domain {
    let mut limits = f::domain_limits();
    limits.client.domain.long_text = 2048;
    limits.client.domain.turn_text = 2048;
    let (mut root, _, watch, _, _) = f::root_at_with_activity(
        d::Address::Chat { task: f::chat::TASK },
        &limits,
        b"Shop",
        None,
        Some(activity(if document { b"Reply." } else { text })),
        None,
    );
    let source = f::chat::messaging_root();
    let task = source.pages().chat().unwrap().task(source.client()).unwrap().clone();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::uncapped();
    d::step(
        &mut root,
        &env,
        d::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: watch.unwrap(),
                    event: c::WatchEvent::Delivered(c::Delivery::Task(Box::new(task))),
                },
            },
        },
        &mut out,
    );
    for _ in 0_u32..64 {
        let Some(request) = out.pop() else { break };
        let d::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Read { read, what } },
        } = request
        else {
            continue;
        };
        let result = match what {
            jig_client::Readable::Jig(c::Read::Activity { .. }) => {
                Some(jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Activity(Box::new(activity(
                    if document { b"Reply." } else { text },
                ))))))
            }
            jig_client::Readable::Jig(c::Read::Notes { scope, .. }) => {
                let rows = if document {
                    vec![c::Note {
                        number: 1,
                        revision: c::Revision { value: 1 },
                        scope,
                        description: c::DocumentText::complete(
                            c::Text::new(b"The document".as_slice().into(), 2048).unwrap(),
                        ),
                        body: c::DocumentText::complete(c::Text::new(text.into(), 2048).unwrap()),
                        writer: c::NoteAuthor::Task { task: f::chat::TASK, attempt: 1 },
                        tasks: List::with_capacity(0),
                        changed: None,
                        acts: List::with_capacity(0),
                        cut: false,
                    }]
                } else {
                    vec![]
                };
                Some(jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Notes(Box::new(c::NotePage {
                    rows: List::from_boxed(rows.into_boxed_slice()),
                    before: None,
                    after: None,
                    cut: false,
                })))))
            }
            jig_client::Readable::Family { record, .. } => {
                let reading = wire::decode_reading(&record.bytes, &w::CEILINGS).unwrap();
                if let w::Read::View(view) = reading.read() {
                    assert_eq!(view.address(), ADDRESS);
                    match shown {
                        Shown::Asking => None,
                        Shown::Failed => Some(jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Refused))),
                        Shown::Drawn(value) => {
                            let answer =
                                w::ViewAnswer::new(&w::CEILINGS, w::ViewAnswerParts { view: *value.clone() }).unwrap();
                            Some(jig_client::Readout::Family {
                                family: wire::FORGE,
                                record: Some(wire::view_answer(&answer)),
                            })
                        }
                    }
                } else {
                    None
                }
            }
            jig_client::Readable::Jig(_) => None,
        };
        if let Some(result) = result {
            d::step(
                &mut root,
                &env,
                d::Event::Client { event: jig_client::Event::Up { up: jig_client::Up::ReadDone { read, result } } },
                &mut out,
            );
        }
    }
    if text != FENCED && text != INDENTED {
        assert_eq!(root.pages().chat().unwrap().view(ADDRESS), Some(shown));
    }
    if document {
        assert!(root.pages().chat().unwrap().draft.is_some());
        d::step(&mut root, &env, d::Event::Act { action: d::Action::Pages(Act::Mode { mode: Mode::Draft }) }, &mut out);
    }
    root
}

struct Row {
    name: &'static str,
    shown: Shown,
    groups: &'static [&'static [u8]],
    labels: &'static [&'static [u8]],
    print: &'static str,
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one table covers every view state and both shells, including committed text and code fences"
)]
fn view_states_draw_in_both_shells_and_committed_text_keeps_its_order() {
    let rows = [
        Row { name: "asking", shown: Shown::Asking, groups: &[], labels: &[], print: "Drawing way Free…" },
        Row { name: "failed", shown: Shown::Failed, groups: &[], labels: &[], print: "Couldn't draw way Free." },
        Row {
            name: "way",
            shown: way(true),
            groups: &[b"Free"],
            labels: &[b"The way a change goes"],
            print: "Free · now",
        },
        Row {
            name: "way not current",
            shown: way(false),
            groups: &[b"Free"],
            labels: &[b"The way a change goes"],
            print: "Agents make the change.",
        },
        Row {
            name: "path",
            shown: Shown::Drawn(Box::new(w::View::Path(path(true)))),
            groups: &[b"The way a change lands on main in acme/shop"],
            labels: &[b"The way a change lands on main in acme/shop"],
            print: "produce, an agent",
        },
        Row {
            name: "path without repository",
            shown: Shown::Drawn(Box::new(w::View::Path(path(false)))),
            groups: &[b"The way a change lands on main"],
            labels: &[b"The way a change lands on main"],
            print: "main, a branch",
        },
        Row {
            name: "proposal",
            shown: Shown::Drawn(Box::new(w::View::Proposal(
                w::ProposalView::new(&w::CEILINGS, w::ProposalViewParts { before: path(true), after: path(false) })
                    .unwrap(),
            ))),
            groups: &[b"Before", b"After"],
            labels: &[b"Before", b"After"],
            print: "After",
        },
        Row {
            name: "charter",
            shown: Shown::Drawn(Box::new(w::View::Charter(
                w::CharterView::new(&w::CEILINGS, w::CharterViewParts { charter: 1, card: card() }).unwrap(),
            ))),
            groups: &[b"The coding agent"],
            labels: &[],
            print: "A model · high",
        },
        Row {
            name: "roles",
            shown: Shown::Drawn(Box::new(w::View::Roles(
                w::RolesView::new(
                    &w::CEILINGS,
                    w::RolesViewParts {
                        roles: List::from_boxed(Box::new([
                            w::CharterView::new(&w::CEILINGS, w::CharterViewParts { charter: 1, card: card() })
                                .unwrap(),
                            w::CharterView::new(
                                &w::CEILINGS,
                                w::CharterViewParts {
                                    charter: 9,
                                    card: w::Card::new(
                                        &w::CEILINGS,
                                        w::CardParts {
                                            name: b"Status agent".as_slice().into(),
                                            who: b"When you look at Home after landings".as_slice().into(),
                                            facts: List::from_boxed(Box::new([w::Fact::new(
                                                &w::CEILINGS,
                                                w::FactParts {
                                                    label: b"Can".as_slice().into(),
                                                    value: b"read the facts in its task; never merge".as_slice().into(),
                                                },
                                            )
                                            .unwrap()])),
                                        },
                                    )
                                    .unwrap(),
                                },
                            )
                            .unwrap(),
                        ])),
                    },
                )
                .unwrap(),
            ))),
            groups: &[b"The coding agent", b"Status agent"],
            labels: &[],
            print: "read the facts in its task; never merge",
        },
        Row {
            name: "rule",
            shown: Shown::Drawn(Box::new(w::View::Rule(
                w::RuleView::new(
                    &w::CEILINGS,
                    w::RuleViewParts {
                        kind: 2,
                        number: 1,
                        card: w::Card::new(
                            &w::CEILINGS,
                            w::CardParts {
                                who: "a landing rule · holds a merge until it is met".as_bytes().into(),
                                name: b"the project's rule 1".as_slice().into(),
                                facts: List::from_boxed(Box::new([w::Fact::new(
                                    &w::CEILINGS,
                                    w::FactParts {
                                        label: b"Covers".as_slice().into(),
                                        value: b"every branch".as_slice().into(),
                                    },
                                )
                                .unwrap()])),
                            },
                        )
                        .unwrap(),
                    },
                )
                .unwrap(),
            ))),
            groups: &[b"the project's rule 1"],
            labels: &[],
            print: "every branch",
        },
        Row {
            name: "not drawn",
            shown: Shown::Drawn(Box::new(w::View::NotDrawn(
                w::NotDrawn::new(
                    &w::CEILINGS,
                    w::NotDrawnParts { words: b"This revision isn't drawn yet.".as_slice().into() },
                )
                .unwrap(),
            ))),
            groups: &[],
            labels: &[],
            print: "This revision isn't drawn yet.",
        },
    ];
    for row in rows {
        for shell in [Shell::Browser, Shell::Terminal] {
            let words = if shell == Shell::Browser { f::browser_words() } else { f::words() };
            let limits = crate::Limits::of(&f::domain_limits(), &words).unwrap().tree;
            let mut b = skein_ui::Builder::new(&limits);
            crate::views::block(&mut b, ADDRESS, Some(&row.shown), &words);
            let tree = b.finish();
            assert_view(&tree, &row, &limits);
            if row.name == "asking" {
                assert_eq!(tree.nodes()[0].styles.as_slice(), &[look::VIEW_MUTED]);
            }
            if row.name.starts_with("way") {
                assert!(
                    tree.nodes()
                        .iter()
                        .any(|n| n.text.iter().any(|t| t.run == Run::Strong && t.text.as_ref() == b"Free"))
                );
                assert!(tree.nodes().iter().enumerate().any(|(i, n)| n.styles.as_slice() == [look::VIEW_MUTED]
                    && f::node_text(&tree, i).as_ref() == b"Agents make the change."));
                assert_eq!(f::text(&tree).windows(b"now".len()).any(|s| s == b"now"), row.name == "way");
            }
            if row.name == "roles" {
                let text = f::text(&tree);
                let first = skein_lib::bytes::find(&text, b"The coding agent").unwrap();
                let last = skein_lib::bytes::find(&text, b"Status agent").unwrap();
                assert!(first < last);
            }
            if row.name == "charter" {
                assert!(f::find(&tree, Role::Heading { level: 3 }, b"The coding agent").is_some());
                assert!(f::find(&tree, Role::Term, b"Model").is_some());
                assert!(f::find(&tree, Role::Description, "A model · high".as_bytes()).is_some());
                assert!(
                    f::text(&tree)
                        .windows("an agent · makes the change".len())
                        .any(|s| s == "an agent · makes the change".as_bytes())
                );
            }
        }
    }
    let row = Row {
        name: "mixed text",
        shown: way(true),
        groups: &[b"Free"],
        labels: &[b"The way a change goes"],
        print: "produce, an agent",
    };
    for document in [false, true] {
        for shell in [Shell::Browser, Shell::Terminal] {
            let words = if shell == Shell::Browser { f::browser_words() } else { f::words() };
            for text in [BLOCK, MIXED, HEADED, FENCED, INDENTED] {
                let root = root(text, &row.shown, document);
                let limits = crate::Limits::of(&f::domain_limits(), &words).unwrap();
                let mut view = crate::View::new(&limits, &f::domain_limits(), words.clone(), f::FORGE);
                crate::build(&mut view, &root, Wall::EPOCH, &mut Queue::uncapped());
                let tree = view.tree();
                if text == FENCED || text == INDENTED {
                    assert!(!tree.nodes().iter().any(|n| n.drawing.is_some()));
                    if text == FENCED || document {
                        assert!(tree.nodes().iter().any(|n| n.text.iter().any(|t| t.run == Run::Code)));
                    }
                    assert!(String::from_utf8_lossy(&f::text(tree)).contains("way Free"));
                } else {
                    assert_view(tree, &row, &limits.tree);
                    assert!(!tree.nodes().iter().any(|n| n.text.iter().any(|t| t.run == Run::Code)));
                    if text == MIXED || text == HEADED {
                        let text = String::from_utf8(f::text(tree).into_vec()).unwrap();
                        assert!(text.find("Before the picture.").unwrap() < text.find("Free · now").unwrap());
                        assert!(text.find("Free · now").unwrap() < text.find("After the picture.").unwrap());
                    }
                }
            }
        }
    }
}

fn assert_view(tree: &Tree, row: &Row, limits: &skein_ui::Limits) {
    for group in row.groups {
        assert!(
            f::find(tree, Role::Group, group).is_some(),
            "{}: group {:?}",
            row.name,
            String::from_utf8_lossy(group)
        );
    }
    let drawings: Vec<_> = tree.nodes().iter().filter_map(|n| n.drawing.as_ref().map(|d| (n, d))).collect();
    assert_eq!(drawings.len(), row.labels.len(), "{}", row.name);
    for ((node, drawing), label) in drawings.iter().zip(row.labels) {
        assert_eq!(drawing.kind, paths::KIND);
        assert_eq!(node.name.as_deref(), Some(*label));
        let mut encoded = skein_lib::Reader::new(&drawing.data);
        assert_eq!(encoded.u8(), Some(4));
        assert_eq!(encoded.u8(), Some(0));
        assert_eq!(encoded.u8(), Some(255));
        for (mark, words) in [(0, b"you ask".as_slice()), (1, b"produce"), (2, b"merge"), (3, b"main")] {
            assert_eq!(encoded.u8(), Some(mark));
            assert_eq!(encoded.u8(), Some(0), "not needed");
            let len = u32::from(encoded.u8().unwrap());
            assert_eq!(encoded.bytes(len), Some(words));
            assert_eq!(encoded.u8(), Some(0), "no over words");
        }
        assert!(encoded.is_empty());
    }
    let mut print = skein_ui_text::Print::new(limits).unwrap();
    skein_ui_text::render(tree, b"", None, &mut print);
    let printed = String::from_utf8_lossy(print.text.as_slice());
    assert!(printed.contains(row.print), "{}: {printed}", row.name);
    if !row.labels.is_empty() {
        for stop in ["you ask, a person", "produce, an agent", "merge, code", "main, a branch"] {
            assert_eq!(
                printed.lines().filter(|line| line.trim().trim_start_matches("- ") == stop).count(),
                row.labels.len(),
                "{printed}"
            );
        }
    }
}
