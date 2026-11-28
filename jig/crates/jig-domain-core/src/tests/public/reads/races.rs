//! Revocation decisions finish independently of readers (jig/docs/design/domain/people/public-reading.md, 6).
use super::*;
use crate::tests::new_core;
use jig_domain_contract as contract;

#[derive(Clone, Copy, Debug)]
enum Change {
    Private,
    Deleting,
    Untracked,
    Moved,
}
#[derive(Clone, Copy, Debug)]
enum While {
    Asking,
    Durable,
    Board,
    Card,
}

fn change(core: &mut Core, limits: &Limits, change: Change) {
    match change {
        Change::Private => visibility_in_decision(core, limits, false),
        Change::Deleting => deleting_in_decision(core, limits),
        Change::Untracked => {
            amend_in_decision(
                core,
                limits,
                1,
                Some(tasks::Tracking::Untracked),
                None,
                Wall::from_nanos(10_000_000_000),
            );
        }
        Change::Moved => send_in_decision(
            core,
            limits,
            tasks::Event::Move {
                operation: tasks::Move { task: 2, reason: Box::new([]) },
                reply_to: reply(),
                person: 1,
                period: 1,
                pool_budget: 500,
                period_budget: 1000,
            },
        ),
    }
}

fn race(revocation: Change) {
    for stage in [While::Asking, While::Durable, While::Board, While::Card] {
        for before in [true, false] {
            let (mut core, limits) = fixture();
            visibility(&mut core, &limits, true);
            make(&mut core, &limits, 1, None, Some(1), false);
            make(&mut core, &limits, 2, Some(1), None, false);
            let read = match stage {
                While::Asking | While::Durable => PublicRead { owner: 2, ..read(3, Some(2)) },
                While::Board => read(2, None),
                While::Card => read(3, Some(2)),
            };
            if before {
                change(&mut core, &limits, revocation);
                finish_changes(&mut core, &limits);
                // A board still exists when a task is removed; only its card is a miss.
                let task_only = match revocation {
                    Change::Untracked | Change::Moved => true,
                    Change::Private | Change::Deleting => false,
                };
                if read.kind == 2 && task_only {
                    continue;
                }
                outputs(start(&mut core, &limits, 93, read), &[terminal(93, PublicAnswer::Unknown)]);
                continue;
            }
            let out = start(&mut core, &limits, 94, read);
            let token = match stage {
                While::Asking | While::Durable => asking(out, read, &[1, 2]),
                While::Board | While::Card => wait(out),
            };
            let answer = PublicApplication { tasks: Box::new([2]), record: Box::new([5]) };
            let token = match stage {
                While::Durable => answered(&mut core, &limits, 1, token, Some(answer.clone())),
                While::Asking | While::Board | While::Card => token,
            };
            change(&mut core, &limits, revocation);
            let token = match stage {
                While::Asking => answered(&mut core, &limits, 1, token, Some(answer)),
                While::Durable | While::Board | While::Card => token,
            };
            finish_changes(&mut core, &limits);
            // The writing decision extends the callback's commit barrier.
            let token = wait(crate::step(&mut core, &environment(&limits), Event::PublicDurable { read: token }));
            finish(&mut core, &limits, token, 94, PublicAnswer::Unknown, Time::from_nanos(99));
        }
    }
}

#[test]
fn made_private_while_under_way_ends_unknown() {
    race(Change::Private);
}
#[test]
fn deletion_admitted_while_under_way_ends_unknown() {
    race(Change::Deleting);
}
#[test]
fn untracked_while_under_way_ends_unknown() {
    race(Change::Untracked);
}
#[test]
fn moved_out_while_under_way_ends_unknown() {
    race(Change::Moved);
}

#[test]
fn a_read_of_a_project_not_yet_built_is_busy() {
    let (_, limits) = fixture();
    let mut core = new_core(crate::tests::config(41), &limits).unwrap();
    let _action = core.start_begin(false, Wall::EPOCH);
    core.counters.deployment.projects = 1;
    assert_eq!(
        core.restore(
            &environment(&limits),
            crate::Record::Core(crate::CoreRecord::Deployment(core.counters.deployment()))
        ),
        crate::Restore::Deployment { commits: 0 }
    );
    let value =
        crate::admission::policy::snapshot(&crate::tests::config(41).seed_projects[0].policy.policy, &[], &[]).unwrap();
    assert_eq!(
        core.restore(
            &environment(&limits),
            crate::Record::People(people::Stored::Policy { project: 1, name: Box::from("Project"), value })
        ),
        crate::Restore::Kept
    );
    assert_eq!(
        core.restore(&environment(&limits), crate::Record::People(people::Stored::Public { project: 1 })),
        crate::Restore::Kept
    );
    // Do not start runtime input: the project is deliberately still in LoadCore.
    outputs(
        crate::step(
            &mut core,
            &environment(&limits),
            Event::PublicRead { reply: ReplyTo::new(Token::new(95)), read: read(3, Some(1)) },
        ),
        &[terminal(95, PublicAnswer::Busy)],
    );
    let read = PublicRead { project: None, ..read(1, None) };
    let token = wait(crate::step(
        &mut core,
        &environment(&limits),
        Event::PublicRead { reply: ReplyTo::new(Token::new(96)), read },
    ));
    outputs(
        crate::step(&mut core, &environment(&limits), Event::PublicDurable { read: token }),
        &[terminal(96, document(PublicDocument::Projects(PublicProjects { projects: Box::new([]) }), Time::ZERO))],
    );
}

#[test]
fn projects_rechecks_each_project_after_the_wait() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    let read = PublicRead { project: None, ..read(1, None) };
    let token = wait(start(&mut core, &limits, 97, read));
    visibility(&mut core, &limits, false);
    let token = wait(crate::step(&mut core, &environment(&limits), Event::PublicDurable { read: token }));
    finish(&mut core, &limits, token, 97, PublicAnswer::Unknown, Time::ZERO);
}

#[test]
fn the_pending_table_is_separate_from_authenticated_loads_and_reclaims() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    let read = PublicRead { owner: 2, ..read(2, None) };
    let first = asking(start(&mut core, &limits, 98, read), read, &[]);
    let second = asking(start(&mut core, &limits, 99, read), read, &[]);
    assert_eq!(core.public_reads.len(), limits.load_slots);
    assert_eq!(core.loads.reads(), 0);
    outputs(start(&mut core, &limits, 100, read), &[terminal(100, PublicAnswer::Busy)]);
    let first = answered(&mut core, &limits, 1, first, None);
    let second = answered(&mut core, &limits, 1, second, None);
    for (token, reply) in [(first, 98), (second, 99)] {
        finish(&mut core, &limits, token, reply, PublicAnswer::Unknown, Time::ZERO);
    }
    let token = asking(start(&mut core, &limits, 101, read), read, &[]);
    assert_ne!(token, first, "stale connector replies cannot name the new read");
    outputs(
        crate::tests::connector_step(
            &mut core,
            &environment(&limits),
            1,
            contract::Up::Parts(contract::parts::Up::Public { reply: ReplyTo::new(first), answer: None }),
        ),
        &[],
    );
    let token = answered(&mut core, &limits, 1, token, None);
    finish(&mut core, &limits, token, 101, PublicAnswer::Unknown, Time::ZERO);
}

#[test]
fn a_newer_commit_extends_the_wait_before_completion() {
    let (mut core, limits) = fixture();
    visibility(&mut core, &limits, true);
    let token = wait(start(&mut core, &limits, 102, read(2, None)));
    let _header = core.counters.commit_header();
    let out = crate::step(&mut core, &environment(&limits), Event::PublicDurable { read: token });
    assert_eq!(wait(out), token, "the older barrier does not complete the read");
    assert_eq!(core.public_reads.len(), 1);
    let board = PublicDocument::Board(PublicBoard {
        project: 1,
        name: Box::from("Project 1"),
        live: Box::new([]),
        live_folded: 0,
        ended: Box::new([]),
    });
    let at = Time::from_nanos(200);
    finish(&mut core, &limits, token, 102, document(board, at), at);
}
