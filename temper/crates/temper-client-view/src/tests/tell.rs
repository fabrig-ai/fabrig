//! Tell temper's two placements and single filing (client/pages/needs-you.md, 1).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "ordinary tests inspect semantic subtrees and hand-built deliveries"
)]
use super::fixtures as f;

use crate::look;

use jig_client::{Down, Readable, Up, Watchable, contract as c};

use skein_lib::{List, Queue, Token};

use skein_ui::{Act as UiAct, Role, State};

use temper_client_domain::{
    self as d, Action, Address, Event, Place, Request,
    pages::{self, Act},
};

use temper_client_protocol_wire::{self as wire, v1 as w};


// These navigation fixtures hold the rail, Team's card, and both Tell placements' reads.
fn limits() -> d::Limits {
    let mut limits = f::domain_limits();
    limits.client.domain.handles = 40;
    limits.client.domain.watches = 40;
    limits.client.domain.reads = 12;
    limits
}

fn event(root: &mut d::Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    d::step(root, &skein_lib::Env { now: skein_lib::Time::ZERO, wall: f::NOW, limits: limits() }, event, &mut out);
    out
}

fn go(root: &mut d::Domain, address: Address) -> Queue<Request> {
    event(root, Event::Went { place: Place::Address(address) })
}

fn act(root: &mut d::Domain, act: Act) -> Queue<Request> {
    event(root, Event::Act { action: Action::Pages(act) })
}

fn task() -> c::Task {
    let mut task = super::chat::asking_task();
    task.chip.task = 52;
    task.project = 1;
    task.label = 3;
    task.chip.title = c::DocumentText::complete(c::Text::new(b"Prover".as_slice().into(), 32).unwrap());
    task
}

fn findings() -> w::Findings {
    let mut areas = List::with_capacity(2);
    for (name, number) in [(b"docs".as_slice(), 40), (b"tests", 52)] {
        areas
            .push(
                w::AreaRow::new(
                    &w::CEILINGS,
                    w::AreaRowParts {
                        name: name.into(),
                        words: name.into(),
                        owner: w::AreaOwner::Task(w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number }).unwrap()),
                        holder: Some(w::Holder::Task(
                            w::HolderTask::new(&w::CEILINGS, w::HolderTaskParts { number }).unwrap(),
                        )),
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    w::Findings::new(
        &w::CEILINGS,
        w::FindingsParts {
            project: 1,
            areas,
            rows: List::with_capacity(0),
            cut: false,
            triager: Some(w::AreaOwner::Task(w::AreaTask::new(&w::CEILINGS, w::AreaTaskParts { number: 52 }).unwrap())),
            holder: w::Holder::Task(w::HolderTask::new(&w::CEILINGS, w::HolderTaskParts { number: 52 }).unwrap()),
        },
    )
    .unwrap()
}

fn deliver(root: &mut d::Domain, watch: Token, delivery: c::Delivery) -> Queue<Request> {
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up { up: Up::Watched { watch, event: c::WatchEvent::Delivered(delivery) } },
        },
    )
}

fn family(root: &mut d::Domain, watch: Token, bytes: Box<[u8]>) -> Queue<Request> {
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::FamilyDelivered {
                    watch,
                    record: jig_client::Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes },
                },
            },
        },
    )
}

fn fill(root: &mut d::Domain, mut out: Queue<Request>) {
    while let Some(request) = out.pop() {
        let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down } } = request else { continue };
        let mut more = match down {
            Down::Open { watch, subject: Watchable::Jig(c::Subject::Task { task: 52 }) } => {
                deliver(root, watch, c::Delivery::Task(Box::new(task())))
            }
            Down::Open { watch, subject: Watchable::Jig(c::Subject::Tasks { query, .. }) }
                if query.label == Some(3) =>
            {
                let task = task();
                let mut line = super::chats::line();
                line.chip = task.chip;
                line.project = 1;
                line.label = 3;
                deliver(
                    root,
                    watch,
                    c::Delivery::Tasks(Box::new(c::Tasks { rows: List::from_boxed(Box::new([line])), more: false })),
                )
            }
            Down::Open { watch, subject: Watchable::Family { record, .. } } => {
                match wire::decode_watching(&record.bytes, &w::CEILINGS).map(|w| w.watch().clone()) {
                    Some(w::Watch::Findings(_)) => family(root, watch, wire::findings(&findings())),
                    Some(w::Watch::Decided(_)) => {
                        let row = w::Decision::new(
                            &w::CEILINGS,
                            w::DecisionParts {
                                number: 1,
                                task: 99,
                                at: 0,
                                question: b"Which tests?".as_slice().into(),
                                took: b"The focused suite".as_slice().into(),
                                why: b"It covers the rule.".as_slice().into(),
                                overturned: false,
                                reach: temper_protocol_reaches::reach_record(&temper_protocol_reaches::Reach {
                                    kind: temper_protocol_reaches::Kind::Tests,
                                    class: temper_protocol_reaches::Class::Decide,
                                    reason: temper_protocol_reaches::Reason::Kind,
                                    under: 0,
                                    sets: 0,
                                    matched: 0,
                                    away: false,
                                }),
                            },
                        )
                        .unwrap();
                        family(
                            root,
                            watch,
                            wire::decisions(
                                &w::Decisions::new(
                                    &w::CEILINGS,
                                    w::DecisionsParts {
                                        project: 1,
                                        rows: List::from_boxed(Box::new([row])),
                                        asked: List::with_capacity(0),
                                        cut: false,
                                    },
                                )
                                .unwrap(),
                            ),
                        )
                    }
                    _ => Queue::uncapped(),
                }
            }
            Down::Read { read, what: Readable::Jig(_) } => event(
                root,
                Event::Client {
                    event: jig_client::Event::Up {
                        up: Up::ReadDone {
                            read,
                            result: jig_client::Readout::Jig(c::ReadResult::None(c::ReadNone::Unreachable)),
                        },
                    },
                },
            ),
            Down::Save { .. }
            | Down::Erase { .. }
            | Down::Send { .. }
            | Down::Open { .. }
            | Down::Continue { .. }
            | Down::Close { .. }
            | Down::Read { .. } => Queue::uncapped(),
        };
        while let Some(request) = more.pop() {
            out.push(request);
        }
    }
}

fn fixture(address: Address) -> d::Domain {
    let (mut root, watch, _, _, _) =
        f::root_at_full(Address::Work { project: 2, filter: d::WorkFilter::All }, &limits(), b"shop");
    let me = f::asking_me(root.pages().me(root.client()).unwrap().clone());
    f::deliver_me(&mut root, watch, me);
    let out = go(&mut root, Address::Talk { project: 1, task: 52 });
    fill(&mut root, out);
    let out = go(&mut root, address);
    fill(&mut root, out);
    root
}

fn filtered() -> Address {
    Address::Inbox { filter: d::address::Filter { project: Some(1), ..d::address::Filter::ALL } }
}

fn named(view: &crate::View, role: Role, name: &str) -> skein_ui::NodeId {
    f::find(view.tree(), role, name.as_bytes()).unwrap_or_else(|| panic!("missing {role:?}: {name}"))
}

fn has_text(view: &crate::View, expected: &str) {
    let text = f::text(view.tree());
    let text = core::str::from_utf8(&text).unwrap();
    assert!(text.contains(expected), "missing {expected:?}: {text}");
}

fn write(view: &mut crate::View, root: &mut d::Domain, words: &[u8]) {
    let field = view.tree().nodes().iter().find(|node| node.styles.as_slice().contains(&look::TELL_FIELD)).unwrap().id;
    let action = crate::act(view, field, UiAct::Input { value: words.into() }).unwrap();
    event(root, Event::Act { action });
    f::build(view, root);
    let field = view.tree().nodes().iter().find(|node| node.styles.as_slice().contains(&look::TELL_FIELD)).unwrap();
    assert_eq!(field.value.as_ref().unwrap().text.as_ref(), words, "redraw keeps the complete field");
}

#[test]
fn tell_region_follows_decided_and_files_the_kept_page() {
    let mut root = fixture(filtered());
    let mut view = f::browser_page_view(&root);
    let region = named(&view, Role::Region, "Tell temper");
    named(&view, Role::Heading { level: 2 }, "Tell temper");
    let nodes = view.tree().nodes();
    let decided = nodes.iter().position(|n| n.styles.as_slice().contains(&look::DECIDED)).unwrap();
    let at = nodes.iter().position(|n| n.id == region).unwrap();
    assert!(at > decided, "Tell temper follows Decided for you");
    for words in
        ["feedback becomes a finding", "About shop · the page you were on, Prover", "Goes to Prover, tests's owner"]
    {
        has_text(&view, words);
    }
    let file = named(&view, Role::Button, "File it");
    assert!(view.tree().nodes().iter().find(|n| n.id == file).unwrap().states.as_slice().contains(&State::Disabled));
    named(&view, Role::TextArea, "Something off?");
    write(&mut view, &mut root, b"The flake is back");
    let file = named(&view, Role::Button, "File it");
    assert!(!view.tree().nodes().iter().find(|n| n.id == file).unwrap().states.as_slice().contains(&State::Disabled));
    assert_eq!(
        crate::act(&mut view, file, UiAct::Press),
        Some(Action::Pages(Act::Tell {
            project: 1,
            words: b"The flake is back".as_slice().into(),
            about: root.pages().was_on().cloned(),
        }))
    );
    let ask = named(&view, Role::Button, "Ask temper instead");
    let action = crate::act(&mut view, ask, UiAct::Press).unwrap();
    assert_eq!(action, Action::Pages(Act::AskWith { words: b"The flake is back".as_slice().into() }));
    let out = event(&mut root, Event::Act { action });
    assert!(out.iter().any(|r| *r == Request::Show { address: Address::Ask { project: 1 }, replace: false }));
    go(&mut root, Address::Ask { project: 1 });
    assert_eq!(root.pages().address(), &Address::Ask { project: 1 });
    assert_eq!(root.pages().chats().unwrap().words.as_ref(), b"The flake is back");
    assert_eq!(root.pages().chats().unwrap().start, None, "words stay unsent");
    let root = fixture(Address::Inbox { filter: d::address::Filter::ALL });
    let view = f::browser_page_view(&root);
    assert_eq!(f::find(view.tree(), Role::Region, b"Tell temper"), None);
}

#[test]
fn tell_page_is_on_ask_and_the_composer_links_to_it() {
    let css = include_str!("../../../temper-client-shell-web/temper.css");
    for rule in [
        ".temper-tell-page .temper-tell-acts > button { min-height: 44px; }",
        ".temper-tell-top > a, .temper-tell-link { display: inline-flex; align-items: center; min-height: 44px; }",
        "#app > .temper-needs > .temper-tell { display: none; }",
        ".temper-tell-page .temper-tell-field { min-height: 120px; }",
    ] {
        assert!(css.contains(rule), "phone Tell rule: {rule}");
    }

    let mut root = fixture(Address::Tell { project: 1 });
    let mut view = f::browser_page_view(&root);
    for (role, words) in [
        (Role::Link, "‹ Back"),
        (Role::Heading { level: 1 }, "Something off?"),
        (Role::TextArea, "What you saw"),
        (Role::Button, "File it"),
        (Role::Button, "It's a question: ask temper instead"),
    ] {
        named(&view, role, words);
    }
    has_text(
        &view,
        "Say it in your words. temper files it as a finding and sends it to the area's owner, who answers on it.",
    );
    let back = named(&view, Role::Link, "‹ Back");
    assert_eq!(crate::href(&mut view, back).as_deref(), Some(b"/projects/1/ask".as_slice()));
    let nodes = view.tree().nodes();
    let at = nodes.iter().position(|node| node.styles.as_slice().contains(&look::TABBAR_ASK)).unwrap();
    let tabs = &nodes[at..at + usize::try_from(nodes[at].size).unwrap()];
    let tab =
        tabs.iter().find(|n| n.role == Role::Link && n.name.as_deref() == Some(b"Ask temper".as_slice())).unwrap();
    assert!(tab.states.as_slice().contains(&State::Current));
    write(&mut view, &mut root, b"Unsent question");
    let ask = named(&view, Role::Button, "It's a question: ask temper instead");
    let action = crate::act(&mut view, ask, UiAct::Press).unwrap();
    let out = event(&mut root, Event::Act { action });
    assert!(out.iter().any(|r| *r == Request::Show { address: Address::Ask { project: 1 }, replace: false }));
    go(&mut root, Address::Ask { project: 1 });
    assert_eq!(root.pages().chats().unwrap().words.as_ref(), b"Unsent question");
    f::build(&mut view, &root);
    let link = named(&view, Role::Link, "Tell temper");
    assert_eq!(crate::href(&mut view, link).as_deref(), Some(b"/projects/1/tell".as_slice()));
    let nodes = view.tree().nodes();
    let composer = nodes.iter().position(|n| n.role == Role::TextArea).unwrap();
    assert!(nodes.iter().position(|n| n.id == link).unwrap() > composer);
    for (address, title) in [
        (Address::Team { project: 1, role: None }, "Team"),
        (Address::Settings { project: 1 }, "How work runs"),
        (Address::Work { project: 1, filter: d::WorkFilter::All }, "Work"),
        (Address::Talk { project: 1, task: 53 }, "Task 53"),
    ] {
        let mut root = fixture(address);
        let out = go(&mut root, Address::Tell { project: 1 });
        fill(&mut root, out);
        let view = f::browser_page_view(&root);
        has_text(&view, &format!("About shop · the page you were on, {title}"));
    }
}

#[test]
fn tell_ignores_a_kept_page_of_another_project_and_keeps_overlong_words() {
    let mut root = fixture(Address::Tell { project: 1 });
    let out = go(&mut root, Address::Team { project: 2, role: None });
    fill(&mut root, out);
    let out = go(&mut root, Address::Tell { project: 1 });
    fill(&mut root, out);
    let mut tree = f::tree_limits();
    tree.text_per_node = 8192;
    let mut view = crate::View::new(&crate::Limits { tree }, &limits(), f::browser_words(), f::FORGE);
    f::build(&mut view, &root);
    has_text(&view, "About shop");
    assert!(!core::str::from_utf8(&f::text(view.tree())).unwrap().contains("the page you were on"));
    write(&mut view, &mut root, b"Wrong project");
    let file = named(&view, Role::Button, "File it");
    assert_eq!(
        crate::act(&mut view, file, UiAct::Press),
        Some(Action::Pages(Act::Tell { project: 1, words: b"Wrong project".as_slice().into(), about: None }))
    );
    write(&mut view, &mut root, " \u{2003} ".as_bytes());
    let file = named(&view, Role::Button, "File it");
    assert!(view.tree().nodes().iter().find(|n| n.id == file).unwrap().states.as_slice().contains(&State::Disabled));
    let long = vec![b'x'; 4097];
    write(&mut view, &mut root, &long);
    assert_eq!(root.pages().tell_words(), long);
    let file = named(&view, Role::Button, "File it");
    assert!(view.tree().nodes().iter().find(|n| n.id == file).unwrap().states.as_slice().contains(&State::Disabled));
    has_text(&view, "1 byte over the 4,096-byte limit.");
    write(&mut view, &mut root, &long[..4096]);
    let file = named(&view, Role::Button, "File it");
    assert!(!view.tree().nodes().iter().find(|n| n.id == file).unwrap().states.as_slice().contains(&State::Disabled));
}


fn sent(out: &Queue<Request>) -> (Token, w::FileFinding) {
    let mut sends = Vec::new();
    for request in out {
        if let Request::Unlinked {
            request:
                jig_client::UnlinkedRequest::Down {
                    down: Down::Send { attempt, request: jig_client::Ask::Family { record, .. }, .. },
                },
        } = request
            && let w::Request::FileFinding(file) =
                wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().request()
        {
            sends.push((*attempt, file.clone()));
        }
    }
    assert_eq!(sends.len(), 1, "one FileFinding");
    sends.pop().unwrap()
}

fn answer(root: &mut d::Domain, attempt: Token, outcome: w::Outcome) {
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: Up::Answer {
                    attempt,
                    envelope: skein_lib::RequestEnvelope::Final,
                    why: None,
                    answer: Some(jig_client::Answered::Family(jig_client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::answer(&w::Answer::new(&w::CEILINGS, w::AnswerParts { outcome }).unwrap()),
                    })),
                },
            },
        },
    );
}

#[test]
fn tell_filing_is_once_refusal_keeps_words_and_filed_clears_with_a_toast() {
    let mut root = fixture(filtered());
    let mut view = f::browser_page_view(&root);
    write(&mut view, &mut root, b"The flake is back");
    let file = named(&view, Role::Button, "File it");
    let action = crate::act(&mut view, file, UiAct::Press).unwrap();
    let (attempt, filing) = sent(&event(&mut root, Event::Act { action: action.clone() }));
    assert_eq!(filing.body(), b"The flake is back\n\nPage: /projects/1/talk/52 (Prover)");
    let again = event(&mut root, Event::Act { action });
    let mut sends = 0_usize;
    for request in &again {
        if let Request::Unlinked { request: jig_client::UnlinkedRequest::Down { down: Down::Send { .. } } } = request {
            sends += 1;
        }
    }
    assert_eq!(sends, 0);
    f::build(&mut view, &root);
    let file = named(&view, Role::Button, "File it");
    assert!(view.tree().nodes().iter().find(|n| n.id == file).unwrap().states.as_slice().contains(&State::Disabled));
    let reason =
        w::FindingRefusal::Full(w::FindingsFull::new(&w::CEILINGS, w::FindingsFullParts { count: 128 }).unwrap());
    answer(
        &mut root,
        attempt,
        w::Outcome::FindingRefused(
            w::FindingRefused::new(&w::CEILINGS, w::FindingRefusedParts { reason: reason.clone() }).unwrap(),
        ),
    );
    f::build(&mut view, &root);
    assert_eq!(root.pages().tell_words(), b"The flake is back");
    let refusal = crate::shared::finding_refusal(&f::browser_words(), &reason);
    named(&view, Role::Paragraph, core::str::from_utf8(&refusal).unwrap());
    let field = named(&view, Role::TextArea, "Something off?");
    let nodes = view.tree().nodes();
    let at = nodes.iter().position(|n| n.id == field).unwrap();
    assert!(nodes.iter().position(|n| n.name.as_deref() == Some(refusal.as_ref())).unwrap() > at);
    let file = named(&view, Role::Button, "File it");
    let action = crate::act(&mut view, file, UiAct::Press).unwrap();
    let (attempt, _) = sent(&event(&mut root, Event::Act { action }));
    let filed = w::Outcome::Filed(w::Filed::new(&w::CEILINGS, w::FiledParts { number: 12 }).unwrap());
    answer(&mut root, attempt, filed.clone());
    f::build(&mut view, &root);
    assert_eq!(root.pages().tell_words(), b"");
    has_text(&view, "Filed for Prover.");
    act(&mut root, Act::Needs(pages::needs::Act::Dismiss));
    write(&mut view, &mut root, b"Next finding");
    answer(&mut root, attempt, filed);
    f::build(&mut view, &root);
    assert_eq!(root.pages().tell_words(), b"Next finding");
    assert_eq!(root.pages().needs().toast(), None, "serial's outcome was said once");
}
