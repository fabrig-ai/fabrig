//! Revocation before admission, during a connector read, and during durability
//! (jig/docs/design/domain/people/public-reading.md, 10).
use super::{World, delegate, delete, delete_event, move_ask, move_out, read, untrack, untrack_ask, visibility};
use jig_domain_contract as jig;
use jig_domain_core as core;
use jig_domain_core::engine::{self, In, Out};
use jig_domain_people as people;
use skein_lib::{Queue, ReplyTo, Token};

#[derive(Clone, Copy, Debug)]
pub enum Change {
    Private,
    Deleting,
    Moved,
    Untracked,
}
#[derive(Clone, Copy, Debug)]
enum Stage {
    Connector,
    Durable,
    Board,
    Card,
}

fn change(world: &mut World, revocation: Change, goal: u64, child: u64) {
    match revocation {
        Change::Private => visibility(world, 60, false),
        Change::Deleting => delete(world),
        Change::Moved => move_out(world, child),
        Change::Untracked => untrack(world, goal),
    }
}

fn step(world: &mut World, input: In) -> Queue<Out> {
    let env = world.env();
    let mut event = jig_test_domain::Event::Engine(input);
    if let jig_test_domain::Event::Engine(In::Core(core_event)) = event {
        event = jig_test_domain::Event::Core(core_event);
        world.public_rule_input(&mut event);
        let jig_test_domain::Event::Core(core_event) = event else { unreachable!() };
        event = jig_test_domain::Event::Engine(In::Core(core_event));
    }
    let jig_test_domain::Event::Engine(input) = event else { unreachable!() };
    let mut out = Queue::uncapped();
    engine::step(world.domain.engine_mut(), &env.with(env.limits.core), input, &mut out);
    let mut checked = Queue::uncapped();
    while let Some(output) = out.pop() {
        if matches!(&output, Out::Core(core::Request::Held(held)) if matches!(**held, core::Held::PublicRead { .. })) {
            let Out::Core(core::Request::Held(held)) = output else { unreachable!() };
            let mut output = jig_test_domain::Request::Deliver(jig_test_domain::Delivery::Core(*held));
            world.public_rule_output(&mut output);
            let jig_test_domain::Request::Deliver(jig_test_domain::Delivery::Core(held)) = output else {
                unreachable!()
            };
            checked.push(Out::Core(core::Request::Held(Box::new(held))));
        } else {
            checked.push(output);
        }
    }
    checked
}

fn revoke_in_decision(world: &mut World, revocation: Change, goal: u64, child: u64) -> Queue<Out> {
    let (key, ask) = match revocation {
        Change::Private => (60, people::Ask::SetVisibility { project: 1, public: false }),
        Change::Moved => (61, move_ask(child)),
        Change::Untracked => (62, untrack_ask(world, goal)),
        Change::Deleting => return step(world, In::Core(delete_event())),
    };
    let jig_test_domain::Event::Core(event) = crate::fixtures::party_ask(8200, world.session(), key, ask) else {
        unreachable!()
    };
    step(world, In::Core(event))
}

fn accept_revocation(world: &mut World, revocation: Change, outputs: Queue<Out>) {
    let env = world.env();
    jig_test_domain::accept_engine_outputs(&mut world.domain, &env, outputs);
    world.drain();
    if let Change::Deleting = revocation {
        assert_eq!(
            world.act_answers.last().expect("revocation act terminal").1,
            people::Outcome::ProjectRemoved { project: 1 }
        );
        return;
    }
    let answer = world.people_answers.last().expect("revocation terminal");
    match revocation {
        Change::Private => {
            assert_eq!(*answer, people::Reply::Outcome(people::Outcome::VisibilitySet { project: 1, public: false }));
        }
        Change::Moved => assert!(matches!(answer, people::Reply::Outcome(people::Outcome::Moved { .. }))),
        Change::Untracked => assert!(matches!(answer, people::Reply::Outcome(people::Outcome::Amended { .. }))),
        Change::Deleting => unreachable!("the act has its own terminal"),
    }
}

fn waiting(mut out: Queue<Out>) -> Token {
    let Some(Out::Core(core::Request::Held(held))) = out.pop() else { panic!("durability callback") };
    let core::Held::PublicWait { read } = *held else { panic!("public wait") };
    assert!(out.is_empty(), "read makes no other output");
    read
}
fn unknown(mut out: Queue<Out>) {
    let Some(Out::Core(core::Request::Held(held))) = out.pop() else { panic!("terminal") };
    let core::Held::PublicRead { answer, .. } = *held else { panic!("public terminal") };
    assert_eq!(answer, core::PublicAnswer::Unknown);
    assert!(out.is_empty(), "whole output is only unknown");
}

/// Application reads script the connector at the core boundary: the reference root's
/// connector answers every public kind unknown synchronously. Jig reads use the root.
/// Every world change still follows its ordinary root decision and real store.
pub fn play(revocation: Change) {
    for &seed in super::SEEDS {
        for stage in [Stage::Connector, Stage::Durable, Stage::Board, Stage::Card] {
            for before in [true, false] {
                eprintln!("public race seed {seed}, {revocation:?}, {stage:?}, change first={before}");
                // Removing a task before a board read leaves a valid smaller board.
                if before && matches!(stage, Stage::Board) && matches!(revocation, Change::Moved | Change::Untracked) {
                    continue;
                }
                let mut world = super::world(seed, false);
                let goal = world.assignments[0].0;
                let child = delegate(&mut world, false);
                visibility(&mut world, 51, true);
                let read = match stage {
                    Stage::Connector | Stage::Durable => core::PublicRead { owner: 2, ..read(3, Some(child)) },
                    Stage::Board => read(2, None),
                    Stage::Card => read(3, Some(child)),
                };
                if before {
                    change(&mut world, revocation, goal, child);
                    if matches!(stage, Stage::Connector | Stage::Durable) {
                        unknown(step(
                            &mut world,
                            In::Core(core::Event::PublicRead { reply: ReplyTo::new(Token::new(600)), read }),
                        ));
                    } else {
                        assert_eq!(super::answer(&mut world, 600, read), core::PublicAnswer::Unknown);
                    }
                    continue;
                }
                if matches!(stage, Stage::Board | Stage::Card) {
                    world.manual = true;
                    world.public_offer(super::event(600, read));
                    // No release occurs between admission and the revoking decision.
                    change(&mut world, revocation, goal, child);
                    let outputs = std::mem::take(&mut world.released);
                    assert!(outputs.into_iter().any(|out| matches!(
                        out,
                        jig_test_domain::Request::Deliver(jig_test_domain::Delivery::Core(core::Held::PublicRead {
                            answer: core::PublicAnswer::Unknown,
                            ..
                        }))
                    )));
                    world.manual = false;
                    continue;
                }
                let mut out =
                    step(&mut world, In::Core(core::Event::PublicRead { reply: ReplyTo::new(Token::new(600)), read }));
                let Some(Out::Ask(
                    connector @ jig::Connector(1),
                    jig::Ask::Parts(jig::parts::Ask::Public { reply, public, .. }),
                )) = out.pop()
                else {
                    panic!("connector public ask");
                };
                assert!(out.is_empty(), "no write or premature answer");
                assert_eq!(public.as_ref(), &[goal, child]);
                let answer = In::Connector(
                    connector,
                    jig::Up::Parts(jig::parts::Up::Public {
                        reply,
                        answer: Some(core::PublicApplication { tasks: Box::new([child]), record: Box::new([7]) }),
                    }),
                );
                let callback = if matches!(stage, Stage::Durable) {
                    let callback = waiting(step(&mut world, answer));
                    change(&mut world, revocation, goal, child);
                    callback
                } else {
                    // The read and its connector answer are in the same decision, with the
                    // revocation between them. Keep that decision's writes and terminal.
                    let outputs = revoke_in_decision(&mut world, revocation, goal, child);
                    let callback = waiting(step(&mut world, answer));
                    accept_revocation(&mut world, revocation, outputs);
                    callback
                };
                let out = step(&mut world, In::Core(core::Event::PublicDurable { read: callback }));
                // The revocation commits either in the answer's decision or in a later one.
                assert_eq!(waiting(out), callback, "newer revocation commit extends the wait");
                let out = step(&mut world, In::Core(core::Event::PublicDurable { read: callback }));
                unknown(out);
                // Retained deletion and host traffic is settled by the world's teardown.
            }
        }
    }
}
