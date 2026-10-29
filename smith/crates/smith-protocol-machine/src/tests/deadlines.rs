//! Concurrent deadline and close snapshots (protocol/agent.md, section 2).

use super::{component, env, place};
use crate::{Below, BelowEvent, Component, FromDomain, ToDomain};
use skein_io::{Event, Request, kernel};
use skein_lib::{Duration, Queue, Time, Token};
use smith_domain::{run, tools};

// Two checks fill the fixture's process admission table.
const OWNERS: [Token; 2] = [Token::new(31), Token::new(32)];
const CHILDREN: [Token; 2] = [Token::new(41), Token::new(42)];
const PIPES: [[Token; 2]; 2] = [[Token::new(51), Token::new(52)], [Token::new(61), Token::new(62)]];
const DEADLINE: Time = Time::from_nanos(100);

fn running(held: bool) -> (Component, Queue<ToDomain>, Queue<Below>) {
    let mut component = component();
    let mut above = Queue::uncapped();
    let mut below = Queue::uncapped();
    for ((owner, child), pipes) in OWNERS.into_iter().zip(CHILDREN).zip(PIPES) {
        let request = if held {
            FromDomain::Op {
                owner,
                op: tools::Op::Spawn {
                    cwd: place(b"."),
                    command: Box::from(b"check".as_slice()),
                    head: 0,
                    tail: 0,
                    after: Some(Duration::ZERO),
                },
                deadline: DEADLINE,
            }
        } else {
            FromDomain::Check {
                owner,
                program: run::Place { root: Token::new(10), path: Box::from(b"check".as_slice()) },
                deadline: DEADLINE,
                tail: 0,
            }
        };
        component.from_domain(&env(), request, &mut above, &mut below);
        component.from_below(
            &env(),
            BelowEvent::Process(Event::Spawned { owner, child, pipes: Box::new(pipes) }),
            &mut above,
            &mut below,
        );
    }
    while below.pop().is_some() {}
    while above.pop().is_some() {}
    (component, above, below)
}

#[test]
fn simultaneously_due_checks_each_stop_and_answer_after_reap() {
    let (mut component, mut above, mut below) = running(false);
    let mut later = env();
    later.now = DEADLINE;
    for signal in [kernel::Signal::Terminate, kernel::Signal::Kill] {
        component.fire(&later, &mut above, &mut below);
        for child in CHILDREN {
            assert_eq!(below.pop(), Some(Below::Process(Request::Signal { child, to: kernel::Target::Group, signal })));
        }
        assert!(below.is_empty());
        later.now = later.now.checked_add(later.limits.group_stop).expect("test deadline fits");
    }
    component.fire(&later, &mut above, &mut below);
    for child in CHILDREN {
        assert_eq!(below.pop(), Some(Below::Process(Request::Close { entity: child })));
    }
    assert!(above.is_empty());
    for (owner, pipes) in OWNERS.into_iter().zip(PIPES) {
        component.from_below(
            &later,
            BelowEvent::Process(Event::Exited { owner, exit: kernel::Exit::Signal(9) }),
            &mut above,
            &mut below,
        );
        for pipe in pipes {
            component.from_below(&later, BelowEvent::Pipe(Event::Closed { owner: pipe }), &mut above, &mut below);
        }
        component.from_below(&later, BelowEvent::Process(Event::Closed { owner }), &mut above, &mut below);
        assert_eq!(
            above.pop(),
            Some(ToDomain::Checked {
                owner,
                ran: run::Ran { exit: run::Exit::TimedOut, output: Box::new([]), cut: 0 }
            })
        );
    }
    assert!(above.is_empty());
    assert!(below.is_empty());
    component.close(&mut below);
    assert!(component.closed());
}

#[test]
fn close_visits_every_held_job_before_their_terminals() {
    let (mut component, mut above, mut below) = running(true);
    component.close(&mut below);
    for child in CHILDREN {
        assert_eq!(below.pop(), Some(Below::Process(Request::Close { entity: child })));
    }
    assert!(below.is_empty());
    assert!(!component.closed());
    for (owner, pipes) in OWNERS.into_iter().zip(PIPES) {
        for pipe in pipes {
            component.from_below(&env(), BelowEvent::Pipe(Event::Closed { owner: pipe }), &mut above, &mut below);
        }
        component.from_below(&env(), BelowEvent::Process(Event::Closed { owner }), &mut above, &mut below);
    }
    assert!(component.closed());
    assert!(above.is_empty());
}
