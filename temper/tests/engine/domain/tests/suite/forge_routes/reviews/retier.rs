//! Held reviewers remain readable after restart (temper/docs/design/domain/testing.md, 2.1).
use super::*;

fn unclaimed_review() -> (World, u64) {
    let (mut world, _, producer, branch) = change_world_with_gate(false, 1000, None, true);
    let review = producer.task + 1; // This fixture delegates one reviewer after its producer.
    produce(&mut world, &producer, &branch);
    world.send(protocol::hello_slots(CHANNEL, 0, Box::new([])));
    let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Live(review)));
    for _ in 0..MAX_TICKS {
        world.tick();
        if world.store.rows.contains_key(&key) && world.store.pending.is_empty() {
            assert_eq!(world.assigned.len(), 2, "no reviewer starts on the worker with no slots");
            assert_eq!(live_task(&world, review).attempt, 0);
            return (world, review);
        }
    }
    panic!("review was not delegated before its first call");
}

fn spent_review() -> (World, u64, u64) {
    let (mut world, reviewer, _) = review_world();
    let spent = live_task(&world, reviewer.task).numbers.budget;
    world.send(protocol::turn(CHANNEL, reviewer.task, reviewer.attempt, 1, spent, None, b"review budget consumed"));
    let event = world.protocol.answer(
        CHANNEL,
        reviewer.task,
        reviewer.attempt,
        spent,
        tasks::End::Parked,
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..MAX_TICKS {
        world.tick();
        if live_task(&world, reviewer.task).numbers.spent == spent && world.store.pending.is_empty() {
            break;
        }
    }
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(981)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [81; 16],
        ask: people::Ask::Say { project: 1, task: reviewer.task, words: b"resume the review".as_slice().into() },
    }));
    (world, reviewer.task, spent)
}

#[test]
fn held_reviewers_remain_readable_after_cold_restart() {
    for budget in [true, false] {
        let (mut world, review, spent) = if budget {
            spent_review()
        } else {
            let (mut world, review) = unclaimed_review();
            world.send(engine::Event::Party(engine::PartyInput::Ask {
                models: Box::new([]),
                reply_to: ReplyTo::new(Token::new(980)),
                sign_in: world.signed_in.expect("owner signed in"),
                key: [80; 16],
                ask: people::Ask::Stop { project: 1, task: review },
            }));
            (world, review, 0)
        };
        for _ in 0..MAX_TICKS {
            world.tick();
            if matches!(live_task(&world, review).phase, tasks::Phase::Held { .. }) && world.store.pending.is_empty() {
                break;
            }
        }
        let held = read_change_task(&mut world, review);
        let reason = held.hold.as_ref().expect("review hold").reason;
        assert!(if budget { reason == tasks::Hold::Budget } else { matches!(reason, tasks::Hold::Placed { .. }) });
        assert_eq!(live_task(&world, review).numbers.spent, spent);
        assert!(world.store.pending.is_empty());
        world.restart(true, true);
        for _ in 0..MAX_TICKS {
            world.tick();
            if world.root.ready() && world.store.pending.is_empty() && world.pending.is_empty() {
                break;
            }
        }
        assert!(world.root.ready(), "restore of held reviewer refused: budget={budget}");
        let restored = read_change_task(&mut world, review);
        assert_eq!(restored.hold, held.hold, "the hold's account remains readable");
        assert_eq!(live_task(&world, review).numbers.spent, spent);
        if !budget {
            assert_eq!(live_task(&world, review).attempt, 0, "no run before or after restart");
        }
        assert_eq!(world.merges, 0);
    }
}
