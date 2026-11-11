//! View replies in an Ask temper chat (temper/docs/design/client/pages/chat/conversation.md, 3).
use super::{Address, Domain, Event, List, Queue, Solo, Then, agent_line, base_task, c, doc, step, world_in_layout};
use skein_lib::Token;
use temper_client_protocol_wire::{self as wire, v1 as w};

pub(super) fn world(name: &str) -> Domain {
    world_in_layout(Solo, Address::Chat { task: 10 }, Then::Shown, true, Some(name), None, false).0
}
pub(super) fn task() -> c::Task {
    let mut task = base_task(10, "How changes land");
    task.spec.specification.words = doc("Show me how changes land in tally.");
    task.activity = 1;
    task.budget.numbers.budget = 40;
    task.budget.left = 40;
    task.chip.details.details.budget = task.budget.clone();
    task
}
pub(super) fn activity(name: &str) -> c::ActivityPage {
    let words = match name {
        "chat-view-way" => {
            "Agents can review each change before it lands.\n\n```view\nway Agents review\n```\n\nThis is the way tally uses now."
        }
        "chat-view-proposal" => {
            "The proposal changes who decides a change may land.\n\n```view\nproposal t10/1\n```\n\nYou can compare the two paths."
        }
        "chat-view-charter" => "The coding agent works within its charter.\n\n```view\ncharter 1\n```",
        _ => panic!("a named view sample"),
    };
    let mut rows = List::with_capacity(1);
    rows.push(agent_line(words, 1, None)).expect("one reply");
    c::ActivityPage { rows, before: None, after: None, cut: false }
}

pub(super) fn answer(root: &mut Domain, read: Token, address: &[u8], out: &mut Queue<temper_client_domain::Request>) {
    let view = match address {
        b"way Agents review" => w::View::Way(
            w::WayView::new(
                &w::CEILINGS,
                w::WayViewParts {
                    name: b"Agents review".as_slice().into(),
                    line: b"Agents make the change and review it; temper lands it.".as_slice().into(),
                    path: path(true),
                    current: true,
                },
            )
            .expect("way"),
        ),
        b"proposal t10/1" => w::View::Proposal(
            w::ProposalView::new(&w::CEILINGS, w::ProposalViewParts { before: path(false), after: path(true) })
                .expect("proposal paths"),
        ),
        b"charter 1" => w::View::Charter(
            w::CharterView::new(&w::CEILINGS, w::CharterViewParts { charter: 1, card: card() }).expect("charter"),
        ),
        _ => panic!("the reply's view address"),
    };
    let value = w::ViewAnswer::new(&w::CEILINGS, w::ViewAnswerParts { view }).expect("view answer");
    step(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Family {
                        family: wire::FORGE,
                        record: Some(wire::view_answer(&value)),
                    },
                },
            },
        },
        out,
    );
}

fn card() -> w::Card {
    let mut facts = List::with_capacity(3);
    for (label, value) in [
        ("Model", "Opus · high"),
        ("Can", "read the code and the forge; change and push its branch; run commands; never merge"),
        ("Asked to", "“Make the change on its branch and report what changed and what was checked.”"),
    ] {
        facts
            .push(
                w::Fact::new(
                    &w::CEILINGS,
                    w::FactParts { label: label.as_bytes().into(), value: value.as_bytes().into() },
                )
                .expect("card fact"),
            )
            .expect("three facts");
    }
    w::Card::new(
        &w::CEILINGS,
        w::CardParts {
            name: b"The coding agent".as_slice().into(),
            who: "an agent · makes the change".as_bytes().into(),
            facts,
        },
    )
    .expect("card")
}

fn path(review: bool) -> w::BranchPath {
    use w::{
        Actor::{Agent, Bar, Code, Person},
        PathStep::{Approve, Ask, Branch, Check, Conflict, Merge, Produce, Repair, Resolve, Review},
    };
    let mut stops = List::with_capacity(10);
    for (actor, step, words) in [
        (Person, Ask, "you ask"),
        (Agent, Produce, "produce"),
        (Code, Check, "check"),
        (
            if review { Agent } else { Person },
            if review { Review } else { Approve },
            if review { "review" } else { "you approve" },
        ),
        (Agent, Repair, "repair"),
        (Code, Check, "check"),
        (Code, Conflict, "conflict"),
        (Agent, Resolve, "resolve"),
        (Code, Merge, "merge"),
        (Bar, Branch, "main"),
    ] {
        stops
            .push(
                w::Stop::new(
                    &w::CEILINGS,
                    w::StopParts {
                        needed: false,
                        over: Box::default(),
                        actor,
                        step,
                        words: words.as_bytes().into(),
                        opens: w::Opens::Nothing,
                        card: card(),
                    },
                )
                .expect("path stop"),
            )
            .expect("ten stops");
    }
    w::BranchPath::new(
        &w::CEILINGS,
        w::BranchPathParts {
            repository: Some(b"robin/tally".as_slice().into()),
            branch: b"main".as_slice().into(),
            stops,
        },
    )
    .expect("path")
}
