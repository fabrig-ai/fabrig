//! Held reviewers remain readable after restart (temper/docs/design/domain/testing.md, 2.1).
use super::*;

#[test]
fn held_reviewers_remain_readable_after_cold_restart() {
    for budget in [true, false] {
        let (mut world, reviewer, _) = review_world();
        let spent = if budget { live_task(&world, reviewer.task).numbers.budget } else { 0 };
        if !budget {
            world.send(engine::Event::Party(engine::PartyInput::Ask {
                models: Box::new([]),
                reply_to: ReplyTo::new(Token::new(980)),
                sign_in: world.signed_in.expect("owner signed in"),
                key: [80; 16],
                ask: people::Ask::Stop { project: 1, task: reviewer.task },
            }));
        }
        if budget {
            world.send(protocol::turn(
                CHANNEL,
                reviewer.task,
                reviewer.attempt,
                1,
                spent,
                None,
                b"review budget consumed",
            ));
        }
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
        if budget {
            world.send(engine::Event::Party(engine::PartyInput::Ask {
                models: Box::new([]),
                reply_to: ReplyTo::new(Token::new(981)),
                sign_in: world.signed_in.expect("owner signed in"),
                key: [81; 16],
                ask: people::Ask::Say {
                    project: 1,
                    task: reviewer.task,
                    words: b"resume the review".as_slice().into(),
                },
            }));
        }
        for _ in 0..MAX_TICKS {
            world.tick();
            if matches!(live_task(&world, reviewer.task).phase, tasks::Phase::Held { .. })
                && world.store.pending.is_empty()
            {
                break;
            }
        }
        assert!(
            matches!(live_task(&world, reviewer.task).phase, tasks::Phase::Held { .. }),
            "reviewer not held: {:?}",
            live_task(&world, reviewer.task)
        );
        let held = read_change_task(&mut world, reviewer.task);
        let reason = held.hold.as_ref().expect("review hold").reason;
        assert!(if budget { reason == tasks::Hold::Budget } else { matches!(reason, tasks::Hold::Placed { .. }) });
        assert_eq!(live_task(&world, reviewer.task).numbers.spent, spent);
        assert!(world.store.pending.is_empty());
        world.restart(true, true);
        for _ in 0..MAX_TICKS {
            world.tick();
            if world.root.ready() && world.store.pending.is_empty() && world.pending.is_empty() {
                break;
            }
        }
        assert!(world.root.ready(), "restore of held reviewer refused: budget={budget}");
        let restored = read_change_task(&mut world, reviewer.task);
        assert_eq!(restored.hold, held.hold, "the hold's account remains readable");
        assert_eq!(live_task(&world, reviewer.task).numbers.spent, spent);
        assert_eq!(world.merges, 0);
    }
}
