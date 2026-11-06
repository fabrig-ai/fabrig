//! Whole-output public reads (jig/docs/design/domain/people.md, 13.6 and 13.10).
#[path = "reads/fixtures.rs"]
mod fixtures;
#[path = "reads/races.rs"]
mod races;

use crate::{
    Ask, Core, Event, Held, Limits, PublicAnswer, PublicApplication, PublicBoard, PublicCard, PublicCounts,
    PublicDocument, PublicExecutor, PublicGoal, PublicIndexLimits, PublicLine, PublicPhase, PublicProject,
    PublicProjects, PublicRead, Request, documents::Text, people, tasks,
};
use fixtures::*;
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};

fn read(kind: u16, task: Option<u64>) -> PublicRead {
    PublicRead { owner: 0, version: 1, kind, project: Some(1), task }
}

fn terminal(reply: u64, answer: PublicAnswer) -> Request {
    Request::Held(Box::new(Held::PublicRead { reply: ReplyTo::new(Token::new(reply)), answer }))
}

// Requests contain linear reply handles, so compare their complete Debug records.
fn outputs(mut out: Queue<Request>, expected: &[Request]) {
    assert_eq!(usize::try_from(out.len()).unwrap(), expected.len(), "complete output count: {out:?}");
    for request in expected {
        assert_eq!(format!("{:?}", out.pop().unwrap()), format!("{request:?}"));
    }
    assert!(out.is_empty());
}

fn start(core: &mut Core, limits: &Limits, reply: u64, read: PublicRead) -> Queue<Request> {
    core.start_input();
    let out =
        crate::step(core, &environment(limits), Event::PublicRead { reply: ReplyTo::new(Token::new(reply)), read });
    outputs(crate::finish_decision(core, &environment(limits)), &[]);
    out
}

fn wait(mut out: Queue<Request>) -> Token {
    let before = format!("{out:?}");
    let Some(Request::Held(held)) = out.pop() else { panic!("one durability callback: {before}") };
    let Held::PublicWait { read } = *held else { panic!("durability callback: {before}") };
    assert!(out.is_empty(), "only the callback: {before}");
    read
}

fn asking(mut out: Queue<Request>, read: PublicRead, public: &[u64]) -> Token {
    assert_eq!(out.len(), 1, "only the connector ask");
    let request = out.pop().unwrap();
    let before = format!("{request:?}");
    let Request::Ask { ask: Ask::Public { reply, .. }, .. } = request else { panic!("{before}") };
    let token = reply.into_token();
    let expected = Request::Ask {
        connector: read.owner.checked_sub(1).unwrap(),
        ask: Ask::Public {
            reply: ReplyTo::new(token),
            kind: read.kind,
            version: read.version,
            project: 1,
            task: read.task,
            public: Box::from(public),
        },
    };
    assert_eq!(before, format!("{expected:?}"));
    token
}

fn answered(core: &mut Core, limits: &Limits, token: Token, answer: Option<PublicApplication>) -> Token {
    let out = crate::step(core, &environment(limits), Event::PublicAnswered { reply: ReplyTo::new(token), answer });
    let durable = wait(out);
    assert_eq!(durable, token, "the same pending read waits for durability");
    outputs(crate::finish_decision(core, &environment(limits)), &[]);
    outputs(
        crate::step(core, &environment(limits), Event::PublicAnswered { reply: ReplyTo::new(token), answer: None }),
        &[],
    );
    durable
}

fn finish(core: &mut Core, limits: &Limits, token: Token, reply: u64, answer: PublicAnswer, at: Time) {
    let mut env = environment(limits);
    env.now = at;
    outputs(crate::step(core, &env, Event::PublicDurable { read: token }), &[terminal(reply, answer)]);
    outputs(crate::finish_decision(core, &env), &[]);
    outputs(crate::step(core, &env, Event::PublicDurable { read: token }), &[]);
    outputs(crate::step(core, &env, Event::PublicAnswered { reply: ReplyTo::new(token), answer: None }), &[]);
}

fn line(task: u64, parent: Option<u64>, priority: Option<u32>, working: u32) -> PublicLine {
    PublicLine {
        task,
        title: Text { bytes: Box::from(b"Public title".as_slice()), cut: false },
        parent,
        goal: priority.is_some(),
        priority,
        executor: PublicExecutor::Agent,
        phase: PublicPhase::Due,
        hold: None,
        made: 10,
        changed: 10,
        active: None,
        ended: None,
        spent: if priority.is_some() { Some(0) } else { None },
        delegates: PublicCounts { working, ..zero() },
    }
}

fn document(document: PublicDocument, at: Time) -> PublicAnswer {
    PublicAnswer::Document { decided: at, document }
}

#[test]
fn a_board_and_a_card_are_projected_from_the_index_alone() {
    let (mut core, mut limits) = fixture();
    limits.public.goals = 2;
    limits.public.lines = 2;
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 1, None, Some(7), false);
    make(&mut core, &limits, 2, Some(1), None, false);
    make(&mut core, &limits, 3, Some(2), None, false);
    make(&mut core, &limits, 4, Some(1), None, false);
    make(&mut core, &limits, 6, None, Some(1), false);
    make(&mut core, &limits, 7, None, Some(9), false);
    for task in [8, 9] {
        make(&mut core, &limits, task, None, Some(0), false);
        end(&mut core, &limits, task);
    }
    let ended = |task| PublicGoal {
        line: PublicLine { phase: PublicPhase::Cancelled, ended: Some(10), ..line(task, None, Some(0), 0) },
        tree: Box::new([]),
        folded: 0,
    };
    let board = PublicDocument::Board(PublicBoard {
        project: 1,
        name: Box::from("Project 1"),
        live: Box::new([
            PublicGoal { line: line(6, None, Some(1), 0), tree: Box::new([]), folded: 0 },
            PublicGoal {
                line: line(1, None, Some(7), 1),
                tree: Box::new([line(2, Some(1), None, 1), line(3, Some(2), None, 0)]),
                folded: 1,
            },
        ]),
        live_folded: 1,
        ended: Box::new([ended(9), ended(8)]),
    });
    let token = wait(start(&mut core, &limits, 70, read(2, None)));
    finish(&mut core, &limits, token, 70, document(board, Time::ZERO), Time::ZERO);
    for (task, card) in [
        (
            1,
            PublicCard {
                line: line(1, None, Some(7), 1),
                lineage: Box::new([]),
                delegates: Box::new([line(2, Some(1), None, 1)]),
                folded: 1,
            },
        ),
        (
            2,
            PublicCard {
                line: line(2, Some(1), None, 1),
                lineage: Box::new([1]),
                delegates: Box::new([line(3, Some(2), None, 0)]),
                folded: 0,
            },
        ),
        (
            3,
            PublicCard {
                line: line(3, Some(2), None, 0),
                lineage: Box::new([2, 1]),
                delegates: Box::new([]),
                folded: 0,
            },
        ),
    ] {
        let token = wait(start(&mut core, &limits, 71, read(3, Some(task))));
        finish(&mut core, &limits, token, 71, document(PublicDocument::Card(card), Time::ZERO), Time::ZERO);
    }
}

#[test]
fn a_private_a_deleting_and_a_never_made_project_answer_alike() {
    let (mut core, limits) = fixture();
    make(&mut core, &limits, 1, None, Some(1), false);
    for project in [Some(1), Some(999), Some(u64::MAX), None] {
        outputs(
            start(&mut core, &limits, 80, PublicRead { project, ..read(2, None) }),
            &[terminal(80, PublicAnswer::Unknown)],
        );
    }
    visibility(&mut core, &limits, true);
    deleting(&mut core, &limits);
    outputs(start(&mut core, &limits, 81, read(2, None)), &[terminal(81, PublicAnswer::Unknown)]);
}

#[test]
fn a_card_of_a_task_that_is_not_public_answers_as_a_missing_project() {
    let (mut core, mut limits) = fixture();
    limits.public.lines = 1;
    make(&mut core, &limits, 1, None, None, true);
    make(&mut core, &limits, 2, Some(1), Some(1), false);
    make(&mut core, &limits, 3, Some(2), None, false);
    make(&mut core, &limits, 4, Some(2), None, false);
    // A task of a different project is never indexed in this project's entry.
    another_project(&mut core, &limits, 2, "Other project", true);
    send(&mut core, &limits, tasks::Event::OpenPeriod { reply_to: reply(), project: 2, period: 1, budget: 1000 });
    make_in_project(&mut core, &limits, 2, 5, None, Some(1), false);
    visibility(&mut core, &limits, true);
    for task in [1, 4, 5, 999, u64::MAX] {
        outputs(start(&mut core, &limits, 82, read(3, Some(task))), &[terminal(82, PublicAnswer::Unknown)]);
        outputs(
            start(&mut core, &limits, 83, PublicRead { project: Some(999), ..read(3, Some(task)) }),
            &[terminal(83, PublicAnswer::Unknown)],
        );
    }
    let token = wait(start(&mut core, &limits, 84, read(3, Some(2))));
    let card = PublicCard {
        line: line(2, None, Some(1), 1),
        lineage: Box::new([]),
        delegates: Box::new([line(3, Some(2), None, 0)]),
        folded: 1,
    };
    finish(&mut core, &limits, token, 84, document(PublicDocument::Card(card), Time::ZERO), Time::ZERO);
}

#[test]
fn projects_lists_the_public_projects_alone() {
    let (mut core, limits) = fixture();
    let read = PublicRead { project: None, ..read(1, None) };
    let token = wait(start(&mut core, &limits, 85, read));
    finish(
        &mut core,
        &limits,
        token,
        85,
        document(PublicDocument::Projects(PublicProjects { projects: Box::new([]) }), Time::ZERO),
        Time::ZERO,
    );
    another_project(&mut core, &limits, 4, "Fourth", true);
    another_project(&mut core, &limits, 2, "Private", false);
    another_project(&mut core, &limits, 3, "Third", true);
    visibility(&mut core, &limits, true);
    let token = wait(start(&mut core, &limits, 86, read));
    finish(
        &mut core,
        &limits,
        token,
        86,
        document(
            PublicDocument::Projects(PublicProjects {
                projects: Box::new([
                    PublicProject { project: 1, name: Box::from("Project 1") },
                    PublicProject { project: 3, name: Box::from("Third") },
                    PublicProject { project: 4, name: Box::from("Fourth") },
                ]),
            }),
            Time::ZERO,
        ),
        Time::ZERO,
    );
}

#[test]
fn an_application_read_asks_its_connector_with_the_public_tasks_ascending() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 8, None, Some(1), false);
    make(&mut core, &limits, 2, Some(8), None, false);
    make(&mut core, &limits, 4, None, Some(2), false);
    make(&mut core, &limits, 3, None, None, true);
    let read = PublicRead { owner: 3, kind: 45, version: 7, ..read(2, None) };
    let token = asking(start(&mut core, &limits, 87, read), read, &[2, 4, 8]);
    let answer = PublicApplication { tasks: Box::new([8, 2]), record: Box::new([9, 7]) };
    let token = answered(&mut core, &limits, token, Some(answer.clone()));
    finish(&mut core, &limits, token, 87, document(PublicDocument::Application(answer), Time::ZERO), Time::ZERO);
    outputs(start(&mut core, &limits, 88, PublicRead { owner: 99, ..read }), &[terminal(88, PublicAnswer::Unknown)]);
}

#[test]
fn an_application_answer_naming_a_task_not_handed_is_unknown() {
    for task in [999, 2] {
        let (mut core, limits) = fixture();
        visibility(&mut core, &limits, true);
        make(&mut core, &limits, 1, None, Some(1), false);
        let read = PublicRead { owner: 2, ..read(3, Some(1)) };
        let token = asking(start(&mut core, &limits, 89, read), read, &[1]);
        // Even a newly public task was not handed to this connector.
        make(&mut core, &limits, 2, None, Some(2), false);
        let token = answered(
            &mut core,
            &limits,
            token,
            Some(PublicApplication { tasks: Box::new([task]), record: Box::new([]) }),
        );
        finish(&mut core, &limits, token, 89, PublicAnswer::Unknown, Time::ZERO);
    }
}

#[test]
fn a_connector_answering_none_is_unknown() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    let read = PublicRead { owner: 2, ..read(2, None) };
    let token = asking(start(&mut core, &limits, 90, read), read, &[]);
    let token = answered(&mut core, &limits, token, None);
    finish(&mut core, &limits, token, 90, PublicAnswer::Unknown, Time::ZERO);
}

#[test]
fn decided_is_the_completing_steps_time() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    make(&mut core, &limits, 1, None, Some(1), false);
    let token = wait(start(&mut core, &limits, 91, read(3, Some(1))));
    amend_at(
        &mut core,
        &limits,
        1,
        None,
        Some(tasks::Spec {
            job: None,
            words: Box::from(b"Current title\nPrivate amendment".as_slice()),
            parameters: Box::new([]),
            inputs: Box::new([]),
            asks: false,
        }),
        Wall::from_nanos(20_000_000_000),
    );
    let token = wait(crate::step(&mut core, &environment(&limits), Event::PublicDurable { read: token }));
    let current = PublicLine {
        title: Text { bytes: Box::from(b"Current title".as_slice()), cut: false },
        changed: 20,
        ..line(1, None, Some(1), 0)
    };
    let card =
        PublicDocument::Card(PublicCard { line: current, lineage: Box::new([]), delegates: Box::new([]), folded: 0 });
    let at = Time::from_nanos(37);
    finish(&mut core, &limits, token, 91, document(card, at), at);
}

#[test]
fn a_public_read_commits_and_marks_nothing() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    core.start_input();
    let before = format!("{core:?}");
    let token = wait(start(&mut core, &limits, 92, read(2, None)));
    let board = PublicDocument::Board(PublicBoard {
        project: 1,
        name: Box::from("Project 1"),
        live: Box::new([]),
        live_folded: 0,
        ended: Box::new([]),
    });
    finish(&mut core, &limits, token, 92, document(board, Time::ZERO), Time::ZERO);
    assert_eq!(core.public_reads.len(), 0);
    // The slab's generation may advance; every durable child is unchanged.
    core.public_reads = skein_lib::Slab::with_capacity(limits.load_slots);
    assert_eq!(format!("{core:?}"), before);
}
