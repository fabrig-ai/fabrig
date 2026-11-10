//! Revocation before admission, during a connector read, and during durability
//! (jig/docs/design/domain/people/public-reading.md, 10).
use super::{World, delegate, delete, move_out, read, untrack, visibility};
use jig_domain_core as core;
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

fn step(world: &mut World, event: core::Event) -> Queue<core::Request> {
    let env = world.env();
    let mut root_event = jig_test_domain::Event::Core(event);
    world.public_rule_input(&mut root_event);
    let jig_test_domain::Event::Core(event) = root_event else { unreachable!() };
    let mut raw = core::step(world.domain.core_mut(), &env.with(env.limits.core), event);
    let mut out = Queue::uncapped();
    while let Some(request) = raw.pop() {
        if let core::Request::Held(held) = request {
            if matches!(*held, core::Held::PublicRead { .. }) {
                let mut output = jig_test_domain::Request::Deliver(jig_test_domain::Delivery::Core(*held));
                world.public_rule_output(&mut output);
                let jig_test_domain::Request::Deliver(jig_test_domain::Delivery::Core(held)) = output else {
                    unreachable!()
                };
                out.push(core::Request::Held(Box::new(held)));
            } else {
                out.push(core::Request::Held(held));
            }
        } else {
            out.push(request);
        }
    }
    out
}

fn waiting(mut out: Queue<core::Request>) -> Token {
    let Some(core::Request::Held(held)) = out.pop() else { panic!("durability callback") };
    let core::Held::PublicWait { read } = *held else { panic!("public wait") };
    assert!(out.is_empty(), "read makes no other output");
    read
}
fn unknown(mut out: Queue<core::Request>) {
    let Some(core::Request::Held(held)) = out.pop() else { panic!("terminal") };
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
                            core::Event::PublicRead { reply: ReplyTo::new(Token::new(600)), read },
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
                let mut out = step(&mut world, core::Event::PublicRead { reply: ReplyTo::new(Token::new(600)), read });
                let Some(core::Request::Ask { connector: 1, ask: core::Ask::Public { reply, public, .. } }) = out.pop()
                else {
                    panic!("connector public ask")
                };
                assert!(out.is_empty(), "no write or premature answer");
                assert_eq!(public.as_ref(), &[goal, child]);
                let connector = reply.into_token();
                let answer = || Some(core::PublicApplication { tasks: Box::new([child]), record: Box::new([7]) });
                let mut callback = if matches!(stage, Stage::Durable) {
                    Some(waiting(step(
                        &mut world,
                        core::Event::PublicAnswered { reply: ReplyTo::new(connector), answer: answer() },
                    )))
                } else {
                    None
                };
                change(&mut world, revocation, goal, child);
                if callback.is_none() {
                    callback = Some(waiting(step(
                        &mut world,
                        core::Event::PublicAnswered { reply: ReplyTo::new(connector), answer: answer() },
                    )));
                }
                let callback = callback.expect("waiting read");
                let mut out = step(&mut world, core::Event::PublicDurable { read: callback });
                if matches!(stage, Stage::Durable) {
                    assert_eq!(waiting(out), callback, "newer revocation commit extends the wait");
                    out = step(&mut world, core::Event::PublicDurable { read: callback });
                }
                unknown(out);
                // Retained deletion and host traffic is settled by the world's teardown.
            }
        }
    }
}
